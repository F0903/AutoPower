use super::PowerScheme;
use crate::{
    display::{set_display_refresh_rate, RefreshRateMode},
    notification_provider::NotificationProvider,
};
use serde::{Deserialize, Serialize};
use windows::Win32::System::Power::PowerSetActiveScheme;

type Result<T> = crate::Result<T>; // Perhaps make a custom error in the future.

#[derive(Serialize, Deserialize, Debug)]
pub struct StateConfig {
    pub(super) state_name: String,
    pub(super) power_scheme: PowerScheme,
    pub(super) change_refresh_rate: bool,
    pub(super) screen_refresh_rate: RefreshRateMode,
    pub(super) send_notification: bool,
}

impl StateConfig {
    pub fn change_to(&self, notif_provider: &mut NotificationProvider) -> Result<()> {
        unsafe {
            PowerSetActiveScheme(None, Some(&self.power_scheme.to_guid())).ok()?;
        }

        if self.send_notification {
            notif_provider
                .send_display_command("AutoPower", &format!("Switching to {}.", self.state_name))
                .map_err(|e| format!("Could not send notification!\n{}", e))?;
        }

        if self.change_refresh_rate {
            set_display_refresh_rate(self.screen_refresh_rate)?;
        }
        Ok(())
    }
}
