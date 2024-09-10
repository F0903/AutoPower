use std::ffi::c_void;

use autopower_shared::logging::Logger;

//DEBUG REMOVE
static LOGGER: Logger = Logger::new("tmp_debug_handler_data", "autopower");

pub struct HandlerData {
    pub event_type: u32,
    pub event_data: *mut c_void,
}
unsafe impl Send for HandlerData {}
unsafe impl Sync for HandlerData {}

impl Drop for HandlerData {
    fn drop(&mut self) {
        LOGGER.debug("dropping HandlerData...")
    }
}
