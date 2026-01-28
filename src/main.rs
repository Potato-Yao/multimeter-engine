use log::{debug, info};
use multimeter_engine::{handle_request, monitor};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    #[cfg(not(debug_assertions))]
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Starting Multimeter Engine Server");
    debug!("Initializing monitor");
    monitor::init()?;

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    info!("Server starts at {}", listener.local_addr()?);
    println!("Server starts at {}", listener.local_addr()?);

    while multimeter_engine::get_running_flag() {
        let (socket, _) = listener.accept().await?;

        handle_request(socket);
    }

    Ok(())
}
