use log::{debug, info};
use multimeter_engine::{handle_request, monitor};
use tokio::net::TcpListener;

fn parse_port_from_args() -> Result<u16, String> {
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--port" | "-p" => {
                let value = args
                    .next()
                    .ok_or_else(|| format!("Missing value after {arg}"))?;
                return value
                    .parse::<u16>()
                    .map_err(|_| format!("Invalid port '{value}'"));
            }
            _ => {
            }
        }
    }

    Ok(8080)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    #[cfg(not(debug_assertions))]
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let port = match parse_port_from_args() {
        Ok(p) => p,
        Err(msg) => {
            eprintln!("{msg}");
            return Ok(());
        }
    };

    info!("Starting Multimeter Engine Server");
    debug!("Initializing monitor");
    monitor::init()?;

    let listener = TcpListener::bind(("127.0.0.1", port)).await?;
    info!("Server starts at {}", listener.local_addr()?);
    println!("Server starts at {}", listener.local_addr()?);

    while multimeter_engine::get_running_flag() {
        let (socket, _) = listener.accept().await?;

        handle_request(socket);
    }

    Ok(())
}
