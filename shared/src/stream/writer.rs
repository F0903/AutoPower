use super::{HandleStream, FileStream};
use windows::Win32::{
    Foundation::GENERIC_WRITE,
    Storage::FileSystem::{WriteFile, PIPE_ACCESS_OUTBOUND},
};

pub struct Write;
impl FileStream for Write {
    fn as_generic_access_rights() -> u32 {
        GENERIC_WRITE.0
    }

    fn as_pipe_access_rights() -> windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES {
        PIPE_ACCESS_OUTBOUND
    }
}

impl HandleStream<Write> {}

impl std::io::Write for HandleStream<Write> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut bytes_written = 0;
        unsafe { WriteFile(self.handle, Some(buf), Some(&mut bytes_written), None)? };
        Ok(bytes_written as usize)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
