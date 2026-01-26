mod core;
mod monitor;
mod util;
mod web;
mod external_program;

use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LinesCodec};

pub fn handle_request(socket: TcpStream) {
    tokio::spawn(async move {
        let mut framed = Framed::new(socket, LinesCodec::new());

        while let Some(Ok(line)) = framed.next().await {
            match web::handle_request(line) {
                Ok(response) | Err(response) => {
                    let response_str = serde_json::to_string(&response).unwrap();
                    let _ = framed.send(response_str).await;
                }
            }
        }
    });
}
