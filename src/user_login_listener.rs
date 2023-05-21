use autopower_shared::{logging::Logger, util::get_last_win32_err, winstr::to_h_string};
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::{
        EventLog::{EvtClose, EvtSubscribe, EvtSubscribeToFutureEvents, EVT_HANDLE},
        Threading::{CreateEventW, WaitForSingleObject, INFINITE},
    },
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const EVENT_SUBSCRIBE_PATH: &str = "System";
const EVENT_SUBSCRIBE_QUERY: &str = "Event/System[EventID=7001]";

const LOGGER: Logger = Logger::new("user_logon_listener", "autopower");

pub struct UserLoginListener {
    wait_event: HANDLE,
    wait_subscription: EVT_HANDLE,
}

impl UserLoginListener {
    pub fn wait_for_login(&self) {
        LOGGER.debug_log("Waiting for user login...");
        let result = unsafe { WaitForSingleObject(self.wait_event, INFINITE) };
        if result.is_err() {
            let err = get_last_win32_err().unwrap();
            LOGGER.debug_log(format!("Could not wait for user login!\n{}", err));
            return;
        }
        LOGGER.debug_log("User has logged in.");
    }

    pub fn new() -> Result<Self> {
        let path = to_h_string(EVENT_SUBSCRIBE_PATH)?;
        let query = to_h_string(EVENT_SUBSCRIBE_QUERY)?;

        println!("{}", path.to_string());
        println!("{}", path.to_string());

        let wait_event = unsafe { CreateEventW(None, false, false, None)? };
        let wait_subscription = unsafe {
            EvtSubscribe(
                None,
                Some(&wait_event),
                &path,
                &query,
                None,
                None,
                None,
                EvtSubscribeToFutureEvents.0,
            )?
        };
        if wait_subscription.is_invalid() {
            let err = get_last_win32_err()?;
            return Err(format!("Could not subscribe to logon event!\n{}", err).into());
        }

        Ok(Self {
            wait_event,
            wait_subscription,
        })
    }
}

impl Drop for UserLoginListener {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.wait_event);
            EvtClose(self.wait_subscription);
        }
    }
}
