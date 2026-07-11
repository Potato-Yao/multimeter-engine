use log::{debug, info};
use multimeter_engine::config::Config;
use multimeter_engine::engine_init;

fn is_cli_mode() -> bool {
    std::env::args().skip(1).any(|arg| arg == "--cli")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    #[cfg(not(debug_assertions))]
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Starting Multimeter Engine Server");
    debug!("Initializing monitor");
    engine_init()?;

    if is_cli_mode() {
        info!("Running in CLI mode; config file is ignored");
        // todo args handling
    } else {
        let config = Config::load()?;
        info!("Loaded config from {}", Config::config_path()?.display());
        info!("TCP enabled: {}, port: {}", config.tcp.enable, config.tcp.port);
        info!(
            "HTTP enabled: {}, port: {}",
            config.http.enable, config.http.port
        );
        info!("TUI enabled: {}", config.tui.enable);
        // todo functions
    }

    Ok(())
}
