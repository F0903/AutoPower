use autopower_shared::logging::Logger;
use windows::Win32::UI::WindowsAndMessaging;

pub fn print_power_event_type(event_type: u32, logger: &Logger) {
    match event_type {
        WindowsAndMessaging::PBT_APMOEMEVENT => logger.debug("Power event was PBT_APMOEMEVENT"),
        WindowsAndMessaging::PBT_APMPOWERSTATUSCHANGE => {
            logger.debug("Power event was PBT_APMPOWERSTATUSCHANGE")
        }
        WindowsAndMessaging::PBT_APMQUERYSTANDBY => {
            logger.debug("Power event was PBT_APMPOWERSTATUSCHANGE")
        }
        WindowsAndMessaging::PBT_APMQUERYSTANDBYFAILED => {
            logger.debug("Power event was PBT_APMPOWERSTATUSCHANGE")
        }
        WindowsAndMessaging::PBT_APMQUERYSUSPEND => {
            logger.debug("Power event was PBT_APMPOWERSTATUSCHANGE")
        }
        WindowsAndMessaging::PBT_APMQUERYSUSPENDFAILED => {
            logger.debug("Power event was PBT_APMPOWERSTATUSCHANGE")
        }
        WindowsAndMessaging::PBT_APMRESUMEAUTOMATIC => {
            logger.debug("Power event was PBT_APMPOWERSTATUSCHANGE")
        }
        WindowsAndMessaging::PBT_APMRESUMECRITICAL => {
            logger.debug("Power event was PBT_APMPOWERSTATUSCHANGE")
        }
        WindowsAndMessaging::PBT_APMRESUMESTANDBY => {
            logger.debug("Power event was PBT_APMPOWERSTATUSCHANGE")
        }
        WindowsAndMessaging::PBT_APMRESUMESUSPEND => {
            logger.debug("Power event was PBT_APMPOWERSTATUSCHANGE")
        }

        WindowsAndMessaging::PBT_APMSTANDBY => {
            logger.debug("Power event was PBT_APMPOWERSTATUSCHANGE")
        }
        WindowsAndMessaging::PBT_APMSUSPEND => {
            logger.debug("Power event was PBT_APMPOWERSTATUSCHANGE")
        }
        WindowsAndMessaging::PBT_POWERSETTINGCHANGE => {
            logger.debug("Power event was PBT_POWERSETTINGCHANGE")
        }
        _ => logger.debug("Power event was unknown."),
    }
}
