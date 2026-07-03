use config::{Config, ConfigError, Environment, File};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::path::Path;
use std::sync::RwLock;

fn default_server_port() -> u16 {
    7777
}

fn default_log_level() -> String {
    "info".to_string()
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    #[serde(default = "default_server_port")]
    pub server_port: u16,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server_port: default_server_port(),
            log_level: default_log_level(),
        }
    }
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let config_file = Path::new("Marty.toml");
        let s = Config::builder()
            .add_source(File::from(config_file).required(false))
            .add_source(Environment::with_prefix("MARTY"))
            .build()?;
        s.try_deserialize()
    }
}

lazy_static! {
    pub static ref SETTINGS: RwLock<Settings> = RwLock::new(
        Settings::new().unwrap_or_else(|e| {
            // Only warn when a config file is present but unreadable. If the
            // binary is run outside the project directory, silent defaults are
            // the expected behaviour.
            if Path::new("Marty.toml").exists() {
                eprintln!(
                    "Warning: Marty.toml exists but could not be loaded ({}); using defaults.",
                    e
                );
            }
            Settings::default()
        })
    );
}
