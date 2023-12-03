mod notification_provider;
mod power;

use autopower_shared::{logging::Logger, winstr::to_win32_wstr};
use notification_provider::NotificationProvider;
use power::{set_power_scheme, PowerScheme};
use std::ffi::c_void;
use windows::{
    core::PWSTR,
    Win32::{
        Foundation::{CloseHandle, FALSE, HANDLE, NO_ERROR, TRUE},
        System::{
            Power::{
                self, RegisterPowerSettingNotification, POWERBROADCAST_SETTING,
                SYSTEM_POWER_CONDITION,
            },
            Services::{
                RegisterServiceCtrlHandlerExW, SetServiceStatus, StartServiceCtrlDispatcherW,
                SERVICE_ACCEPT_POWEREVENT, SERVICE_ACCEPT_STOP, SERVICE_CONTROL_POWEREVENT,
                SERVICE_CONTROL_STOP, SERVICE_RUNNING, SERVICE_START_PENDING, SERVICE_STATUS,
                SERVICE_STATUS_CURRENT_STATE, SERVICE_STATUS_HANDLE, SERVICE_STOPPED,
                SERVICE_STOP_PENDING, SERVICE_TABLE_ENTRYW, SERVICE_WIN32_OWN_PROCESS,
            },
            SystemServices::GUID_ACDC_POWER_SOURCE,
            Threading::{CreateEventW, SetEvent, WaitForSingleObject, INFINITE},
        },
        UI::WindowsAndMessaging::PBT_POWERSETTINGCHANGE,
    },
};

struct HandlerData {
    event_type: u32,
    event_data: *mut c_void,
}
unsafe impl Send for HandlerData {}
unsafe impl Sync for HandlerData {}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const SERVICE_NAME: &str = "AutoPower";

static mut CURRENT_STATUS: Option<SERVICE_STATUS> = None;
static mut STATUS_HANDLE: Option<SERVICE_STATUS_HANDLE> = None;
static mut STOP_EVENT: Option<HANDLE> = None;
static mut NOTIFICATION_PROVIDER: Option<NotificationProvider> = None;

const LOGGER: Logger = Logger::new("main_service", "autopower");

fn set_service_status(
    state: SERVICE_STATUS_CURRENT_STATE,
    checkpoint: Option<u32>,
    controls: Option<u32>,
) -> Result<()> {
    let status = SERVICE_STATUS {
        dwServiceType: SERVICE_WIN32_OWN_PROCESS,
        dwCurrentState: state,
        dwControlsAccepted: controls.unwrap_or(0),
        dwCheckPoint: checkpoint.unwrap_or(0),
        ..Default::default()
    };
    unsafe {
        SetServiceStatus(STATUS_HANDLE.ok_or("STATUS_HANDLE was not set!")?, &status).unwrap();
        CURRENT_STATUS = Some(status);
    }
    Ok(())
}

fn handle_on_wall_power() -> Result<()> {
    set_power_scheme(PowerScheme::HighPerformance)?;
    Ok(())
}

fn handle_on_battery_power() -> Result<()> {
    set_power_scheme(PowerScheme::Balanced)?;
    Ok(())
}

fn handle_power_event(data: HandlerData) {
    let HandlerData {
        event_type,
        event_data,
    } = data;
    if event_type != PBT_POWERSETTINGCHANGE {
        return;
    }

    let pbs = event_data as *mut POWERBROADCAST_SETTING;
    if unsafe { (*pbs).PowerSetting } != GUID_ACDC_POWER_SOURCE {
        return;
    }

    let new_power = unsafe { (*pbs).Data[0] };
    match SYSTEM_POWER_CONDITION(new_power as i32) {
        Power::PoAc => handle_on_wall_power().unwrap(),
        Power::PoDc => handle_on_battery_power().unwrap(),
        _ => (),
    }
}

fn handle_stop() {
    LOGGER.debug("Received stop event... Stopping...");
    if unsafe {
        CURRENT_STATUS
            .ok_or("Current status was not set when stopping!")
            .unwrap()
    }
    .dwCurrentState
        != SERVICE_RUNNING
    {
        return;
    }

    set_service_status(SERVICE_STOP_PENDING, Some(4), None)
        .map_err(|_| "Could not set service status!")
        .unwrap();
    unsafe { SetEvent(STOP_EVENT.ok_or("Stop event was not created!").unwrap()).unwrap() };
}

unsafe extern "system" fn service_ctrl_handler(
    ctrl_code: u32,
    event_type: u32,
    event_data: *mut c_void,
    _context: *mut c_void,
) -> u32 {
    // Win32 docs say to start new thread for any other work than returning immediately
    match ctrl_code {
        SERVICE_CONTROL_POWEREVENT => {
            let data = HandlerData {
                event_type,
                event_data,
            };
            std::thread::spawn(|| {
                handle_power_event(data);
            });
        }
        SERVICE_CONTROL_STOP => {
            std::thread::spawn(handle_stop);
        }
        x => {
            LOGGER.debug(format!("Received unknown control code: {}", x));
        }
    };
    NO_ERROR.0
}

unsafe extern "system" fn service_main(_arg_num: u32, _args: *mut PWSTR) {
    let service_name = to_win32_wstr(SERVICE_NAME);

    LOGGER.debug("Registering service control handler...");
    STATUS_HANDLE = Some(
        match RegisterServiceCtrlHandlerExW(
            service_name.get_const(),
            Some(service_ctrl_handler),
            None,
        ) {
            Ok(x) => x,
            Err(e) => {
                LOGGER.error(format!(
                    "Could not register service control handler!\n{}",
                    e
                ));
                panic!();
            }
        },
    );

    if let Err(e) = set_service_status(SERVICE_START_PENDING, None, None) {
        LOGGER.error(format!("Could not set service status!\n{}", e));
    }

    NOTIFICATION_PROVIDER = Some(match NotificationProvider::create() {
        Ok(x) => x,
        Err(e) => {
            LOGGER.error(format!("Could not create notification provider!\n{}", e));
            panic!();
        }
    });
    LOGGER.debug("Creation of notification provider successful.");

    LOGGER.debug("Registering power setting notification handling...");
    if let Err(e) = RegisterPowerSettingNotification(
        HANDLE(STATUS_HANDLE.unwrap().0),
        &GUID_ACDC_POWER_SOURCE,
        1,
    ) {
        LOGGER.error(format!(
            "Could not register power settings notification!\n{}",
            e
        ));
    }

    STOP_EVENT = Some(match CreateEventW(None, TRUE, FALSE, None) {
        Ok(x) => x,
        Err(err) => {
            LOGGER.error(format!("Could not create stop event!\n{}", err));
            panic!();
        }
    });

    if let Err(e) = set_service_status(
        SERVICE_RUNNING,
        None,
        Some(SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_POWEREVENT),
    ) {
        LOGGER.error(format!("Could not set service status!\n{}", e));
    }

    // Wait for exit.
    WaitForSingleObject(STOP_EVENT.unwrap(), INFINITE);
    LOGGER.debug("Stop event signaled. Cleaning up and terminating...");
    CloseHandle(STOP_EVENT.unwrap()).unwrap();

    if let Err(e) = set_service_status(SERVICE_STOPPED, Some(3), None) {
        LOGGER.error(format!("Could not set service status!\n{}", e));
    }
    NOTIFICATION_PROVIDER.as_ref().unwrap().terminate().unwrap();
}

fn service_setup() -> Result<()> {
    std::panic::set_hook(Box::new(|info| {
        LOGGER.error(format!("Fatal panic!\n    {}", info));
    }));

    LOGGER.debug("Starting setup...");
    let mut service_name = to_win32_wstr(SERVICE_NAME);
    LOGGER.debug(format!("Service name is: {}", unsafe {
        service_name.get_const().display()
    }));
    let service_entry = SERVICE_TABLE_ENTRYW {
        lpServiceName: service_name.get_mut(),
        lpServiceProc: Some(service_main),
    };

    unsafe { StartServiceCtrlDispatcherW(&service_entry)? };

    Ok(())
}

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

    if let Err(e) = service_setup() {
        LOGGER.error(format!("Fatal error!\n  {}", e))
    }
    Ok(())
}
