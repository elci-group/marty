use config::{Config, ConfigError, File, Environment};
use serde::Deserialize;
use std::sync::RwLock;
use lazy_static::lazy_static;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server_port: u16,
    pub log_level: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("Marty.toml").required(false))
            .add_source(Environment::with_prefix("MARTY"))
            .build()?;
        s.try_deserialize()
    }
}

lazy_static! {
    pub static ref SETTINGS: RwLock<Settings> = RwLock::new(Settings::new().expect("Failed to load settings"));
}

