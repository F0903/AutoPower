pub mod reader;
pub mod writer;

pub use reader::Read;
pub use writer::Write;

use windows::Win32::Foundation::{CloseHandle, HANDLE};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct HandleStream<M> {
    handle: HANDLE,
    mode: std::marker::PhantomData<M>,
}

impl<M> HandleStream<M> {
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

impl<M> Drop for HandleStream<M> {
    fn drop(&mut self) {
        self.close()
    }
}
