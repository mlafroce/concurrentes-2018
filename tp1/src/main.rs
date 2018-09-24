extern crate concurrentes;
extern crate rand;
extern crate libc;

mod main_lock;
mod config;
mod lake;
mod ship;
mod ship_signal_handler;

use libc::SIGINT;

use concurrentes::signal::SignalHandlerDispatcher;

use config::Config;
use main_lock::{MainLock, MainLockInfo};
use lake::Lake;
use ship::Ship;
use ship_signal_handler::ShipSigIntHandler;

use std::cell::RefCell;
use std::io;
use std::rc::Rc;

const MAIN_LOCK_FILENAME : &str = "tp1.lock";
const MAIN_CONFIG_FILENAME: &str = "config.cfg";

fn main() -> io::Result<()> {
  // Register signal handler
  let sigint_handler = Rc::new(RefCell::new(ShipSigIntHandler::new()));
  SignalHandlerDispatcher::register(SIGINT, sigint_handler.clone());
  // Load lock info
  let mut main_lock = MainLock::new(MAIN_LOCK_FILENAME)?;
  main_lock.lock.lock_exclusive()?;
  // Load config
  let mut lock_info = main_lock.get_info();
  let lake_config = Config::new(MAIN_CONFIG_FILENAME, &lock_info)?;

  let lake: Lake;
  if lock_info.is_counter_zero() {
    lake = Lake::init(&lake_config);
  } else {
    lake = Lake::load(&lake_config);
  }
  lock_info.counter_inc();
  lock_info.save(MAIN_LOCK_FILENAME)?;
  main_lock.lock.unlock()?;
  // Start ship
  let mut ship = Ship::new(lake);
  // Main loop
  while !sigint_handler.borrow().has_graceful_quit() {
    ship.tick()?;
  }
  // Save lock info
  main_lock.lock.lock_exclusive()?;
  lock_info = main_lock.get_info();
  lock_info.counter_dec();
  lock_info.save(MAIN_LOCK_FILENAME)?;
  main_lock.lock.unlock()?;
  // Exit
  Ok(())
}
