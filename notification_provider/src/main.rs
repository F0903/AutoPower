mod toast;

use autopower_shared::{notifications::NotificationCommand, util::output_debug};
use std::io::{BufRead, StdinLock};
use toast::Toast;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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

fn read_notification_command(mut input: StdinLock) -> Result<NotificationCommand> {
    let mut buf = String::new();
    input.read_line(&mut buf)?;
    let object = serde_json::from_str::<NotificationCommand>(unsafe {
        std::str::from_utf8_unchecked(buf.as_bytes())
    })?;
    Ok(object)
}

fn wait_for_input() -> Result<()> {
    loop {
        let stdin = std::io::stdin();
        let command = read_notification_command(stdin.lock())?;
        execute_command(command)?;
    }
}

fn main() -> Result<()> {
    match wait_for_input() {
        Ok(_) => (),
        Err(e) => {
            output_debug(&format!("notification_provider error!\n{}", e))?;
        }
    }
    Ok(())
}
