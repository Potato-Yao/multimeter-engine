use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use log::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpConfig {
    pub enable: bool,
    pub port: u16,
}

impl Default for TcpConfig {
    fn default() -> Self {
        Self {
            enable: false,
            port: 5000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    pub enable: bool,
    pub port: u16,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            enable: false,
            port: 5100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    pub enable: bool,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self { enable: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMigrationConfig {
    pub user_package: bool,
    pub configuration: bool,
    pub home_file: Vec<String>,
}

impl Default for SystemMigrationConfig {
    fn default() -> Self {
        Self {
            user_package: true,
            configuration: true,
            home_file: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub tcp: TcpConfig,
    pub http: HttpConfig,
    pub tui: TuiConfig,
    pub system_migration: SystemMigrationConfig,
}

impl Config {
    /// return configuration directory.
    pub fn config_dir() -> Result<PathBuf> {
        #[cfg(debug_assertions)]
        {
            Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".multimeter-engine"))
        }
        #[cfg(not(debug_assertions))]
        {
            #[allow(deprecated)]
            let home = std::env::home_dir().context("failed to determine home directory")?;
            Ok(home.join(".multimeter-engine"))
        }
    }

    /// return the path to the configuration file.
    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("multimeter-engine.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            info!(
                "Config file not found at {}, creating default config",
                path.display()
            );
            let config = Self::default();
            config.save(&path)?;
            return Ok(config);
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read config file {}", path.display()))?;
        let config: Self = toml::from_str(&content)
            .with_context(|| format!("failed to parse config file {}", path.display()))?;

        Ok(config)
    }

    pub fn load_or_default() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read config file {}", path.display()))?;
        let config: Self = toml::from_str(&content)
            .with_context(|| format!("failed to parse config file {}", path.display()))?;

        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create config directory {}", parent.display())
            })?;
        }

        let body = toml::to_string_pretty(self)
            .context("failed to serialize config to TOML")?;

        let mut file = fs::File::create(path)
            .with_context(|| format!("failed to create config file {}", path.display()))?;
        file.write_all(body.as_bytes())
            .with_context(|| format!("failed to write config file {}", path.display()))?;

        Ok(())
    }
}
