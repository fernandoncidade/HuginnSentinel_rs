use parking_lot::Mutex;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::OnceLock;

static LOGGER: OnceLock<LogManager> = OnceLock::new();

pub struct LogManager {
    file: Mutex<Option<File>>,
}

impl LogManager {
    pub fn init() -> &'static Self {
        LOGGER.get_or_init(|| {
            let log_dir = super::paths::obter_caminho_persistente();
            let log_path = log_dir.join("huginn.log");
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)
                .ok();

            LogManager {
                file: Mutex::new(file),
            }
        })
    }

    pub fn log(&self, level: &str, msg: &str) {
        let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let line = format!("[{}] [{}] {}\n", ts, level, msg);

        eprint!("{}", line);

        let mut lock = self.file.lock();
        if let Some(f) = lock.as_mut() {
            let _ = f.write_all(line.as_bytes());
            let _ = f.flush();
        }
    }

    pub fn debug(&self, msg: &str) {
        self.log("DEBUG", msg);
    }

    pub fn info(&self, msg: &str) {
        self.log("INFO", msg);
    }

    pub fn warn(&self, msg: &str) {
        self.log("WARN", msg);
    }

    pub fn error(&self, msg: &str) {
        self.log("ERROR", msg);
    }

    pub fn critical(&self, msg: &str) {
        self.log("CRITICAL", msg);
    }
}

pub fn get_logger() -> &'static LogManager {
    LogManager::init()
}
