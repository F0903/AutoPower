use autopower_shared::{logging::Logger, util::get_last_win32_err};

mod debug_utils;
mod proxy;
mod services;
mod user_process;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

static LOGGER: Logger = Logger::new("main", "autopower");

fn main() -> Result<()> {
    let res = user_process::UserProcess::new("autopower_proxy.exe");
    if let Err(e) = res {
        println!("{}", e);
        let err = get_last_win32_err()?;
        println!("{}", err);
    }
    return Ok(());
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
