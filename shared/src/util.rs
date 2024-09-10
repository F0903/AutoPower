use std::{ffi::OsStr, path::PathBuf};

use windows::{
    core::PWSTR,
    Win32::{
        Foundation::GetLastError,
        System::{
            Diagnostics::Debug::{FormatMessageW, FORMAT_MESSAGE_FROM_SYSTEM},
            LibraryLoader::GetModuleFileNameA,
        },
    },
};

pub fn get_last_win32_err() -> super::Result<String> {
    let err = unsafe { GetLastError().0 };
    const BUF_SIZE: usize = 128;
    let buf: PWSTR = PWSTR::from_raw([0; BUF_SIZE + 1].as_mut_ptr());
    let count = unsafe {
        FormatMessageW(
            FORMAT_MESSAGE_FROM_SYSTEM,
            None,
            err,
            0,
            buf,
            BUF_SIZE as u32,
            None,
        )
    };
    let str = unsafe { buf.to_string()? };
    return Ok(str[..count as usize].to_owned());
}

pub fn get_process_exe_path() -> super::Result<PathBuf> {
    let mut buf = [0; 512];
    unsafe {
        let count = GetModuleFileNameA(None, &mut buf);
        let count_usize = count as usize;
        if count_usize == buf.len() {
            return Err("Process path is too long!".into());
        }
        if buf[count_usize] != b'\0' {
            return Err(
                "Process path buffer did not end with a null terminator, possible overflow.".into(),
            );
        }
        let os_str = OsStr::from_encoded_bytes_unchecked(&buf[0..(count_usize)]);
        Ok(PathBuf::from(os_str))
    }
}
