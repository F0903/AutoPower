use autopower_shared::winstr::Win32String;
use windows::{
    core::{w, Free, PWSTR},
    Win32::{
        Foundation::HANDLE,
        System::{
            RemoteDesktop::{WTSGetActiveConsoleSessionId, WTSQueryUserToken},
            Threading::{
                CreateProcessAsUserW, PROCESS_CREATION_FLAGS, PROCESS_INFORMATION, STARTUPINFOW,
            },
        },
    },
};

type Result<T> = crate::Result<T>;

pub struct UserProcess {
    token_handle: HANDLE,
    process_info: PROCESS_INFORMATION,
}

impl UserProcess {
    pub fn new(exe_path: impl Into<Win32String<u16>>) -> Result<Self> {
        let mut token_handle = HANDLE::default();
        unsafe {
            WTSQueryUserToken(WTSGetActiveConsoleSessionId(), &mut token_handle)?;
            let path_str: Win32String<u16> = exe_path.into();
            let mut desktop = Win32String::from_str(r"winsta0\default");
            let startup_info = STARTUPINFOW {
                cb: size_of::<STARTUPINFOW>() as u32,
                dwFlags: windows::Win32::System::Threading::STARTUPINFOW_FLAGS(0),
                lpDesktop: desktop.get_mut(),
                ..Default::default()
            };
            let mut process_info = PROCESS_INFORMATION::default();
            CreateProcessAsUserW(
                token_handle,
                path_str.get_const(),
                PWSTR::null(),
                None,
                None,
                false,
                PROCESS_CREATION_FLAGS(0),
                None,
                None,
                &startup_info,
                &mut process_info,
            )?;
            Ok(Self {
                token_handle,
                process_info,
            })
        }
    }
}

impl Drop for UserProcess {
    fn drop(&mut self) {
        unsafe {
            self.token_handle.free();
            self.process_info.hProcess.free();
            self.process_info.hThread.free();
        }
    }
}
