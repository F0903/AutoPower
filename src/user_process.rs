use autopower_shared::{
    logging::Logger,
    stream::{HandleStream, Write},
    util::get_last_win32_err,
    winstr::to_win32_wstr,
    PIPE_BUFFER_SIZE,
};
use windows::{
    core::PWSTR,
    Win32::{
        Foundation::{
            CloseHandle, SetHandleInformation, HANDLE, HANDLE_FLAGS, HANDLE_FLAG_INHERIT,
        },
        Security::SECURITY_ATTRIBUTES,
        System::{
            Environment::{CreateEnvironmentBlock, DestroyEnvironmentBlock},
            Pipes::CreatePipe,
            RemoteDesktop::{
                WTSActive, WTSEnumerateSessionsW, WTSFreeMemory, WTSQueryUserToken,
                WTS_CURRENT_SERVER, WTS_SESSION_INFOW,
            },
            Threading::{
                CreateProcessAsUserW, GetProcessId, OpenProcess, TerminateProcess,
                CREATE_UNICODE_ENVIRONMENT, NORMAL_PRIORITY_CLASS, PROCESS_INFORMATION,
                PROCESS_QUERY_INFORMATION, PROCESS_READ_CONTROL, PROCESS_TERMINATE,
                STARTF_USESTDHANDLES, STARTUPINFOW,
            },
        },
    },
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const LOGGER: Logger = Logger::new("user_process", "autopower");

pub struct UserProcess {
    proc: PROCESS_INFORMATION,
    write_pipe: HandleStream<Write>,
}

impl UserProcess {
    fn get_current_session_id() -> u32 {
        LOGGER.debug_log("Getting session id...");
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
            LOGGER.debug_log("Session result was 0!");
            panic!();
        }

        let mut session_id = 0;
        for i in 0..session_count {
            let info = unsafe { *(session_info.add(i as usize)) };
            LOGGER.debug_log(format!(
                "Found session: {} | {:?} | {}",
                info.SessionId,
                info.State,
                unsafe { info.pWinStationName.to_string().unwrap() }
            ));
            if info.State != WTSActive {
                continue;
            }
            session_id = info.SessionId;
            break;
        }

        unsafe {
            WTSFreeMemory(session_info as *mut std::ffi::c_void);
        }
        return session_id;
    }

    fn get_user_info() -> Result<(HANDLE, *mut std::ffi::c_void)> {
        let mut session_id = Self::get_current_session_id();
        if session_id == 0 {
            loop {
                LOGGER.debug_log("Could not get session id... Waiting and trying again...");
                std::thread::sleep(std::time::Duration::from_millis(1000));
                let id = Self::get_current_session_id();
                if id == 0 {
                    continue;
                }
                session_id = id;
                break;
            }
        }

        let mut token_handle = HANDLE::default();
        let result = unsafe { WTSQueryUserToken(session_id, &mut token_handle) };
        if !result.as_bool() {
            LOGGER.debug_log("Was not able to query user token!");
            return Err("Was not able to query user token!".into());
        }

        LOGGER.debug_log("Creating environment block...");
        let mut environment = std::ptr::null_mut();
        let result = unsafe { CreateEnvironmentBlock(&mut environment, token_handle, true) };
        if !result.as_bool() {
            unsafe {
                CloseHandle(token_handle);
            }
            return Err("Was not able to create environment block!".into());
        }

        Ok((token_handle, environment))
    }

    fn create_pipes() -> Result<(HANDLE, HANDLE)> {
        let security = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            bInheritHandle: true.into(),
            lpSecurityDescriptor: std::ptr::null_mut(),
        };

        let mut read_pipe = HANDLE::default();
        let mut write_pipe = HANDLE::default();
        unsafe {
            let result = CreatePipe(
                &mut read_pipe as *mut HANDLE,
                &mut write_pipe as *mut HANDLE,
                Some(&security),
                PIPE_BUFFER_SIZE,
            );
            if !result.as_bool() {
                return Err("Could not create pipe!".into());
            }

            let result = SetHandleInformation(write_pipe, HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0));
            if !result.as_bool() {
                return Err("Could not set handle info!".into());
            }

            Ok((read_pipe, write_pipe))
        }
    }

    pub fn get_writer(&self) -> &HandleStream<Write> {
        &self.write_pipe
    }

    pub fn create(path: impl AsRef<str>) -> Result<Self> {
        let (token_handle, environment) = Self::get_user_info()?;
        let (read_pipe, write_pipe) = Self::create_pipes()?;
        let start_info: STARTUPINFOW = STARTUPINFOW {
            cb: std::mem::size_of::<STARTUPINFOW>() as u32,
            lpDesktop: to_win32_wstr("winsta0\\default").get_mut(),
            hStdInput: read_pipe,
            dwFlags: STARTF_USESTDHANDLES,
            ..Default::default()
        };

        LOGGER.debug_log("Creating user process...");
        let win32_service_dir = to_win32_wstr(path.as_ref());
        LOGGER.debug_log(format!("{}", unsafe {
            win32_service_dir.get_const().to_string().unwrap()
        }));
        let mut proc_info = PROCESS_INFORMATION::default();
        let result = unsafe {
            CreateProcessAsUserW(
                token_handle,
                win32_service_dir.get_const(),
                PWSTR::null(),
                None,
                None,
                true,
                NORMAL_PRIORITY_CLASS | CREATE_UNICODE_ENVIRONMENT,
                Some(environment),
                None,
                &start_info,
                &mut proc_info,
            )
        };

        unsafe {
            let result = DestroyEnvironmentBlock(environment);
            if !result.as_bool() {
                let err_msg = get_last_win32_err()?;
                let msg = format!("Could not destroy environment block!\n{}", &err_msg);
                LOGGER.debug_log(&msg);
                return Err(msg.into());
            };

            let result = CloseHandle(token_handle);
            if !result.as_bool() {
                let err_msg = get_last_win32_err()?;
                let msg = format!("Could not close token handle!\n{}", &err_msg);
                LOGGER.debug_log(&msg);
                return Err(msg.into());
            };
        }

        if !result.as_bool() {
            let err_msg = get_last_win32_err()?;
            let msg = format!("Could not create user process!\n{}", &err_msg);
            LOGGER.debug_log(&msg);
            return Err(msg.into());
        };
        LOGGER.debug_log("Created user process...");

        Ok(Self {
            proc: proc_info,
            write_pipe: HandleStream::create(write_pipe),
        })
    }

    pub fn terminate(&self) {
        LOGGER.debug_log("Terminating user process...");
        unsafe {
            let start_handle = self.proc.hProcess;
            let handle = OpenProcess(
                PROCESS_READ_CONTROL | PROCESS_QUERY_INFORMATION | PROCESS_TERMINATE,
                false,
                GetProcessId(start_handle),
            )
            .unwrap();
            if !TerminateProcess(handle, 0).as_bool() {
                let msg = get_last_win32_err().unwrap();
                LOGGER.debug_log(&format!("Could not terminate user process!\n{}", msg));
            }
            TerminateProcess(handle, 0);

            CloseHandle(start_handle);
            CloseHandle(handle);
        }
    }
}

impl Drop for UserProcess {
    fn drop(&mut self) {
        LOGGER.debug_log("Dropping user process...");
        self.terminate();
    }
}
