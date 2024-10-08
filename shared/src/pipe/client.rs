use super::{Pipe, Result, LOGGER, PIPE_PATH_ROOT};
use crate::{
    stream::{FileStreamMode, FileStream},
    util::get_last_win32_err,
    winstr::Win32String,
};
use windows::Win32::{
    Storage::FileSystem::{
        CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE,
        FILE_WRITE_ATTRIBUTES, OPEN_EXISTING,
    },
    System::Pipes::{SetNamedPipeHandleState, PIPE_READMODE_MESSAGE},
};

const RETRYING_DELAY: u32 = 1000;
const RETRYING_ATTEMPTS: u32 = 15;

pub struct Client;

impl<S: FileStreamMode> Pipe<Client, S> {
    pub fn create_client_retrying(name: &str) -> Result<Self> {
        let mut first_error = None;
        for _ in 0..RETRYING_ATTEMPTS {
            match Self::create_client(name) {
                Ok(x) => return Ok(x),
                Err(e) => {
                    LOGGER.error(format!("Got connection error:\n{}", e));
                    if let None = first_error {
                        first_error = Some(e);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(RETRYING_DELAY as u64));
                }
            }
        }
        Err(format!(
            "Could not connect client after several attempts...\n{}",
            first_error.unwrap_or_else(|| "No error set...".into())
        )
        .into())
    }

    pub fn create_client(name: &str) -> Result<Self> {
        let pipe_name = Win32String::from_str(&format!("{}{}", PIPE_PATH_ROOT, name));
        let access_rights = S::as_generic_access_rights();
        LOGGER.debug(format!(
            "Got following access rights for client pipe: {}",
            access_rights
        ));
        let pipe = unsafe {
            CreateFileW(
                pipe_name.get_const(),
                access_rights | FILE_WRITE_ATTRIBUTES.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                None,
            )?
        };
        if pipe.is_invalid() {
            let err = get_last_win32_err()?;
            return Err(format!("Could not open pipe!\n{}", err).into());
        }

        unsafe { SetNamedPipeHandleState(pipe, Some(&PIPE_READMODE_MESSAGE), None, None)? };

        Ok(Self {
            stream: FileStream::create(pipe),
            mode: std::marker::PhantomData,
        })
    }
}
