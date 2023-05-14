use std::ffi::c_void;

use autopower_shared::notifications::NotificationCommand;
use windows::Win32::{
    Foundation::{CloseHandle, FALSE, HANDLE, TRUE},
    Storage::FileSystem::WriteFile,
    System::{
        Environment::{CreateEnvironmentBlock, DestroyEnvironmentBlock},
        Pipes::CreatePipe,
        RemoteDesktop::{
            WTSActive, WTSEnumerateSessionsW, WTSFreeMemory, WTSQueryUserToken, WTS_CURRENT_SERVER,
            WTS_SESSION_INFOW,
        },
        Threading::{
            CreateProcessAsUserW, CREATE_UNICODE_ENVIRONMENT, NORMAL_PRIORITY_CLASS,
            PROCESS_INFORMATION, STARTF_USESTDHANDLES, STARTUPINFOW,
        },
    },
};

use autopower_shared::util::{output_debug, to_cw_str, to_w_str};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const NOTIFICATION_PROVIDER_PATH: &str = "./autopower_notification_provider.exe";
const PIPE_BUFFER_SIZE: u32 = 1024;

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

        let token_handle: *mut HANDLE = std::ptr::null_mut();
        let result = unsafe { WTSQueryUserToken(session_id, token_handle) };
        if !result.as_bool() {
            output_debug("Was not able to query user token!").ok();
            return Err("Was not able to query user token!".into());
        }

        let mut environment = std::ptr::null_mut();
        let result = unsafe { CreateEnvironmentBlock(&mut environment, *token_handle, TRUE) };
        if !result.as_bool() {
            unsafe {
                CloseHandle(*token_handle);
            }
        }

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
            lpDesktop: to_w_str("winsta0\\default"),
            dwFlags: STARTF_USESTDHANDLES,
            hStdInput: read_pipe,
            ..Default::default()
        };
        let mut proc_info = PROCESS_INFORMATION::default();
        let result = unsafe {
            CreateProcessAsUserW(
                *token_handle,
                to_cw_str(NOTIFICATION_PROVIDER_PATH),
                to_w_str(""),
                None,
                None,
                FALSE,
                NORMAL_PRIORITY_CLASS | CREATE_UNICODE_ENVIRONMENT,
                Some(environment),
                None,
                &start_info,
                &mut proc_info,
            )
        };

        unsafe {
            DestroyEnvironmentBlock(environment);
            CloseHandle(*token_handle);
        }

        if !result.as_bool() {
            return Err("Could not create notification provider!".into());
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
