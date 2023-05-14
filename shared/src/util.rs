use std::io::Write;

use windows::core::{PCWSTR, PWSTR};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Win32StrPtr<T> {
    #[allow(dead_code)] // Need 'data' to keep the data backing the pointer alive.
    data: Vec<T>,
    str_ptr: PWSTR,
}

impl<T> Win32StrPtr<T> {
    pub fn get(&self) -> PWSTR {
        self.str_ptr
    }

    pub fn get_const(&self) -> PCWSTR {
        PCWSTR(self.str_ptr.0)
    }

    pub fn new(data: Vec<T>, str: PWSTR) -> Self {
        Self { data, str_ptr: str }
    }
}

pub fn output_debug(input: &str) -> Result<()> {
    const DEBUG_PATH: &str = "C:/autorun_debug.txt";
    let mut file = std::fs::File::options()
        .create(true)
        .write(true)
        .append(true)
        .read(true)
        .open(DEBUG_PATH)?;
    let mut buf = Vec::with_capacity(input.len() + 1);
    buf.write_all(input.as_bytes())?;
    buf.write_all(b"\n")?;
    file.write_all(&buf)?;
    Ok(())
}

pub fn to_win32_wstr(input: &str) -> Win32StrPtr<u16> {
    let mut data = input
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let len = input.len();
    Win32StrPtr {
        str_ptr: PWSTR(data[..len].as_mut_ptr()),
        data,
    }
}
