pub mod logging;
pub mod notifications;
pub mod stream;
pub mod util;
pub mod winstr;

pub const PIPE_BUFFER_SIZE: u32 = 512;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
