pub mod client;
pub mod server;

pub use super::stream::{Read, Write};
pub use client::Client;
pub use server::Server;

use crate::{
    logging::Logger,
    stream::{HandleStream, HandleStreamMode},
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub const PIPE_BUFFER_SIZE: u32 = 512;
pub const PIPE_PATH_ROOT: &str = "\\\\.\\pipe\\";
pub const PIPE_NAME: &str = "AutoPowerNotificationPipe";

const LOGGER: Logger = Logger::new("pipe", "autopower_shared");

pub struct Pipe<M, S: HandleStreamMode> {
    stream: HandleStream<S>,
    mode: std::marker::PhantomData<M>,
}

impl<M> Pipe<M, Read> {
    pub fn read(&self) -> Result<String> {
        self.stream.read_string()
    }
}

impl<M> Pipe<M, Write> {
    pub fn write(&self, input: &[u8]) -> Result<()> {
        self.stream.write(input)
    }
}

impl<M, S: HandleStreamMode> Pipe<M, S> {
    pub fn get_stream(&self) -> &HandleStream<S> {
        &self.stream
    }

    pub fn close(&self) {
        LOGGER.debug("Closing pipe...");
        self.stream.close();
    }
}

impl<M, S: HandleStreamMode> Drop for Pipe<M, S> {
    fn drop(&mut self) {
        LOGGER.debug("Dropping pipe...");
        self.close();
    }
}
