extern crate concurrentes;
extern crate libc;

use concurrentes::signal::{SignalHandler, SignalHandlerDispatcher};

use std::rc::Rc;
use std::{thread, time};
use std::io;
use std::cell::RefCell;

use libc::SIGINT;

struct SigIntHandler {
  quit: bool
}

impl SignalHandler for SigIntHandler {
  fn handle(&mut self) {
    println!("SigInt handled");
    self.quit = true;
  }
}

impl SigIntHandler {
  pub fn new() -> SigIntHandler {
    SigIntHandler {quit: false}
  }

  pub fn has_graceful_quit(&self) -> bool {
    self.quit
  }
}

fn main() -> io::Result<()> {

  // Use RC so I can keep one reference here in main and one in the handler
  let sigint_handler = Rc::new(RefCell::new(SigIntHandler::new()));
  SignalHandlerDispatcher::register(SIGINT, sigint_handler.clone());

  while !sigint_handler.borrow().has_graceful_quit() {
    println!("Running");
    let millis = time::Duration::from_millis(500);
    thread::sleep(millis);
  }
  println!("Exit");
  Ok(())
}