use std::ffi::c_void;

pub struct HandlerData {
    pub event_type: u32,
    pub event_data: *mut c_void,
}
unsafe impl Send for HandlerData {}
unsafe impl Sync for HandlerData {}
