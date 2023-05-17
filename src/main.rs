mod client_pipe;
mod notifications;
mod user_process;
mod util;

use autopower_shared::{logging::Logger, winstr::to_win32_wstr};
use notifications::NotificationProvider;
use std::ffi::c_void;
use windows::{
    core::PWSTR,
    Win32::{
        Foundation::{CloseHandle, GetLastError, FALSE, HANDLE, NO_ERROR, TRUE},
        System::{
            Power::{
                self, PowerSetActiveScheme, RegisterPowerSettingNotification,
                POWERBROADCAST_SETTING, SYSTEM_POWER_CONDITION,
            },
            Services::{
                RegisterServiceCtrlHandlerExW, SetServiceStatus, StartServiceCtrlDispatcherW,
                SERVICE_ACCEPT_POWEREVENT, SERVICE_ACCEPT_STOP, SERVICE_CONTROL_POWEREVENT,
                SERVICE_CONTROL_STOP, SERVICE_RUNNING, SERVICE_START_PENDING, SERVICE_STATUS,
                SERVICE_STATUS_CURRENT_STATE, SERVICE_STATUS_HANDLE, SERVICE_STOPPED,
                SERVICE_STOP_PENDING, SERVICE_TABLE_ENTRYW, SERVICE_WIN32_OWN_PROCESS,
            },
            SystemServices::{
                GUID_ACDC_POWER_SOURCE, GUID_MIN_POWER_SAVINGS, GUID_TYPICAL_POWER_SAVINGS,
            },
            Threading::{CreateEventW, SetEvent, WaitForSingleObject, INFINITE},
        },
        UI::WindowsAndMessaging::PBT_POWERSETTINGCHANGE,
    },
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const SERVICE_NAME: &str = "AutoPower";

static mut CURRENT_STATUS: Option<SERVICE_STATUS> = None;
static mut STATUS_HANDLE: Option<SERVICE_STATUS_HANDLE> = None;
static mut STOP_EVENT: Option<HANDLE> = None;
static mut NOTIFICATION_PROVIDER: Option<NotificationProvider> = None;

const LOGGER: Logger = Logger::new("main_service", "autopower");

enum PowerScheme {
    HighPerformance,
    Balanced,
}

impl PowerScheme {
    pub const fn to_guid(&self) -> windows::core::GUID {
        match self {
            Self::HighPerformance => GUID_MIN_POWER_SAVINGS,
            Self::Balanced => GUID_TYPICAL_POWER_SAVINGS,
        }
    }

    pub const fn get_name(&self) -> &'static str {
        match self {
            Self::HighPerformance => "High Performance",
            Self::Balanced => "Balanced",
        }
    }
}

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
        SetServiceStatus(STATUS_HANDLE.ok_or("STATUS_HANDLE was not set!")?, &status);
        CURRENT_STATUS = Some(status);
    }
    Ok(())
}

fn set_power_scheme(scheme: PowerScheme) -> Result<()> {
    unsafe {
        PowerSetActiveScheme(None, Some(&scheme.to_guid()));
        if let Some(notifications) = &NOTIFICATION_PROVIDER {
            notifications
                .send_display_command(
                    SERVICE_NAME,
                    &format!("Switching to {}.", scheme.get_name()),
                )
                .map_err(|e| format!("Could not send notification!\n{}", e))?;
        }
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

fn handle_power_event(event_type: u32, event_data: *mut c_void) -> Result<()> {
    if event_type != PBT_POWERSETTINGCHANGE {
        return Ok(());
    }

    let pbs = event_data as *mut POWERBROADCAST_SETTING;
    if unsafe { (*pbs).PowerSetting } != GUID_ACDC_POWER_SOURCE {
        return Ok(());
    }

    let new_power = unsafe { (*pbs).Data[0] };
    match SYSTEM_POWER_CONDITION(new_power as i32) {
        Power::PoAc => handle_on_wall_power()?,
        Power::PoDc => handle_on_battery_power()?,
        _ => (),
    }
    Ok(())
}

fn handle_stop() -> Result<()> {
    if unsafe { CURRENT_STATUS.ok_or("Current status was not set when stopping!")? }.dwCurrentState
        != SERVICE_RUNNING
    {
        return Ok(());
    }

    set_service_status(SERVICE_STOP_PENDING, Some(4), None).unwrap();
    unsafe { SetEvent(STOP_EVENT.ok_or("Stop event was not created!")?) };
    Ok(())
}

unsafe extern "system" fn service_ctrl_handler(
    ctrl_code: u32,
    event_type: u32,
    event_data: *mut c_void,
    _context: *mut c_void,
) -> u32 {
    let handler_result = match ctrl_code {
        SERVICE_CONTROL_POWEREVENT => handle_power_event(event_type, event_data),
        SERVICE_CONTROL_STOP => handle_stop(),
        _ => Ok(()),
    };
    if let Err(e) = handler_result {
        LOGGER.debug_log(e);
    }
    NO_ERROR.0
}

unsafe extern "system" fn service_main(_arg_num: u32, _args: *mut PWSTR) {
    LOGGER.debug_log("Starting AutoPower...");

    let service_name = to_win32_wstr(SERVICE_NAME);

    STATUS_HANDLE = Some(
        match RegisterServiceCtrlHandlerExW(
            service_name.get_const(),
            Some(service_ctrl_handler),
            None,
        ) {
            Ok(x) => x,
            Err(e) => {
                LOGGER.debug_log(format!(
                    "Could not register service control handler!\n{}",
                    e
                ));
                panic!();
            }
        },
    );

    NOTIFICATION_PROVIDER = Some(match NotificationProvider::create() {
        Ok(x) => x,
        Err(e) => {
            LOGGER.debug_log(format!("Could not create notification provider!\n{}", e));
            panic!();
        }
    });

    if let Err(e) = set_service_status(SERVICE_START_PENDING, None, None) {
        LOGGER.debug_log(format!("Could not set service status!\n{}", e));
    }

    if let Err(e) = RegisterPowerSettingNotification(
        HANDLE(STATUS_HANDLE.unwrap().0),
        &GUID_ACDC_POWER_SOURCE,
        1,
    ) {
        LOGGER.debug_log(format!(
            "Could not register power settings notification!\n{}",
            e
        ));
    }

    STOP_EVENT = Some(match CreateEventW(None, TRUE, FALSE, None) {
        Ok(x) => x,
        Err(err) => {
            LOGGER.debug_log(format!("Could not create stop event!\n{}", err));
            if let Err(e) = set_service_status(SERVICE_STOPPED, None, None) {
                LOGGER.debug_log(format!("Could set service status!\n{}", e));
            }
            panic!();
        }
    });

    if let Err(e) = set_service_status(
        SERVICE_RUNNING,
        None,
        Some(SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_POWEREVENT),
    ) {
        LOGGER.debug_log(format!("Could not set service status!\n{}", e));
    }

    // Wait for exit.
    WaitForSingleObject(STOP_EVENT.unwrap(), INFINITE);
    CloseHandle(STOP_EVENT.unwrap());

    if let Err(e) = set_service_status(SERVICE_STOPPED, Some(3), None) {
        LOGGER.debug_log(format!("Could not set service status!\n{}", e));
    }
    NOTIFICATION_PROVIDER.as_ref().unwrap().terminate();
}

fn service_setup() -> Result<()> {
    LOGGER.debug_log("Starting...");
    let mut service_name = to_win32_wstr(SERVICE_NAME);
    let service_entry = SERVICE_TABLE_ENTRYW {
        lpServiceName: service_name.get_mut(),
        lpServiceProc: Some(service_main),
    };

    let start_success = unsafe { StartServiceCtrlDispatcherW(&service_entry) };
    if !start_success.as_bool() {
        let err = unsafe { GetLastError() };
        println!("Could not start service!\n{}", err.ok().unwrap_err());
    }

    Ok(())
}

fn main() -> Result<()> {
    service_setup()
}
