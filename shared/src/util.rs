use windows::{
    core::PWSTR,
    Win32::{
        Foundation::GetLastError,
        System::Diagnostics::Debug::{FormatMessageW, FORMAT_MESSAGE_FROM_SYSTEM},
    },
};

pub fn get_last_win32_err() -> super::Result<String> {
    let err = unsafe { GetLastError().unwrap_err() };
    const BUF_SIZE: usize = 128;
    let buf: PWSTR = PWSTR::from_raw([0; BUF_SIZE + 1].as_mut_ptr());
    let count = unsafe {
        FormatMessageW(
            FORMAT_MESSAGE_FROM_SYSTEM,
            None,
            err.code().0 as u32,
            0,
            buf,
            BUF_SIZE as u32,
            None,
        )
    };
    let str = unsafe { buf.to_string()? };
    return Ok(str[..count as usize].to_owned());
}
