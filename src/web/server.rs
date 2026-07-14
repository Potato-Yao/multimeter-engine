use crate::get_running_flag;
use crate::web::handle_request;
use futures::{SinkExt, StreamExt};
use log::{error, info};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LinesCodec};

pub async fn start_tcp_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(("127.0.0.1", port)).await?;
    let local_addr = listener.local_addr()?;
    info!("TCP server starts at {}", local_addr);

    while get_running_flag() {
        match listener.accept().await {
            Ok((socket, _)) => handle_tcp_connection(socket),
            Err(e) => error!("TCP accept error: {}", e),
        }
    }

    Ok(())
}

fn handle_tcp_connection(socket: TcpStream) {
    tokio::spawn(async move {
        let mut framed = Framed::new(socket, LinesCodec::new());

        while let Some(Ok(line)) = framed.next().await {
            match handle_request(line) {
                Ok(response) | Err(response) => {
                    let response_str = match serde_json::to_string(&response) {
                        Ok(s) => s,
                        Err(e) => {
                            error!("Failed to serialize response: {}", e);
                            continue;
                        }
                    };
                    if let Err(e) = framed.send(response_str).await {
                        error!("TCP send error: {}", e);
                        break;
                    }
                }
            }
        }
    });
}
