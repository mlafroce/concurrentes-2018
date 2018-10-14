use concurrentes::signal::SignalHandler;
use libc;

/// Handler para una salida agraciada
#[derive(Default)]
pub struct QuitHandler {
  quit: bool
}

/// Handler genérico multiuso
#[derive(Default)]
pub struct GenericHandler {
  handled: bool
}

impl SignalHandler for QuitHandler {
  fn handle(&mut self) {
    self.quit = true;
    unsafe {
      libc::close(0);
    }
  }
}

impl QuitHandler {
  /// Constructor default
  pub fn new() -> QuitHandler {
    QuitHandler {quit: false}
  }

  /// Devuelve `true` si la señal fue capturada
  pub fn has_graceful_quit(&self) -> bool {
    self.quit
  }
}

impl SignalHandler for GenericHandler {
  fn handle(&mut self) {
    self.handled = true;
  }
}

impl GenericHandler {
  /// Constructor default 
  pub fn new() -> GenericHandler {
    GenericHandler {handled: false}
  }

  /// Devuelve `true` si la señal fue capturada
  pub fn get_handled(&self) -> bool {
    self.handled
  }

  /// Resetea el indicador de captura
  pub fn reset(&mut self) {
    self.handled = false;
  }
}