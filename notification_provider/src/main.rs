#![windows_subsystem = "windows"]

mod toast;

use autopower_shared::{
    logging::Logger,
    notification_command::NotificationCommand,
    pipe::{Client, Pipe, PIPE_NAME},
    stream::Read,
};
use toast::Toast;
use windows::Win32::System::Com::CoInitialize;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const LOGGER: Logger = Logger::new("main", "autopower_notifier");

fn execute_display_command(command: NotificationCommand) -> Result<()> {
    let mut cmd_lines = command.content.lines();
    let title = cmd_lines.next().expect("Could not get next cmd line!");
    let content = cmd_lines
        .next()
        .expect("Could not get next second cmd line!");
    let toast = Toast::new(title, content);
    toast.send()?;
    Ok(())
}

fn execute_command(command: NotificationCommand) -> Result<()> {
    match command.name.as_str() {
        "display" => execute_display_command(command),
        _ => Ok(()),
    }
}

fn read_notification_command(input: &Pipe<Client, Read>) -> Result<NotificationCommand> {
    let input_string = input
        .read()
        .map_err(|e| format!("Could not read input!\n{}", e))?;
    LOGGER.debug_log(format!(
        "notification_provider: read input:\n{}",
        input_string
    ));
    let object = serde_json::from_str::<NotificationCommand>(&input_string)
        .map_err(|e| format!("Could not convert string to command!\n{}", e))?;
    Ok(object)
}

fn wait_for_input() -> Result<()> {
    let input = Pipe::create_client_retrying(PIPE_NAME)
        .map_err(|e| format!("Could not create client pipe!\n{}", e))?;
    LOGGER.debug_log("notification_provider: waiting for input...");
    loop {
        let command = match read_notification_command(&input) {
            Ok(x) => x,
            Err(e) => {
                LOGGER.debug_log(format!("Could not read command!\n{}", e));
                return Err(e);
            }
        };
        execute_command(command).map_err(|e| format!("Could not execute command!\n{}", e))?;
    }
}

fn run() -> Result<()> {
    unsafe { CoInitialize(None).map_err(|e| format!("Could not init COM!\n{}", e))? };
    wait_for_input().map_err(|e| format!("Error occured while waiting for input!\n{}", e))?;
    Ok(())
}

fn main() -> Result<()> {
    std::panic::set_hook(Box::new(|info| {
        LOGGER.debug_log(info);
    }));
    match run() {
        Ok(_) => (),
        Err(e) => {
            LOGGER.debug_log(format!("Exited with error!\n{}", e));
        }
    }
    Ok(())
}
