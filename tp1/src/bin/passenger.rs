extern crate concurrentes;
extern crate rand;
extern crate libc;
extern crate tp1;

use concurrentes::signal::SignalHandlerDispatcher;

use tp1::handlers::signal_handler::QuitHandler;
use tp1::live_objects::{live_object, passenger::Passenger};

use std::io;
use std::rc::Rc;
use std::cell::RefCell;

fn main() -> io::Result<()> {
  let quit_handler = Rc::new(RefCell::new(QuitHandler::new()));
  SignalHandlerDispatcher::register(libc::SIGINT, quit_handler.clone());
  SignalHandlerDispatcher::register(libc::SIGTERM, quit_handler.clone());

  let runner = live_object::LiveObjectRunner::new(quit_handler)?;

  let destination = runner.get_random_port();
  let current_port = runner.get_random_port();
  let passenger = Passenger::new(current_port, destination);
  runner.run(passenger)?;
  runner.exit()
}
