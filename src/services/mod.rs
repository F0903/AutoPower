mod handler_data;
mod power_service;

pub use power_service::PowerService;

use autopower_shared::{logging::Logger, winstr::to_win32_wstr};
use windows::{
    core::PWSTR,
    Win32::System::Services::{StartServiceCtrlDispatcherW, SERVICE_TABLE_ENTRYW},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait WindowsService {
    unsafe extern "system" fn service_main(_arg_num: u32, _args: *mut PWSTR);
    fn get_name() -> &'static str;
}

static LOGGER: Logger = Logger::new("services", "autopower");

pub fn start<S: WindowsService>() -> Result<()> {
    std::panic::set_hook(Box::new(|info| {
        LOGGER.error(format!("Fatal panic!\n\t{}", info));
    }));

    LOGGER.debug("Starting setup...");
    let mut service_name = to_win32_wstr(S::get_name());
    LOGGER.debug(format!("Service name is: {}", unsafe {
        service_name.get_const().display()
    }));
    let service_entry = SERVICE_TABLE_ENTRYW {
        lpServiceName: service_name.get_mut(),
        lpServiceProc: Some(S::service_main),
    };

    unsafe { StartServiceCtrlDispatcherW(&service_entry)? };

    Ok(())
}
