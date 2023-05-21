use super::{Pipe, Result, PIPE_BUFFER_SIZE, PIPE_PATH_ROOT};
use crate::{
    stream::{HandleStream, HandleStreamMode},
    util::get_last_win32_err,
    winstr::to_win32_wstr,
};
use windows::Win32::{
    Storage::FileSystem::FILE_FLAG_FIRST_PIPE_INSTANCE,
    System::Pipes::{ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_MESSAGE, PIPE_TYPE_MESSAGE},
};

pub struct Server;

impl<S: HandleStreamMode> Pipe<Server, S> {
    pub fn create_server(name: &str) -> Result<Self> {
        let pipe_name = to_win32_wstr(&format!("{}{}", PIPE_PATH_ROOT, name));
        let pipe = unsafe {
            CreateNamedPipeW(
                pipe_name.get_const(),
                S::as_pipe_access_rights() | FILE_FLAG_FIRST_PIPE_INSTANCE,
                PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE,
                1,
                PIPE_BUFFER_SIZE,
                PIPE_BUFFER_SIZE,
                0,
                None,
            )
        };
        if pipe.is_invalid() {
            let err = get_last_win32_err()?;
            return Err(format!("Could not create named pipe!\n{}", err).into());
        }

        Ok(Self {
            stream: HandleStream::create(pipe),
            mode: std::marker::PhantomData,
        })
    }

    pub fn connect(&self) -> Result<()> {
        let result = unsafe { ConnectNamedPipe(self.stream.get_raw_handle(), None) };
        if !result.as_bool() {
            let err = get_last_win32_err()?;
            return Err(format!("Could not connect named pipe!\n{}", err).into());
        }
        Ok(())
    }
}
