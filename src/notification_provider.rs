use crate::{user_process::UserProcess, util::get_service_dir};
use autopower_shared::{logging::Logger, notification_command::NotificationCommand};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const NOTIFICATION_PROVIDER_NAME: &str = "autopower_notification_provider.exe";

const LOGGER: Logger = Logger::new("notifications", "autopower");

pub struct NotificationProvider {
    process: UserProcess,
}

impl NotificationProvider {
    pub fn create() -> Result<Self> {
        LOGGER.debug_log("Creating process...");
        let process = UserProcess::create(format!(
            "{}\\{}",
            get_service_dir()?,
            NOTIFICATION_PROVIDER_NAME
        ))?;
        Ok(NotificationProvider { process })
    }

    pub fn send_display_command(&self, title: &str, description: &str) -> Result<()> {
        LOGGER.debug_log(format!("Sent command:\n{} | {}", title, description));
        let command = NotificationCommand {
            name: "display".to_owned(),
            content: format!("{}\n{}", title, description),
        };
        let mut command_str = serde_json::to_string(&command)?;
        command_str.push('\n');
        self.process.get_writer().write(command_str.as_bytes())?;
        Ok(())
    }

    pub fn terminate(&self) {
        LOGGER.debug_log("Terminating notification provider...");
        self.process.terminate();
    }
}

impl Drop for NotificationProvider {
    fn drop(&mut self) {
        LOGGER.debug_log("Dropping notification provider...");
        self.terminate();
    }
}
