use serde::{Deserialize, Serialize};

pub const NOTIFICATION_PIPE_NAME: &str = "AutoPowerPipe";
pub const PIPE_PATH_ROOT: &str = "\\\\.\\pipe\\";

#[derive(Serialize, Deserialize)]
pub struct NotificationCommand {
    pub name: String,
    pub content: String,
}
