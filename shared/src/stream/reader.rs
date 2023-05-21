use super::{HandleStream, HandleStreamMode};
use crate::{pipe::PIPE_BUFFER_SIZE, util::get_last_win32_err};
use windows::Win32::{
    Foundation::GENERIC_READ,
    Storage::FileSystem::{ReadFile, PIPE_ACCESS_INBOUND},
};

pub struct Read;
impl HandleStreamMode for Read {
    fn as_generic_access_rights() -> u32 {
        GENERIC_READ.0
    }

    fn as_pipe_access_rights() -> windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES {
        PIPE_ACCESS_INBOUND
    }
}

impl HandleStream<Read> {
    pub fn read_string(&self) -> super::Result<String> {
        let mut buf = [0; PIPE_BUFFER_SIZE as usize];
        let mut bytes_read = 0;
        let result = unsafe {
            ReadFile(
                self.handle,
                Some(buf.as_mut_ptr() as *mut std::ffi::c_void),
                buf.len() as u32,
                Some(&mut bytes_read),
                None,
            )
        };
        if !result.as_bool() {
            let err = get_last_win32_err()?;
            return Err(format!("Could not read from pipe!\n{}", err).into());
        }
        Ok(std::str::from_utf8(&buf[..bytes_read as usize])
            .map_err(|_| "Could not convert buffer to &str")?
            .to_owned())
    }
}
