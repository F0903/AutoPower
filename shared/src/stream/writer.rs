use super::HandleStream;
use crate::util::get_last_win32_err;
use windows::Win32::Storage::FileSystem::WriteFile;

pub struct Write;

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
