use windows::core::{HSTRING, PCWSTR, PWSTR};

pub struct Win32StrPtr<T> {
    data: Vec<T>,
}

impl Win32StrPtr<u16> {
    pub fn get_mut(&mut self) -> PWSTR {
        PWSTR(self.data.as_ptr() as *mut u16)
    }

    pub fn get_const(&self) -> PCWSTR {
        PCWSTR(self.data.as_ptr())
    }

    pub fn from_buffer(data: Vec<u16>) -> Self {
        Self { data: data }
    }
}

fn get_nullterminated_utf16_from_utf8(input: &str) -> Vec<u16> {
    input
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>()
}

pub fn to_win32_wstr(input: &str) -> Win32StrPtr<u16> {
    let data = get_nullterminated_utf16_from_utf8(input);
    Win32StrPtr::from_buffer(data)
}

pub fn to_h_string(input: &str) -> super::Result<HSTRING> {
    let data = get_nullterminated_utf16_from_utf8(input);
    let str = HSTRING::from_wide(&data)?;
    Ok(str)
}
