use libc::SIGINT;

use concurrentes::signal::SignalHandlerDispatcher;

use live_objects::main_lock::MainLock;
use live_objects::lake::Lake;
use live_objects::ship::Ship;
use handlers::signal_handler::SigIntHandler;
use misc::config::Config;

use std::cell::RefCell;
use std::io;
use std::rc::Rc;

const MAIN_LOCK_FILENAME : &str = "tp1.lock";
const MAIN_CONFIG_FILENAME: &str = "config.cfg";

pub trait LiveObject {
  fn new(lake: Lake) -> Ship;
  fn tick(&mut self) -> Result<(), io::Error>;
}

pub fn start<T: LiveObject>() -> io::Result<()> {
  // Register signal handler
  let sigint_handler = Rc::new(RefCell::new(SigIntHandler::new()));
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
  // Main loop
  // Start object
  let mut object = T::new(lake);
  while !sigint_handler.borrow().has_graceful_quit() {
    object.tick()?;
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
