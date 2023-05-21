use super::{HandleStream, HandleStreamMode};
use crate::util::get_last_win32_err;
use windows::Win32::{
    Foundation::GENERIC_WRITE,
    Storage::FileSystem::{WriteFile, PIPE_ACCESS_OUTBOUND},
};

pub struct Write;
impl HandleStreamMode for Write {
    fn as_generic_access_rights() -> u32 {
        GENERIC_WRITE.0
    }

    fn as_pipe_access_rights() -> windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES {
        PIPE_ACCESS_OUTBOUND
    }
}

impl HandleStream<Write> {
    pub fn write(&self, input: &[u8]) -> super::Result<()> {
        let mut bytes_written = 0;
        let result = unsafe { WriteFile(self.handle, Some(input), Some(&mut bytes_written), None) };
        if !result.as_bool() {
            let err = get_last_win32_err()?;
            return Err(format!("Could not write to ouput pipe!\n{}", err).into());
        }
        Ok(())
    }
}
