pub mod logging;
pub mod pipe;
pub mod proxy_command;
pub mod stream;
pub mod util;
pub mod winstr;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
