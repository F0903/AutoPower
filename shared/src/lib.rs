pub mod logging;
pub mod notifications;
pub mod util;
pub mod winstr;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
