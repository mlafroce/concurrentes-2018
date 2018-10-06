use handlers::signal_handler::QuitHandler;

use live_objects::main_lock::MainLock;
use live_objects::lake::Lake;
use misc::config::Config;

use std::cell::RefCell;
use std::io;
use std::rc::Rc;

const MAIN_LOCK_FILENAME : &str = "tp1.lock";
const MAIN_CONFIG_FILENAME: &str = "config.cfg";

pub trait LiveObject {
  fn tick(&mut self, &RefCell<Lake>) -> Result<(), io::Error>;
}

pub struct LiveObjectRunner {
  quit_handler: Rc<RefCell<QuitHandler>>,
  lake: RefCell<Lake>
}

impl LiveObjectRunner {
  pub fn new(quit_handler: Rc<RefCell<QuitHandler>>) -> io::Result<(LiveObjectRunner)> {
    
    // Load lock info
    let mut main_lock = MainLock::new(MAIN_LOCK_FILENAME)?;
    main_lock.lock.lock_exclusive()?;
    // Load config
    let mut lock_info = main_lock.get_info();
    let lake_config = Config::new(MAIN_CONFIG_FILENAME, &lock_info)?;

    let lake = if lock_info.is_counter_zero() {
      Lake::init(&lake_config)
    } else {
      Lake::load(&lake_config)
    };
    lock_info.counter_inc();
    lock_info.save(MAIN_LOCK_FILENAME)?;
    main_lock.lock.unlock()?;
    Ok(LiveObjectRunner{quit_handler, lake: RefCell::new(lake)})
  }

  // Main loop
  pub fn run<T: LiveObject>(&self, mut object: T) -> io::Result<()> {
    // Start object
    while !self.quit_handler.borrow().has_graceful_quit() {
      object.tick(&self.lake)?;
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
    if lock_info.is_counter_zero() {
      self.lake.borrow_mut().destroy()?;
    }
    main_lock.lock.unlock()?;
    // Exit
    Ok(())
  }
}
