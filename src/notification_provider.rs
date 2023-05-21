use autopower_shared::{
    logging::Logger,
    notification_command::NotificationCommand,
    pipe::{Pipe, Server, PIPE_NAME},
    stream::Write,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const LOGGER: Logger = Logger::new("notifications", "autopower");

pub struct NotificationProvider {
    pipe: Pipe<Server, Write>,
}

impl NotificationProvider {
    pub fn create() -> Result<Self> {
        LOGGER.debug_log("Creating pipe...");
        let pipe = Pipe::create_server(PIPE_NAME)?;
        LOGGER.debug_log("Created pipe, waiting for connection...");
        pipe.connect()?;
        Ok(NotificationProvider { pipe })
    }

    pub fn send_display_command(&self, title: &str, description: &str) -> Result<()> {
        LOGGER.debug_log(format!("Sent command:\n{} | {}", title, description));
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
        LOGGER.debug_log("Terminating notification provider...");
        self.pipe.close();
    }
}

impl Drop for NotificationProvider {
    fn drop(&mut self) {
        LOGGER.debug_log("Dropping notification provider...");
        self.terminate();
    }
}
