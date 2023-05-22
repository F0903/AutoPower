use windows::core::{HSTRING, PCWSTR, PWSTR};

pub struct Win32StrPtr<T> {
    data: Vec<T>,
    len: usize,
}

impl Win32StrPtr<u16> {
    pub fn get_mut(&mut self) -> PWSTR {
        PWSTR(self.data[..self.len].as_mut_ptr())
    }

    pub fn get_const(&self) -> PCWSTR {
        PCWSTR(self.data[..self.len].as_ptr())
    }

    pub fn from_buffer(data: Vec<u16>, length: usize) -> Self {
        Self { data, len: length }
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
    let len = input.len();
    Win32StrPtr::from_buffer(data, len)
}

pub fn to_h_string(input: &str) -> super::Result<HSTRING> {
    let data = get_nullterminated_utf16_from_utf8(input);
    let str = HSTRING::from_wide(&data)?;
    Ok(str)
}
