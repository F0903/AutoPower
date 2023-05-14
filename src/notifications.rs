use std::ffi::c_void;

use autopower_shared::{notifications::NotificationCommand, util::to_win32_wstr};
use windows::{
    core::PWSTR,
    Win32::{
        Foundation::{CloseHandle, GetLastError, HANDLE},
        Storage::FileSystem::WriteFile,
        System::{
            Diagnostics::Debug::{FormatMessageW, FORMAT_MESSAGE_FROM_SYSTEM},
            Environment::{CreateEnvironmentBlock, DestroyEnvironmentBlock},
            Pipes::CreatePipe,
            RemoteDesktop::{
                WTSActive, WTSEnumerateSessionsW, WTSFreeMemory, WTSQueryUserToken,
                WTS_CURRENT_SERVER, WTS_SESSION_INFOW,
            },
            Services::{
                OpenSCManagerW, OpenServiceW, QueryServiceConfigW, QUERY_SERVICE_CONFIGW,
                SC_MANAGER_CONNECT, SERVICE_QUERY_CONFIG,
            },
            Threading::{
                CreateProcessAsUserW, CREATE_UNICODE_ENVIRONMENT, NORMAL_PRIORITY_CLASS,
                PROCESS_INFORMATION, STARTF_USESTDHANDLES, STARTUPINFOW,
            },
        },
    },
};

use autopower_shared::util::output_debug;

use crate::SERVICE_NAME;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const NOTIFICATION_PROVIDER_NAME: &str = "autopower_notification_provider.exe";
const PIPE_BUFFER_SIZE: u32 = 1024;

pub fn get_service_dir() -> Result<String> {
    let sc = unsafe { OpenSCManagerW(None, None, SC_MANAGER_CONNECT)? };
    let service_name = to_win32_wstr(SERVICE_NAME);
    let service_handle =
        unsafe { OpenServiceW(sc, service_name.get_const(), SERVICE_QUERY_CONFIG)? };

    let mut size = std::mem::size_of::<QUERY_SERVICE_CONFIGW>() as u32;
    let mut buf: Vec<u8> = Vec::with_capacity(size as usize);
    let config = unsafe {
        // Loop so we can try again if the buffer is too small.
        loop {
            buf.resize(size as usize, 0);
            let config = buf.as_ptr() as *mut QUERY_SERVICE_CONFIGW;
            let result = QueryServiceConfigW(service_handle, Some(config), size, &mut size);
            if result.as_bool() {
                break *config;
            }
        }
    };

    let path = unsafe { config.lpBinaryPathName.to_string()? };
    let dir_path = path.rsplit_once('\\').unwrap().0;
    Ok(dir_path.to_owned())
}

pub struct NotificationProvider {
    proc_info: PROCESS_INFORMATION,
    input_pipe_read: HANDLE,
    input_pipe_write: HANDLE,
}

impl NotificationProvider {
    fn get_current_session_id() -> u32 {
        let mut session_info: *mut WTS_SESSION_INFOW = std::ptr::null_mut();
        let mut session_count: u32 = 0;
        let result = unsafe {
            WTSEnumerateSessionsW(
                WTS_CURRENT_SERVER,
                0,
                1,
                &mut session_info,
                &mut session_count,
            )
        };
        if !result.as_bool() {
            output_debug("Session result was 0!").ok();
            panic!();
        }

        let mut session_id = 0;
        for i in 0..session_count {
            let info = unsafe { *(session_info.add(i as usize)) };
            if info.State != WTSActive {
                continue;
            }
            session_id = info.SessionId;
            break;
        }

        unsafe {
            WTSFreeMemory(session_info as *mut c_void);
        }
        return session_id;
    }

    pub fn create() -> Result<Self> {
        let session_id = Self::get_current_session_id();
        if session_id == 0 {
            output_debug("Was not able to get session id!").ok();
            return Err("Was not able to get session id!".into());
        }

        let mut token_handle = HANDLE::default();
        let result = unsafe { WTSQueryUserToken(session_id, &mut token_handle) };
        if !result.as_bool() {
            output_debug("Was not able to query user token!").ok();
            return Err("Was not able to query user token!".into());
        }

        output_debug("Creating environment block...").ok();
        let mut environment = std::ptr::null_mut();
        let result = unsafe { CreateEnvironmentBlock(&mut environment, token_handle, true) };
        if !result.as_bool() {
            unsafe {
                CloseHandle(token_handle);
            }
            return Err("Was not able to create environment block!".into());
        }

        output_debug("Creating pipes...").ok();
        let mut read_pipe = HANDLE::default();
        let mut write_pipe = HANDLE::default();
        let result = unsafe { CreatePipe(&mut read_pipe, &mut write_pipe, None, PIPE_BUFFER_SIZE) };
        if !result.as_bool() {
            output_debug("Could not create process pipes!").ok();
            return Err("Could not create process pipes!".into());
        }

        output_debug("Creating notification provider...").ok();
        let start_info = STARTUPINFOW {
            cb: std::mem::size_of::<STARTUPINFOW>() as u32,
            lpDesktop: to_win32_wstr("winsta0\\default").get(),
            dwFlags: STARTF_USESTDHANDLES,
            hStdInput: read_pipe,
            ..Default::default()
        };

        output_debug("Creating notification process...").ok();
        let mut service_dir = get_service_dir()?;
        service_dir.push('\\');
        service_dir.push_str(NOTIFICATION_PROVIDER_NAME);
        let win32_service_dir = to_win32_wstr(&service_dir);
        output_debug(&format!("{}", service_dir)).ok();
        let mut proc_info = PROCESS_INFORMATION::default();
        let result = unsafe {
            CreateProcessAsUserW(
                token_handle,
                win32_service_dir.get_const(),
                PWSTR::null(),
                None,
                None,
                false,
                NORMAL_PRIORITY_CLASS | CREATE_UNICODE_ENVIRONMENT,
                Some(environment),
                None,
                &start_info,
                &mut proc_info,
            )
        };

        output_debug("Created notificaion process... Cleaning up...").ok();
        unsafe {
            DestroyEnvironmentBlock(environment);
            CloseHandle(token_handle);
        }
        output_debug("Cleanup done...").ok();

        if !result.as_bool() {
            output_debug("State NB").ok();
            let err = unsafe { GetLastError() };
            const BUF_SIZE: usize = 128;
            let buf: PWSTR = PWSTR::from_raw([0; BUF_SIZE + 1].as_mut_ptr());
            let count = unsafe {
                FormatMessageW(
                    FORMAT_MESSAGE_FROM_SYSTEM,
                    None,
                    err.0,
                    0,
                    buf,
                    BUF_SIZE as u32,
                    None,
                )
            };
            let str = unsafe {
                match buf.to_string() {
                    Ok(x) => x,
                    Err(_) => {
                        output_debug("Could not convert PWSTR to String!").ok();
                        panic!()
                    }
                }
            };
            let msg = format!(
                "Could not create notification provider!\n{}",
                &str[..count as usize]
            );
            output_debug(&msg).ok();
            return Err(msg.into());
        }

        output_debug("Created notification provider.").ok();
        Ok(NotificationProvider {
            proc_info,
            input_pipe_read: read_pipe,
            input_pipe_write: write_pipe,
        })
    }

    pub fn send_display_command(&self, title: &str, description: &str) -> Result<()> {
        let command = NotificationCommand {
            name: "display".to_owned(),
            content: format!("{}\n{}", title, description),
        };
        let command_str = serde_json::to_string(&command)?;
        let mut bytes_written = 0;
        let result = unsafe {
            WriteFile(
                self.input_pipe_write,
                Some(command_str.as_bytes()),
                Some(&mut bytes_written),
                None,
            )
        };
        if !result.as_bool() {
            output_debug("Could not write to output pipe!").ok();
            return Err("Could not write to ouput pipe!".into());
        }
        Ok(())
    }
}

impl Drop for NotificationProvider {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.proc_info.hThread);
            CloseHandle(self.proc_info.hProcess);
            CloseHandle(self.input_pipe_read);
            CloseHandle(self.input_pipe_write);
        }
    }
}
