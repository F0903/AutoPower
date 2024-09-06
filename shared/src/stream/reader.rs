use crate::logging::Logger;

use super::{HandleStream, HandleStreamMode};
use windows::Win32::{
    Foundation::GENERIC_READ,
    Storage::FileSystem::{ReadFile, PIPE_ACCESS_INBOUND},
};

const LOGGER: Logger = Logger::new("stream_reader", "autopower_shared");

pub struct Read;
impl HandleStreamMode for Read {
    fn as_generic_access_rights() -> u32 {
        GENERIC_READ.0
    }

    fn as_pipe_access_rights() -> windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES {
        PIPE_ACCESS_INBOUND
    }
}

impl HandleStream<Read> {}

impl std::io::Read for HandleStream<Read> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut bytes_read = 0;
        LOGGER.debug("Reading from file handle... (blocking)");
        unsafe { ReadFile(self.handle, Some(buf), Some(&mut bytes_read), None)? };
        Ok(bytes_read as usize)
    }
}
