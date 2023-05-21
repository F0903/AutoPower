use windows::Win32::System::RemoteDesktop::{
    WTSActive, WTSEnumerateSessionsW, WTSFreeMemory, WTS_CURRENT_SERVER, WTS_SESSION_INFOW,
};

pub fn get_current_session_id() -> super::Result<u32> {
    let mut session_info: *mut WTS_SESSION_INFOW = std::ptr::null_mut();
    let mut session_count: u32 = 0;
    let result = unsafe {
        WTSEnumerateSessionsW(
            WTS_CURRENT_SERVER,
            0,
            1,
            &mut session_info,
            &mut session_count,
        )
    };
    if !result.as_bool() {
        return Err("Session result was 0!".into());
    }

    let mut session_id = 0;
    for i in 0..session_count {
        let info: WTS_SESSION_INFOW = unsafe { *(session_info.add(i as usize)) };
        if info.State != WTSActive {
            continue;
        }
        session_id = info.SessionId;
        break;
    }

    unsafe {
        WTSFreeMemory(session_info as *mut std::ffi::c_void);
    }
    return Ok(session_id);
}
