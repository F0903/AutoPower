use super::WindowsService;
use crate::{
    config::PowerConfig, handler_data::HandlerData, notification_provider::NotificationProvider,
    power_scheme::set_power_scheme,
};
use autopower_shared::{logging::Logger, winstr::to_win32_wstr};
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
                RegisterServiceCtrlHandlerExW, SetServiceStatus, SERVICE_ACCEPT_POWEREVENT,
                SERVICE_ACCEPT_STOP, SERVICE_CONTROL_POWEREVENT, SERVICE_CONTROL_STOP,
                SERVICE_RUNNING, SERVICE_START_PENDING, SERVICE_STATUS,
                SERVICE_STATUS_CURRENT_STATE, SERVICE_STATUS_HANDLE, SERVICE_STOPPED,
                SERVICE_STOP_PENDING, SERVICE_WIN32_OWN_PROCESS,
            },
            SystemServices::GUID_ACDC_POWER_SOURCE,
            Threading::{CreateEventW, SetEvent, WaitForSingleObject, INFINITE},
        },
        UI::WindowsAndMessaging::{self, PBT_POWERSETTINGCHANGE},
    },
};

type Result<T> = super::Result<T>;

const SERVICE_NAME: &str = "AutoPower";
const LOGGER: Logger = Logger::new("power_service", "autopower");

pub struct PowerService {
    current_status: Option<SERVICE_STATUS>,
    status_handle: Option<SERVICE_STATUS_HANDLE>,
    stop_event: Option<HANDLE>,
    notification_provider: Option<NotificationProvider>,
    power_config: PowerConfig,
}

impl PowerService {
    fn new(power_config: PowerConfig) -> Self {
        Self {
            current_status: None,
            status_handle: None,
            stop_event: None,
            notification_provider: None,
            power_config,
        }
    }

    fn set_service_status(
        &mut self,
        set_state: SERVICE_STATUS_CURRENT_STATE,
        checkpoint: Option<u32>,
        controls: Option<u32>,
    ) -> Result<()> {
        let status = SERVICE_STATUS {
            dwServiceType: SERVICE_WIN32_OWN_PROCESS,
            dwCurrentState: set_state,
            dwControlsAccepted: controls.unwrap_or(0),
            dwCheckPoint: checkpoint.unwrap_or(0),
            ..Default::default()
        };
        unsafe {
            SetServiceStatus(
                self.status_handle.ok_or("STATUS_HANDLE was not set!")?,
                &status,
            )
            .unwrap();
            self.current_status = Some(status);
        }
        Ok(())
    }

    fn handle_on_wired_power(&mut self) -> Result<()> {
        let wired_config = self.power_config.get_wired_config();
        set_power_scheme(
            wired_config.get_power_scheme(),
            self.notification_provider.as_mut().unwrap(),
        )?;
        Ok(())
    }

    fn handle_on_battery_power(&mut self) -> Result<()> {
        let battery_config = self.power_config.get_battery_config();
        set_power_scheme(
            battery_config.get_power_scheme(),
            self.notification_provider.as_mut().unwrap(),
        )?;
        Ok(())
    }

    fn handle_power_event(&mut self, data: HandlerData) {
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
            Power::PoAc => self.handle_on_wired_power().unwrap(),
            Power::PoDc => self.handle_on_battery_power().unwrap(),
            _ => (),
        }
    }

    fn handle_stop(&mut self) {
        LOGGER.debug("Received stop event... Stopping...");
        if {
            self.current_status
                .ok_or("Current status was not set when stopping!")
                .unwrap()
        }
        .dwCurrentState
            != SERVICE_RUNNING
        {
            return;
        }

        self.set_service_status(SERVICE_STOP_PENDING, Some(4), None)
            .map_err(|_| "Could not set service status!")
            .unwrap();
        unsafe {
            SetEvent(
                self.stop_event
                    .ok_or("Stop event was not created!")
                    .unwrap(),
            )
            .unwrap()
        };
    }

    unsafe extern "system" fn service_ctrl_handler(
        ctrl_code: u32,
        event_type: u32,
        event_data: *mut c_void,
        _context: *mut c_void,
    ) -> u32 {
        let mut me = _context.cast::<Self>().read();
        let data = HandlerData {
            event_type,
            event_data,
        };
        // Win32 docs say to start new thread for any other work than returning immediately
        std::thread::spawn(move || {
            match ctrl_code {
                SERVICE_CONTROL_POWEREVENT => {
                    me.handle_power_event(data);
                }
                SERVICE_CONTROL_STOP => me.handle_stop(),
                x => {
                    LOGGER.debug(format!("Received unknown control code: {}", x));
                }
            };
        });
        NO_ERROR.0
    }
}

impl WindowsService for PowerService {
    unsafe extern "system" fn service_main(_arg_num: u32, _args: *mut PWSTR) {
        LOGGER.debug("Registering service control handler...");

        let me: &'static mut Self =
            Box::leak(Box::new(Self::new(PowerConfig::get_or_create().unwrap())));
        let service_name = to_win32_wstr(SERVICE_NAME);
        me.status_handle = Some(
            match RegisterServiceCtrlHandlerExW(
                service_name.get_const(),
                Some(Self::service_ctrl_handler),
                Some((me as *mut Self).cast()),
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

        if let Err(e) = me.set_service_status(SERVICE_START_PENDING, None, None) {
            LOGGER.error(format!("Could not set service status!\n{}", e));
        }

        me.notification_provider = Some(match NotificationProvider::create() {
            Ok(x) => x,
            Err(e) => {
                LOGGER.error(format!("Could not create notification provider!\n{}", e));
                panic!();
            }
        });
        LOGGER.debug("Creation of notification provider successful.");

        LOGGER.debug("Registering power setting notification handling...");
        if let Err(e) = RegisterPowerSettingNotification(
            HANDLE(me.status_handle.unwrap().0),
            &GUID_ACDC_POWER_SOURCE,
            WindowsAndMessaging::REGISTER_NOTIFICATION_FLAGS(1),
        ) {
            LOGGER.error(format!(
                "Could not register power settings notification!\n{}",
                e
            ));
        }

        me.stop_event = Some(match CreateEventW(None, TRUE, FALSE, None) {
            Ok(x) => x,
            Err(err) => {
                LOGGER.error(format!("Could not create stop event!\n{}", err));
                panic!();
            }
        });

        if let Err(e) = me.set_service_status(
            SERVICE_RUNNING,
            None,
            Some(SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_POWEREVENT),
        ) {
            LOGGER.error(format!("Could not set service status!\n{}", e));
        }

        // Wait for exit.
        WaitForSingleObject(me.stop_event.unwrap(), INFINITE);
        LOGGER.debug("Stop event signaled. Cleaning up and terminating...");
        CloseHandle(me.stop_event.unwrap()).unwrap();

        if let Err(e) = me.set_service_status(SERVICE_STOPPED, Some(3), None) {
            LOGGER.error(format!("Could not set service status!\n{}", e));
        }
        me.notification_provider
            .as_ref()
            .unwrap()
            .terminate()
            .unwrap();
    }

    fn get_name() -> &'static str {
        &SERVICE_NAME
    }
}

unsafe impl Sync for PowerService {}
unsafe impl Send for PowerService {}
