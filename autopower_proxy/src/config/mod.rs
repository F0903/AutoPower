mod power_scheme;
mod state_config;

pub use power_scheme::PowerScheme;
use state_config::StateConfig;

use crate::display::RefreshRateMode;
use autopower_shared::{logging::Logger, util::get_process_exe_path};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    sync::LazyLock,
};

static LOGGER: Logger = Logger::new("power_config", "autopower_proxy");

type Result<T> = crate::Result<T>;

static CACHED_CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    const CONFIG_FILE: &str = "config.json";
    get_process_exe_path().unwrap().with_file_name(CONFIG_FILE)
});

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
    fn get(path: &Path) -> Result<Self> {
        LOGGER.debug(format!("Reading power config at {}", path.display()));
        let fs = File::open(path)?;
        let buf = BufReader::new(fs);
        serde_json::from_reader(buf).map_err(|e| e.into())
    }

    fn new(path: &Path) -> Result<Self> {
        LOGGER.debug(format!("Writing new power config at {}", path.display()));
        let new_config = PowerConfig::default();
        let fs = File::create(path)?;
        let mut buf = BufWriter::new(fs);
        serde_json::to_writer_pretty(&mut buf, &new_config)?;
        buf.flush()?;
        Ok(new_config)
    }

    pub fn get_or_create() -> Result<Self> {
        Self::get(&CACHED_CONFIG_PATH).or_else(|_| Self::new(&CACHED_CONFIG_PATH))
    }

    pub fn get_wired_config(&self) -> &StateConfig {
        &self.wired_config
    }

    pub fn get_battery_config(&self) -> &StateConfig {
        &self.battery_config
    }
}
