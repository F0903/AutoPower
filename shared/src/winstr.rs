use std::{ffi::OsStr, os::windows::ffi::OsStrExt, path::PathBuf};

use windows::core::{HSTRING, PCWSTR, PWSTR};

fn get_nullterminated_utf16_from_utf8(input: &str) -> Vec<u16> {
    input
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>()
}

pub fn to_h_string(input: &str) -> super::Result<HSTRING> {
    let data = get_nullterminated_utf16_from_utf8(input);
    let str = HSTRING::from_wide(&data)?;
    Ok(str)
}

pub struct Win32String<T> {
    data: Vec<T>,
}

impl Win32String<u16> {
    pub fn from_osstr(input: &OsStr) -> Win32String<u16> {
        let mut buf = Vec::new();
        buf.extend(input.encode_wide());
        Self::from_buffer(buf)
    }

    pub fn from_str(input: &str) -> Win32String<u16> {
        let data = get_nullterminated_utf16_from_utf8(input);
        Self::from_buffer(data)
    }

    pub fn push(&mut self, input: impl Into<Win32String<u16>>) {
        self.data.extend(input.into().data);
    }

    pub fn get_mut(&mut self) -> PWSTR {
        PWSTR(self.data.as_ptr() as *mut u16)
    }

    pub fn get_const(&self) -> PCWSTR {
        PCWSTR(self.data.as_ptr())
    }

    pub fn from_buffer(data: Vec<u16>) -> Self {
        Self { data }
    }
}

impl From<&str> for Win32String<u16> {
    fn from(value: &str) -> Self {
        Self::from_str(value)
    }
}

impl From<&OsStr> for Win32String<u16> {
    fn from(value: &OsStr) -> Self {
        Self::from_osstr(value)
    }
}

impl From<PathBuf> for Win32String<u16> {
    fn from(value: PathBuf) -> Self {
        Self::from_osstr(value.as_os_str())
    }
}
