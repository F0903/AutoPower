use once_cell::unsync::{Lazy, OnceCell};
use std::{fmt::Display, io::Write, ops::Deref, path::PathBuf, str::FromStr};

const TEMP_PATH: &str = std::env!("TEMP");

pub struct Logger {
    source_name: &'static str,
    process_name: &'static str,
    log_root: Lazy<PathBuf>,
    log_path: OnceCell<PathBuf>,
}

impl Logger {
    pub const fn new(source_name: &'static str, process_name: &'static str) -> Self {
        let log_root: Lazy<PathBuf> = Lazy::new(|| {
            let mut log_root = PathBuf::from_str(TEMP_PATH).expect("Could not get debug path!");
            log_root.push("autopower\\");
            log_root
        });

        Self {
            source_name,
            process_name,
            log_root,
            log_path: OnceCell::new(),
        }
    }

    pub fn debug_log<A: Display>(&self, input: A) {
        std::fs::create_dir_all(self.log_root.deref()).unwrap();

        self.log_path.get_or_init(|| {
            let mut log_path = PathBuf::from_str(TEMP_PATH).expect("Could not get debug path!");
            log_path.push("autopower\\");
            log_path.push(format!("log_{}.txt", self.process_name));
            log_path
        });

        let mut file = std::fs::File::options()
            .write(true)
            .append(true)
            .create(true)
            .read(true)
            .open(self.log_path.get().unwrap())
            .expect("Could not open log file!");

        let time_now = time::OffsetDateTime::now_utc();
        let mut msg = format!(
            "[{} | {}] {}",
            time::PrimitiveDateTime::new(time_now.date(), time_now.time()),
            self.source_name,
            input
        );
        msg.push('\n');
        file.write_all(msg.as_bytes())
            .expect("Could not write to log file!");
    }
}