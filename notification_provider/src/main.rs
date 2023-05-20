#![windows_subsystem = "windows"]

mod toast;

use autopower_shared::{
    logging::Logger,
    notification_command::NotificationCommand,
    stream::{HandleStream, Read},
};
use toast::Toast;
use windows::Win32::System::{
    Com::CoInitialize,
    Console::{GetStdHandle, STD_INPUT_HANDLE},
};

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

fn read_notification_command(input_stream: &HandleStream<Read>) -> Result<NotificationCommand> {
    let input = input_stream.read_string()?;
    LOGGER.debug_log(format!("notification_provider: read input:\n{}", input));
    let object = serde_json::from_str::<NotificationCommand>(&input)?;
    Ok(object)
}

fn wait_for_input() -> Result<()> {
    let stdin = unsafe { GetStdHandle(STD_INPUT_HANDLE)? };
    let input_stream = HandleStream::create(stdin);
    LOGGER.debug_log("notification_provider: waiting for input...");
    loop {
        let command = read_notification_command(&input_stream)?;
        execute_command(command)?;
    }
}

fn run() -> Result<()> {
    unsafe { CoInitialize(None)? };
    wait_for_input()?;
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
