use autopower_shared::logging::Logger;

mod config;
mod handler_data;
mod notification_provider;
mod power_scheme;
mod services;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const LOGGER: Logger = Logger::new("main", "autopower");

fn main() -> Result<()> {
    LOGGER.debug("Starting...");
    let mut args = std::env::args();
    if let Some(cmd) = args.nth(1) {
        match cmd.as_str() {
            "version" => println!(env!("CARGO_PKG_VERSION")),
            _ => println!("Unknown command."),
        }
        return Ok(());
    }

    if let Err(e) = services::start::<services::PowerService>() {
        LOGGER.error(format!("Fatal error!\n  {}", e))
    }
    Ok(())
}
