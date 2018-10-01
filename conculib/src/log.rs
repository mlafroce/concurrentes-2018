use super::ipc::flock::FileLock;

use chrono::Local;

use std::fs::OpenOptions;
use std::io;
use std::process;
use std::io::{Write};
use std::cell::RefCell;
use std::str::FromStr;

#[derive(Debug)]
pub enum LogSeverity {
  ERROR,
  WARN,
  INFO,
  DEBUG
}

pub struct Log {
  file_lock: FileLock
}

thread_local! {
  pub static GLOBAL_LOG: RefCell<Log> = RefCell::new(
    Log::create(String::from_str("tp.log").unwrap()).unwrap()
  );
}

#[macro_export]
macro_rules! log {
  ($msg: expr, $severity: expr) => {{
    GLOBAL_LOG.with(|log_cell| {
      let mut log = log_cell.borrow_mut();
      log.log($msg, $severity);
    })
  }}
}

impl Log {
  pub fn create(path: String) -> io::Result<Log> {
    let mut options = OpenOptions::new();
    options.append(true).create(true);
    let file_lock = FileLock::new_with_options(path, options)?;
    Ok(Log{file_lock})
  }

  pub fn log(&mut self, message: &str, severity: LogSeverity) -> io::Result<()>{
    self.file_lock.lock_exclusive()?;
    let date = Local::now();
    let date_str = date.format("%Y-%m-%d %H:%M:%S");
    let id = process::id();
    let fmt_msg = format!("{} [{}] [{:?}] - {}\n", date_str, id, severity, message);
    self.file_lock.file.write_all(fmt_msg.as_bytes())?;
    self.file_lock.unlock()
  }
}