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
  fn new(lake: &RefCell<Lake>) -> Ship;
  fn tick(&mut self) -> Result<(), io::Error>;
}

pub struct LiveObjectRunner {
  sigint_handler: Rc<RefCell<SigIntHandler>>,
  lake: RefCell<Lake>
}

impl LiveObjectRunner {
  pub fn new() -> io::Result<(LiveObjectRunner)> {
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
    Ok(LiveObjectRunner{sigint_handler, lake: RefCell::new(lake)})
  }

  // Main loop
  pub fn run<T: LiveObject>(&self) -> io::Result<()> {
    // Start object
    let mut object = T::new(&self.lake);
    while !self.sigint_handler.borrow().has_graceful_quit() {
      object.tick()?;
    }
    Ok(())
  }
  
  /// TODO? Drop
  pub fn exit(&self) -> io::Result<()> { 
    // Save lock info
    let mut main_lock = MainLock::new(MAIN_LOCK_FILENAME)?;
    main_lock.lock.lock_exclusive()?;
    let mut lock_info = main_lock.get_info();
    lock_info.counter_dec();
    lock_info.save(MAIN_LOCK_FILENAME)?;
    main_lock.lock.unlock()?;
    // Exit
    Ok(())
  }
}
