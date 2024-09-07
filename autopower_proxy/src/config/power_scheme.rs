use serde::{Deserialize, Serialize};
use windows::Win32::System::SystemServices::{GUID_MIN_POWER_SAVINGS, GUID_TYPICAL_POWER_SAVINGS};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum PowerScheme {
    HighPerformance,
    Balanced,
}

impl PowerScheme {
    pub const fn to_guid(&self) -> windows::core::GUID {
        match self {
            Self::HighPerformance => GUID_MIN_POWER_SAVINGS,
            Self::Balanced => GUID_TYPICAL_POWER_SAVINGS,
        }
    }
}
