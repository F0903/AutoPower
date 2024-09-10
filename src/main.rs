use autopower_shared::logging::Logger;

mod debug_utils;
mod proxy;
mod services;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

static LOGGER: Logger = Logger::new("main", "autopower");

fn main() -> Result<()> {
    LOGGER.debug("Starting... (main)");
    let mut args = std::env::args();
    if let Some(cmd) = args.nth(1) {
        match cmd.as_str() {
            "version" => println!(env!("CARGO_PKG_VERSION")),
            _ => println!("Unknown command."),
        }
        return Ok(());
    }

    services::start::<services::PowerService>()
}
