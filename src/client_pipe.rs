use autopower_shared::{notifications::PIPE_PATH_ROOT, util::to_win32_wstr};
use windows::Win32::{
    Foundation::{CloseHandle, GENERIC_WRITE, HANDLE},
    Storage::FileSystem::{
        CreateFileW, WriteFile, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_NONE, OPEN_EXISTING,
    },
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const PIPE_CONNECT_ATTEMPTS: usize = 10;

pub struct Pipe {
    handle: HANDLE,
}

impl Pipe {
    pub fn create(name: &str) -> Result<Self> {
        let pipe_name = to_win32_wstr(&format!("{}{}", PIPE_PATH_ROOT, name));
        let mut pipe = None;
        unsafe {
            for _ in 0..PIPE_CONNECT_ATTEMPTS {
                pipe = Some(
                    CreateFileW(
                        pipe_name.get_const(),
                        GENERIC_WRITE.0,
                        FILE_SHARE_NONE,
                        None,
                        OPEN_EXISTING,
                        FILE_ATTRIBUTE_NORMAL,
                        None,
                    )
                    .map_err(|e| format!("Could not open write pipe!\n{}", e)),
                );

                if let Some(Ok(x)) = pipe {
                    pipe = Some(Ok(x));
                    break;
                }

                // If we can't connect, it might be due to the other process not having created the pipe yet.
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        };

        let pipe = match pipe {
            Some(Ok(x)) => x,
            Some(Err(e)) => {
                return Err(format!(
                    "Could not connect to pipe after {} attempts!\n{}",
                    PIPE_CONNECT_ATTEMPTS, e
                )
                .into())
            }
            None => unreachable!(),
        };

        Ok(Self { handle: pipe })
    }

    pub fn write(&self, input: &[u8]) -> Result<()> {
        let mut bytes_written = 0;
        let result = unsafe { WriteFile(self.handle, Some(input), Some(&mut bytes_written), None) };
        if !result.as_bool() {
            return Err("Could not write to ouput pipe!".into());
        }
        Ok(())
    }

    pub fn close(&self) {
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

impl Drop for Pipe {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle);
        }
    }
}
