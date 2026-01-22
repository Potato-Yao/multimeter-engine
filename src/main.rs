use log::info;
use tokio::net::TcpListener;
use multimeter_engine::handle_request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    #[cfg(not(debug_assertions))]
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    info!("Server starts at 127.0.0.1:8080");

    loop {
        let (socket, _) = listener.accept().await?;

        handle_request(socket);
    }
}
