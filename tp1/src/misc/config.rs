use live_objects::main_lock::MainLockInfo;

use std::collections::HashMap;
use std::fs::File;
use std::fs::metadata;
use std::io;
use std::io::{BufRead, BufReader};
use std::io::{Error, ErrorKind};
use std::time::SystemTime;

pub struct Config {
  pub config_map: HashMap<String, String>,
  pub timestamp: u64
}

impl Config {
  pub fn new(path: &str, lock_info: &MainLockInfo) -> Result<Config, Error> {
    let metadata = metadata(path)?;
    let metadata_modified = metadata.modified()?;
    let config_timestamp = metadata_modified.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

    println!("{:?} vs {:?}", config_timestamp, lock_info.timestamp);
    if config_timestamp != lock_info.timestamp {
      return Err(Error::new(ErrorKind::InvalidData, "Invalid timestamp!"));
    }
    let config_map = Config::read_config(path)?;
    Ok(Config {config_map: config_map, timestamp: config_timestamp})
  }

  fn read_config(path: &str) -> io::Result<HashMap<String, String>> {
    let file = File::open(path)?;
    let buf = BufReader::new(file);
    let mut config_map = HashMap::new();
    for line in buf.lines() {
      let buf_line = line.unwrap();
      let values : Vec<&str> = buf_line.split("=").collect();
      config_map.insert(values[0].to_string(), values[1].to_string());
    }
    Ok(config_map)
  }

  pub fn get(&self, key: &str) -> Option<&String> {
    self.config_map.get(key)
  }
}