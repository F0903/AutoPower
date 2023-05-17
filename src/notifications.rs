use crate::{client_pipe::Pipe, user_process::UserProcess, util::get_service_dir};
use autopower_shared::{logging::Logger, notifications::NotificationCommand};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const NOTIFICATION_PROVIDER_NAME: &str = "autopower_notification_provider.exe";

const LOGGER: Logger = Logger::new("notifications", "autopower");

pub struct NotificationProvider {
    process: UserProcess,
    pipe: Pipe,
}

impl NotificationProvider {
    pub fn create() -> Result<Self> {
        let process = UserProcess::create(format!(
            "{}\\{}",
            get_service_dir()?,
            NOTIFICATION_PROVIDER_NAME
        ))?;
        LOGGER.debug_log("Created notification provider process.");
        let pipe = Pipe::create("AutoPowerPipe")?;
        LOGGER.debug_log("Created notification provider pipe.");
        LOGGER.debug_log("Created notification provider.");
        Ok(NotificationProvider { process, pipe })
    }

    pub fn send_display_command(&self, title: &str, description: &str) -> Result<()> {
        let command = NotificationCommand {
            name: "display".to_owned(),
            content: format!("{}\n{}", title, description),
        };
        let mut command_str = serde_json::to_string(&command)?;
        command_str.push('\n');
        self.pipe.write(command_str.as_bytes())?;
        Ok(())
    }

    pub fn terminate(&self) {
        self.process.terminate();
        self.pipe.close();
    }
}

impl Drop for NotificationProvider {
    fn drop(&mut self) {
        self.terminate();
    }
}
