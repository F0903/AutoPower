pub mod reader;
pub mod writer;

pub use reader::Read;
pub use writer::Write;

use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait HandleStreamMode {
    fn as_generic_access_rights() -> u32;
    fn as_pipe_access_rights() -> FILE_FLAGS_AND_ATTRIBUTES;
}

pub struct HandleStream<M: HandleStreamMode> {
    handle: HANDLE,
    mode: std::marker::PhantomData<M>,
}

impl<M: HandleStreamMode> HandleStream<M> {
    pub fn get_raw_handle(&self) -> HANDLE {
        self.handle
    }

    pub fn create(handle: HANDLE) -> Self {
        Self {
            handle,
            mode: std::marker::PhantomData,
        }
    }

    pub fn close(&self) {
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

impl<M: HandleStreamMode> Drop for HandleStream<M> {
    fn drop(&mut self) {
        self.close()
    }
}
