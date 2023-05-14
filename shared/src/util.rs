use std::io::Write;

use windows::core::{PCWSTR, PWSTR};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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

pub fn to_cw_str(input: &str) -> PCWSTR {
    PCWSTR(input.encode_utf16().collect::<Vec<_>>().as_ptr())
}

pub fn to_w_str(input: &str) -> PWSTR {
    PWSTR(input.encode_utf16().collect::<Vec<_>>().as_mut_ptr())
}
