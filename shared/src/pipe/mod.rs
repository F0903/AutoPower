pub mod client;
pub mod server;

pub use super::stream;
pub use client::Client;
pub use server::Server;

use crate::{
    logging::Logger,
    stream::{HandleStream, HandleStreamMode},
};
use std::{
    fmt::Debug,
    io::{Read, Write},
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub const PIPE_BUFFER_SIZE: usize = 1024;
pub const PIPE_PATH_ROOT: &str = "\\\\.\\pipe\\";
pub const PIPE_NAME: &str = "AutoPowerProxy";

static LOGGER: Logger = Logger::new("pipe", "autopower_shared");

pub struct Pipe<M, S: HandleStreamMode> {
    stream: HandleStream<S>,
    mode: std::marker::PhantomData<M>,
}

impl<M> Pipe<M, stream::Read> {
    pub fn read_to<T: serde::de::DeserializeOwned + Debug>(&mut self) -> Result<T> {
        let mut buf = [0; PIPE_BUFFER_SIZE];
        let count = self.read(&mut buf)?;
        LOGGER.debug(format!("Got {} bytes. Deserializing...", count));
        let obj = bincode::deserialize(&mut buf[..count])?;
        LOGGER.debug(format!("Deserialized to {:?}", obj));
        Ok(obj)
    }
}

impl<M> std::io::Read for Pipe<M, stream::Read> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(buf)
    }
}

impl<M> Pipe<M, stream::Write> {
    pub fn write_as(&mut self, obj: impl serde::Serialize) -> Result<()> {
        let bytes = bincode::serialize(&obj)?;
        self.write_all(&bytes)?;
        Ok(())
    }
}

impl<M> std::io::Write for Pipe<M, stream::Write> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        LOGGER.debug("Writing to pipe...");
        self.stream.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<M, S: HandleStreamMode> Pipe<M, S> {
    pub fn get_stream(&self) -> &HandleStream<S> {
        &self.stream
    }

    pub fn close(&self) -> Result<()> {
        LOGGER.debug("Closing pipe...");
        self.stream.close()
    }
}

impl<M, S: HandleStreamMode> Drop for Pipe<M, S> {
    fn drop(&mut self) {
        LOGGER.debug("Dropping pipe...");
        self.close().unwrap();
    }
}
