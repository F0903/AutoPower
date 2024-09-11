use super::{Pipe, Result, PIPE_BUFFER_SIZE, PIPE_PATH_ROOT};
use crate::{
    stream::{FileStreamMode, FileStream},
    util::get_last_win32_err,
    winstr::Win32String,
};
use windows::Win32::{
    Security::{
        InitializeSecurityDescriptor, SetSecurityDescriptorDacl, PSECURITY_DESCRIPTOR,
        SECURITY_ATTRIBUTES, SECURITY_DESCRIPTOR,
    },
    Storage::FileSystem::FILE_FLAG_FIRST_PIPE_INSTANCE,
    System::{
        Pipes::{ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_MESSAGE, PIPE_TYPE_MESSAGE},
        SystemServices::SECURITY_DESCRIPTOR_REVISION,
    },
};

pub struct Server;

impl<S: FileStreamMode> Pipe<Server, S> {
    fn get_security_descriptor() -> Result<SECURITY_DESCRIPTOR> {
        let mut security_desc = SECURITY_DESCRIPTOR::default();
        let p_security_desc =
            PSECURITY_DESCRIPTOR((&mut security_desc as *mut SECURITY_DESCRIPTOR).cast());

        unsafe {
            InitializeSecurityDescriptor(p_security_desc, SECURITY_DESCRIPTOR_REVISION)?;
            SetSecurityDescriptorDacl(p_security_desc, true, None, false)?;
        }

        Ok(security_desc)
    }

    pub fn create_server(name: &str) -> Result<Self> {
        let mut security_desc = Self::get_security_descriptor()?;
        let security = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            bInheritHandle: true.into(),
            lpSecurityDescriptor: (&mut security_desc as *mut SECURITY_DESCRIPTOR).cast(),
        };

        let pipe_name = Win32String::from_str(&format!("{}{}", PIPE_PATH_ROOT, name));
        let pipe = unsafe {
            CreateNamedPipeW(
                pipe_name.get_const(),
                S::as_pipe_access_rights() | FILE_FLAG_FIRST_PIPE_INSTANCE,
                PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE,
                1,
                PIPE_BUFFER_SIZE as u32,
                PIPE_BUFFER_SIZE as u32,
                0,
                Some(&security),
            )
        };
        if pipe.is_invalid() {
            let err = get_last_win32_err()?;
            return Err(format!("Could not create named pipe!\n{}", err).into());
        }

        Ok(Self {
            stream: FileStream::create(pipe),
            mode: std::marker::PhantomData,
        })
    }

    pub fn connect(&self) -> Result<()> {
        unsafe { ConnectNamedPipe(self.stream.get_raw_handle(), None)? };
        Ok(())
    }
}
