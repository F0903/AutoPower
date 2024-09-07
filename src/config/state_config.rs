use crate::{display::RefreshRateMode, power_scheme::PowerScheme};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct StateConfig {
    pub(super) power_scheme: PowerScheme,
    pub(super) change_refresh_rate: bool,
    pub(super) screen_refresh_rate: RefreshRateMode,
}

impl StateConfig {
    pub fn get_power_scheme(&self) -> PowerScheme {
        self.power_scheme
    }

    pub fn should_change_refresh_rate(&self) -> bool {
        self.change_refresh_rate
    }

    pub fn get_refresh_rate(&self) -> RefreshRateMode {
        self.screen_refresh_rate
    }
}
