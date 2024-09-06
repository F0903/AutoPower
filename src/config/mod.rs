mod config_error;
mod state_config;

pub use config_error::ConfigError;
use state_config::StateConfig;

use crate::power_scheme::PowerScheme;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct PowerConfig {
    wired_config: StateConfig,
    battery_config: StateConfig,
}

impl Default for PowerConfig {
    fn default() -> Self {
        Self {
            wired_config: StateConfig {
                power_scheme: PowerScheme::HighPerformance,
                screen_refresh_rate: "max".to_owned(),
            },
            battery_config: StateConfig {
                power_scheme: PowerScheme::Balanced,
                screen_refresh_rate: "60".to_owned(),
            },
        }
    }
}

impl PowerConfig {
    fn get(path: &str) -> Result<Self, ConfigError> {
        let fs = File::open(path).map_err(|_| ConfigError::CouldNotLoadOrCreate)?;
        let buf = BufReader::new(fs);
        serde_json::from_reader(buf).map_err(|_| ConfigError::CouldNotLoadOrCreate)
    }

    fn new(path: &str) -> Result<Self, ConfigError> {
        let new_config = PowerConfig::default();
        let fs = File::create(path).map_err(|_| ConfigError::CouldNotLoadOrCreate)?;
        let mut buf = BufWriter::new(fs);
        serde_json::to_writer_pretty(&mut buf, &new_config)
            .map_err(|_| ConfigError::CouldNotLoadOrCreate)?;
        buf.flush().map_err(|_| ConfigError::CouldNotLoadOrCreate)?;
        Ok(new_config)
    }

    pub fn get_or_create() -> Result<Self, ConfigError> {
        const CONFIG_PATH: &str = "config.json";
        Self::get(CONFIG_PATH).or_else(|_| Self::new(CONFIG_PATH))
    }

    pub fn get_wired_config(&self) -> &StateConfig {
        &self.wired_config
    }

    pub fn get_battery_config(&self) -> &StateConfig {
        &self.battery_config
    }
}