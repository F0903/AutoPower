use windows::{
    core::{HSTRING, PCWSTR, PWSTR},
    Win32::{
        Foundation::GetLastError,
        System::Diagnostics::Debug::{FormatMessageW, FORMAT_MESSAGE_FROM_SYSTEM},
    },
};

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

fn get_nullterminated_utf16_from_utf8(input: &str) -> Vec<u16> {
    input
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>()
}

pub fn to_win32_wstr(input: &str) -> Win32StrPtr<u16> {
    let mut data = get_nullterminated_utf16_from_utf8(input);
    let len = input.len();
    Win32StrPtr {
        str_ptr: PWSTR(data[..len].as_mut_ptr()),
        data,
    }
}

pub fn to_h_string(input: &str) -> Result<HSTRING> {
    let data = get_nullterminated_utf16_from_utf8(input);
    let str = HSTRING::from_wide(&data)?;
    Ok(str)
}

pub fn get_last_win32_err() -> Result<String> {
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
    let str = unsafe { buf.to_string()? };
    return Ok(str[..count as usize].to_owned());
}
