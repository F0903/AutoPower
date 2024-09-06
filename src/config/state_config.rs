use crate::power_scheme::PowerScheme;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct StateConfig {
    pub(super) power_scheme: PowerScheme,
    pub(super) screen_refresh_rate: String, // Use string instead of number so we can specify things like "max" or "min"
}

impl StateConfig {
    pub fn get_power_scheme(&self) -> PowerScheme {
        self.power_scheme
    }

    pub fn get_refresh_rate(&self) -> &str {
        &self.screen_refresh_rate
    }
}
