use serde::{Deserialize, Serialize};
use windows::core::GUID;
use windows::Win32::System::SystemServices::{
    GUID_MAX_POWER_SAVINGS, GUID_MIN_POWER_SAVINGS, GUID_TYPICAL_POWER_SAVINGS,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PowerScheme {
    HighPerformance,
    Balanced,
    PowerSavings,
    Custom(String),
}

impl PowerScheme {
    pub fn to_guid(&self) -> GUID {
        match self {
            Self::HighPerformance => GUID_MIN_POWER_SAVINGS,
            Self::Balanced => GUID_TYPICAL_POWER_SAVINGS,
            Self::PowerSavings => GUID_MAX_POWER_SAVINGS,
            Self::Custom(val) => GUID::from(val.as_str()),
        }
    }
}
