#[cfg(feature = "notification")]
mod notifications;

use autopower_shared::util::{output_debug, to_cw_str, to_w_str};
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

fn handle_on_wall_power() {
    unsafe {
        PowerSetActiveScheme(None, Some(&GUID_MIN_POWER_SAVINGS));
        if let Some(notifications) = &NOTIFICATION_PROVIDER {
            notifications
                .send_display_command(SERVICE_NAME, "Switching to High Performance.")
                .ok();
        }
    }
}

fn handle_on_battery_power() {
    unsafe {
        PowerSetActiveScheme(None, Some(&GUID_TYPICAL_POWER_SAVINGS));
        if let Some(notifications) = &NOTIFICATION_PROVIDER {
            notifications
                .send_display_command(SERVICE_NAME, "Switching to Balanced.")
                .ok();
        }
    }
}

fn handle_power_event(event_type: u32, event_data: *mut c_void) {
    if event_type != PBT_POWERSETTINGCHANGE {
        return;
    }

    let pbs = event_data as *mut POWERBROADCAST_SETTING;
    if unsafe { (*pbs).PowerSetting } != GUID_ACDC_POWER_SOURCE {
        return;
    }

    let new_power = unsafe { (*pbs).Data[0] };
    match SYSTEM_POWER_CONDITION(new_power as i32) {
        Power::PoAc => handle_on_wall_power(),
        Power::PoDc => handle_on_battery_power(),
        _ => (),
    }
}

fn handle_stop() {
    if unsafe { CURRENT_STATUS.unwrap() }.dwCurrentState != SERVICE_RUNNING {
        return;
    }

    set_service_status(SERVICE_STOP_PENDING, Some(4), None).unwrap();
    unsafe { SetEvent(STOP_EVENT.expect("Stop event was not created!")) };
}

unsafe extern "system" fn service_ctrl_handler(
    ctrl_code: u32,
    event_type: u32,
    event_data: *mut c_void,
    _context: *mut c_void,
) -> u32 {
    match ctrl_code {
        SERVICE_CONTROL_POWEREVENT => handle_power_event(event_type, event_data),
        SERVICE_CONTROL_STOP => handle_stop(),
        _ => (),
    }
    NO_ERROR.0
}

unsafe extern "system" fn service_main(_arg_num: u32, _args: *mut PWSTR) {
    output_debug("Starting AutoPower...").ok();

    STATUS_HANDLE = Some(
        RegisterServiceCtrlHandlerExW(to_cw_str(SERVICE_NAME), Some(service_ctrl_handler), None)
            .expect("Could not register service control handler!"),
    );

    NOTIFICATION_PROVIDER =
        Some(NotificationProvider::create().expect("Could not create notification provider!"));

    set_service_status(SERVICE_START_PENDING, None, None).unwrap();

    RegisterPowerSettingNotification(HANDLE(STATUS_HANDLE.unwrap().0), &GUID_ACDC_POWER_SOURCE, 1)
        .unwrap();

    STOP_EVENT = Some(match CreateEventW(None, TRUE, FALSE, None) {
        Ok(x) => x,
        Err(err) => {
            set_service_status(SERVICE_STOPPED, None, None).unwrap();
            panic!("Could create stop event!\n{}", err);
        }
    });

    set_service_status(
        SERVICE_RUNNING,
        None,
        Some(SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_POWEREVENT),
    )
    .unwrap();

    // Wait for exit.
    WaitForSingleObject(STOP_EVENT.unwrap(), INFINITE);
    CloseHandle(STOP_EVENT.unwrap());
    set_service_status(SERVICE_STOPPED, Some(3), None).expect("Could not set service status.");
}

fn service_setup() -> Result<()> {
    let service_entry = SERVICE_TABLE_ENTRYW {
        lpServiceName: to_w_str(SERVICE_NAME),
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
