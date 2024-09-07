use once_cell::unsync::OnceCell;
use std::{fmt::Display, io::Write, path::PathBuf, str::FromStr};

const TEMP_PATH: &str = std::env!("TEMP");

#[cfg(debug_assertions)]
const LOG_LEVEL: LogLevel = LogLevel::Debug;
#[cfg(not(debug_assertions))]
const LOG_LEVEL: LogLevel = LogLevel::Error;

#[derive(PartialEq, PartialOrd)]
pub enum LogLevel {
    Debug,
    Error,
}

pub struct Logger {
    source_name: &'static str,
    process_name: &'static str,
    log_path: OnceCell<PathBuf>,
}

impl Logger {
    pub const fn new(source_name: &'static str, group_name: &'static str) -> Self {
        Self {
            source_name,
            process_name: group_name,
            log_path: OnceCell::new(),
        }
    }

    pub fn error<A: Display>(&self, input: A) {
        self.log(input, LogLevel::Error);
    }

    #[cfg(debug_assertions)]
    pub fn debug<A: Display>(&self, input: A) {
        self.log(input, LogLevel::Debug);
    }

    #[cfg(not(debug_assertions))]
    pub fn debug<A: Display>(&self, _input: A) {}

    pub fn log<A: Display>(&self, input: A, level: LogLevel) {
        let log_path = self.log_path.get_or_init(|| {
            let mut log_root = PathBuf::from_str(TEMP_PATH).expect("Could not get debug path!");
            log_root.push("autopower\\");
            std::fs::create_dir_all(&log_root).unwrap();

            let mut log_path = PathBuf::from_str(TEMP_PATH).expect("Could not get debug path!");
            log_path.push("autopower\\");
            log_path.push(format!("log_{}.txt", self.process_name));
            log_path
        });

        if level < LOG_LEVEL {
            return;
        }

        let mut file = std::fs::File::options()
            .write(true)
            .append(true)
            .create(true)
            .read(true)
            .open(log_path)
            .expect("Could not open log file!");

        let time_now = time::OffsetDateTime::now_utc();
        let mut msg = format!(
            "[{} | ({} - {})] {}",
            time::PrimitiveDateTime::new(time_now.date(), time_now.time()),
            self.process_name,
            self.source_name,
            input
        );
        msg.push('\n');
        file.write_all(msg.as_bytes())
            .expect("Could not write to log file!");
    }
}
