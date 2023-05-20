use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NotificationCommand {
    pub name: String,
    pub content: String,
}
