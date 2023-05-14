#![windows_subsystem = "windows"]

mod toast;

use autopower_shared::{notifications::NotificationCommand, util::output_debug};
use std::io::Read;
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

fn wait_for_input() -> Result<()> {
    let mut buf = Vec::with_capacity(128);
    loop {
        let mut stdin = std::io::stdin();
        let count = stdin.read_to_end(&mut buf)?;
        let value = std::str::from_utf8(&buf[..count])?;
        let command = serde_json::from_str::<NotificationCommand>(value)?;
        execute_command(command)?;
        buf.clear();
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
