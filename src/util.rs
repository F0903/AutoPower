use autopower_shared::util::to_win32_wstr;
use windows::Win32::System::Services::{
    OpenSCManagerW, OpenServiceW, QueryServiceConfigW, QUERY_SERVICE_CONFIGW, SC_MANAGER_CONNECT,
    SERVICE_QUERY_CONFIG,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn get_service_dir() -> Result<String> {
    let sc = unsafe { OpenSCManagerW(None, None, SC_MANAGER_CONNECT)? };
    let service_name = to_win32_wstr(crate::SERVICE_NAME);
    let service_handle =
        unsafe { OpenServiceW(sc, service_name.get_const(), SERVICE_QUERY_CONFIG)? };

    let mut size = std::mem::size_of::<QUERY_SERVICE_CONFIGW>() as u32;
    let mut buf: Vec<u8> = Vec::with_capacity(size as usize);
    let config = unsafe {
        // Loop so we can try again if the buffer is too small.
        loop {
            buf.resize(size as usize, 0);
            let config = buf.as_ptr() as *mut QUERY_SERVICE_CONFIGW;
            let result = QueryServiceConfigW(service_handle, Some(config), size, &mut size);
            if result.as_bool() {
                break *config;
            }
        }
    };

    let path = unsafe { config.lpBinaryPathName.to_string()? };
    let dir_path = path.rsplit_once('\\').unwrap().0;
    Ok(dir_path.to_owned())
}
