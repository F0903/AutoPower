#![windows_subsystem = "windows"]

mod config;
mod display;
mod toast;

use config::PowerConfig;

use autopower_shared::{
    logging::Logger,
    pipe::{Client, Pipe, PIPE_NAME},
    proxy_command::{PowerConfigSelection, ProxyCommand},
    stream::Read,
};
use windows::Win32::System::Com::CoInitialize;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const LOGGER: Logger = Logger::new("main", "autopower_proxy");

fn change_power_config(selection: PowerConfigSelection) -> Result<()> {
    let config = PowerConfig::get_or_create()?;
    match selection {
        PowerConfigSelection::Wired => config.get_wired_config().change_to(),
        PowerConfigSelection::Battery => config.get_battery_config().change_to(),
    }
}

fn execute_command(command: ProxyCommand) -> Result<()> {
    match command {
        ProxyCommand::ChangePowerConfig(selection) => change_power_config(selection),
    }
}

fn read_command(input: &mut Pipe<Client, Read>) -> Result<ProxyCommand> {
    LOGGER.debug(format!("Waiting for input..."));
    let object = input.read_to()?;
    LOGGER.debug(format!("Input object:\n{:?}", object));
    Ok(object)
}

fn input_loop() -> Result<()> {
    let mut input = Pipe::create_client_retrying(PIPE_NAME)
        .map_err(|e| format!("Could not create client pipe!\n{}", e))?;
    LOGGER.debug("Entering input loop...");
    loop {
        let command = match read_command(&mut input) {
            Ok(x) => x,
            Err(e) => {
                LOGGER.error(format!("Could not read command!\n{}", e));
                return Err(e);
            }
        };
        execute_command(command).map_err(|e| format!("Could not execute command!\n{}", e))?;
    }
}

fn run() -> Result<()> {
    unsafe {
        CoInitialize(None)
            .ok()
            .map_err(|e| format!("Could not init COM!\n{}", e))?
    };
    input_loop().map_err(|e| format!("Error occured while waiting for input!\n{}", e))?;
    Ok(())
}

fn main() -> Result<()> {
    LOGGER.debug("Starting proxy...");
    std::panic::set_hook(Box::new(|info| {
        LOGGER.error(info);
    }));
    match run() {
        Ok(_) => (),
        Err(e) => {
            LOGGER.error(format!("Exited with error!\n{}", e));
        }
    }
    Ok(())
}
