mod cli;

use log::{debug, info};
use multimeter_engine::config::Config;
use multimeter_engine::engine_init;
use std::time::Duration;

use cli::{CliCommand, parse_cli_command};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    #[cfg(not(debug_assertions))]
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Starting Multimeter Engine Server");
    let config = Config::load()?;
    info!("Loaded config from {}", Config::config_path()?.display());

    if let Some(command) = parse_cli_command() {
        info!("Running in CLI mode; config file is ignored");
        match command {
            CliCommand::Migration(migration) => {
                info!("Migration command: {:?}", migration);
                // todo implementation
            }
        }
    } else {
        info!("Sensor enabled: {}", config.sensor.enable);
        info!(
            "TCP enabled: {}, port: {}",
            config.tcp.enable, config.tcp.port
        );
        info!(
            "HTTP enabled: {}, port: {}",
            config.http.enable, config.http.port
        );
        info!("TUI enabled: {}", config.tui.enable);

        debug!("Initializing engine");
        engine_init(config)?;

        // keep server alive
        while multimeter_engine::get_running_flag() {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    Ok(())
}
