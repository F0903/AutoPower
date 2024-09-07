mod config_error;
mod power_scheme;
mod state_config;

pub use config_error::ConfigError;
pub use power_scheme::PowerScheme;
use state_config::StateConfig;

use crate::display::RefreshRateMode;
use autopower_shared::logging::Logger;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
};

const LOGGER: Logger = Logger::new("power_config", "autopower");

#[derive(Serialize, Deserialize, Debug)]
pub struct PowerConfig {
    wired_config: StateConfig,
    battery_config: StateConfig,
}

impl Default for PowerConfig {
    fn default() -> Self {
        Self {
            wired_config: StateConfig {
                state_name: "Wired".to_owned(),
                power_scheme: PowerScheme::HighPerformance,
                change_refresh_rate: true,
                screen_refresh_rate: RefreshRateMode::Max,
                send_notification: true,
            },
            battery_config: StateConfig {
                state_name: "Battery".to_owned(),
                power_scheme: PowerScheme::Balanced,
                change_refresh_rate: true,
                screen_refresh_rate: RefreshRateMode::Value(60),
                send_notification: true,
            },
        }
    }
}

impl PowerConfig {
    fn get(path: &str) -> Result<Self, ConfigError> {
        LOGGER.debug("Reading power config...");
        let fs = File::open(path).map_err(|_| ConfigError::CouldNotLoadOrCreate)?;
        let buf = BufReader::new(fs);
        serde_json::from_reader(buf).map_err(|_| ConfigError::CouldNotLoadOrCreate)
    }

    fn new(path: &str) -> Result<Self, ConfigError> {
        LOGGER.debug("Writing new power config...");
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
