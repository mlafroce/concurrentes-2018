extern crate concurrentes;
extern crate rand;

mod main_lock;
mod config;
mod lake;
mod ship;

use main_lock::MainLockInfo;
use config::Config;
use lake::Lake;
use ship::Ship;

use concurrentes::ipc::flock::FileLock;

use std::io;

const MAIN_LOCK_FILENAME : &str = "tp1.lock";
const MAIN_CONFIG_FILENAME: &str = "config.cfg";

fn main() -> io::Result<()> {
  let mut main_lock = FileLock::create(MAIN_LOCK_FILENAME)?;
  main_lock.lock_exclusive()?;
  let mut lock_info = match MainLockInfo::read_info(MAIN_LOCK_FILENAME) {
    Ok(info) => info,
    Err(e) => {
      println!("Error! {:?}", e);
      MainLockInfo::create(MAIN_CONFIG_FILENAME).unwrap()
    }
  };
  let lake_config = Config::new(MAIN_CONFIG_FILENAME, &lock_info)?;
  let lake: Lake;
  if lock_info.is_counter_zero() {
    lake = Lake::init(&lake_config);
  } else {
    lake = Lake::load(&lake_config);
  }
  lock_info.counter_inc();
  lock_info.save(MAIN_LOCK_FILENAME)?;
  main_lock.unlock()?;
  
  let mut ship = Ship::new(lake);
  ship.run()?;

  Ok(())
}
