use concurrentes::signal::SignalHandler;
use libc;

pub struct QuitHandler {
  quit: bool
}

impl SignalHandler for QuitHandler {
  fn handle(&mut self) {
    println!("Exit signal handled");
    self.quit = true;
    unsafe {
      libc::close(0);
    }
  }
}

impl QuitHandler {
  pub fn new() -> QuitHandler {
    QuitHandler {quit: false}
  }

  pub fn has_graceful_quit(&self) -> bool {
    self.quit
  }
}