use super::HandleStream;
use crate::{util::get_last_win32_err, PIPE_BUFFER_SIZE};
use windows::Win32::Storage::FileSystem::ReadFile;

pub struct Read;

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
        Ok(std::str::from_utf8(&buf[..bytes_read as usize])?.to_owned())
    }
}
