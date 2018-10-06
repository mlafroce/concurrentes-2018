use concurrentes::signal::SignalHandler;
use libc;

#[derive(Default)]
pub struct QuitHandler {
  quit: bool
}

#[derive(Default)]
pub struct GenericHandler {
  handled: bool
}

/// Sets `quit` flag true and closes stdin in order to finish launcher prompt
/// 
impl SignalHandler for QuitHandler {
  fn handle(&mut self) {
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

  /// Returns true if signal got handled
  pub fn has_graceful_quit(&self) -> bool {
    self.quit
  }
}

impl SignalHandler for GenericHandler {
  fn handle(&mut self) {
    self.handled = true;
  }
}

/// Generic handler for multiple uses
impl GenericHandler {
  /// Default constructor
  pub fn new() -> GenericHandler {
    GenericHandler {handled: false}
  }

  /// Returs `true` if signal got handled
  pub fn get_handled(&self) -> bool {
    self.handled
  }

  /// Sets `handled` flag on false
  pub fn reset(&mut self) {
    self.handled = false;
  }
}