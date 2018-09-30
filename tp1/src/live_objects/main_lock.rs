use concurrentes::ipc::flock::FileLock;

use std::fs::{File, metadata};
use std::io;
use std::io::{BufRead, BufReader, Write};
use std::io::{Error, ErrorKind};
use std::str::FromStr;
use std::time::SystemTime;

pub struct MainLock {
  path: &'static str,
  pub lock: FileLock
}

pub struct MainLockInfo {
  process_counter: i32,
  pub timestamp: u64
}

impl MainLock {
  pub fn new(path : &'static str) -> Result<MainLock, Error> {
    let lock = FileLock::create(String::from_str(path).unwrap())?;
    Ok(MainLock{path: path, lock: lock})
  } 

  pub fn get_info(&self) -> MainLockInfo {
  match MainLockInfo::read_info(self.path) {
      Ok(info) => info,
      Err(_e) => {
        MainLockInfo::create(self.path).unwrap()
      }
    }
  }
}

impl MainLockInfo {
  pub fn read_info(path: &str) -> Result<MainLockInfo, Error> {
    let file = File::open(path)?;
    let mut buf = BufReader::new(file);
    // Read process counter
    let mut buf_line = String::new();
    buf.read_line(&mut buf_line)?;
    buf_line.pop();
    let result = buf_line.parse::<i32>();
    let counter = match result {
      Ok(read_value) => {read_value},
      _error => {0},
    };
    buf_line.clear();
    // Read config config_timestamp
    buf.read_line(&mut buf_line)?;
    buf_line.pop();
    match buf_line.parse::<u64>() {
      Ok(timestamp) => Ok(MainLockInfo {process_counter: counter, timestamp: timestamp}),
      Err(_e) => Err(Error::new(ErrorKind::InvalidData, "missing timestamp!")),
    }
  }

  pub fn create(path: &str) -> Result<MainLockInfo, Error> {
    let metadata = metadata(path)?;
    let metadata_modified = metadata.modified()?;
    println!("{:?} ---> Config meta: {:?}", path, metadata);
    let config_timestamp = metadata_modified.duration_since(
      SystemTime::UNIX_EPOCH).unwrap().as_secs();
    Ok(MainLockInfo {process_counter: 0, timestamp: config_timestamp})
  }

  pub fn is_counter_zero(&self) -> bool {
    self.process_counter == 0
  }

  pub fn counter_inc(&mut self) {
    self.process_counter += 1;
  }

  pub fn counter_dec(&mut self) {
    self.process_counter -= 1;
  }

  pub fn save(&self, path: &str) -> io::Result<()>{
    let mut file = File::create(path)?;
    let data = format!("{:?}\n{:?}\n", self.process_counter, self.timestamp);
    file.write_all(data.as_bytes())
  }
}
