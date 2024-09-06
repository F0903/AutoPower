use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct NotificationCommand {
    pub name: String,
    pub content: String,
}

impl Display for NotificationCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}\n{}", &self.name, &self.content))
    }
}
