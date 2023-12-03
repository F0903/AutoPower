use crate::{NOTIFICATION_PROVIDER, SERVICE_NAME};
use windows::Win32::System::{
    Power::PowerSetActiveScheme,
    SystemServices::{GUID_MIN_POWER_SAVINGS, GUID_TYPICAL_POWER_SAVINGS},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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

    pub const fn get_name(&self) -> &'static str {
        match self {
            Self::HighPerformance => "High Performance",
            Self::Balanced => "Balanced",
        }
    }
}

pub fn set_power_scheme(scheme: PowerScheme) -> Result<()> {
    unsafe {
        PowerSetActiveScheme(None, Some(&scheme.to_guid()))?;
        if let Some(notifications) = &mut NOTIFICATION_PROVIDER {
            notifications
                .send_display_command(
                    SERVICE_NAME,
                    &format!("Switching to {}.", scheme.get_name()),
                )
                .map_err(|e| format!("Could not send notification!\n{}", e))?;
        }
    }
    Ok(())
}
