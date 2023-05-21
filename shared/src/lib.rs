pub mod logging;
pub mod notification_command;
pub mod pipe;
pub mod stream;
pub mod util;
pub mod winstr;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
