pub mod reader;
pub mod writer;

pub use reader::Read;
pub use writer::Write;

use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES,
};

use crate::Result;

pub trait FileStreamMode {
    fn as_generic_access_rights() -> u32;
    fn as_pipe_access_rights() -> FILE_FLAGS_AND_ATTRIBUTES;
}

pub struct FileStream<M: FileStreamMode> {
    handle: HANDLE,
    mode: std::marker::PhantomData<M>,
}

impl<M: FileStreamMode> FileStream<M> {
    pub fn get_raw_handle(&self) -> HANDLE {
        self.handle
    }

    pub fn create(handle: HANDLE) -> Self {
        Self {
            handle,
            mode: std::marker::PhantomData,
        }
    }

    pub fn close(&self) -> Result<()> {
        unsafe { CloseHandle(self.handle)? };
        Ok(())
    }
}

impl<M: FileStreamMode> Drop for FileStream<M> {
    fn drop(&mut self) {
        self.close().unwrap();
    }
}
