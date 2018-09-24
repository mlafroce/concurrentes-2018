use concurrentes::signal::SignalHandler;

pub struct SigIntHandler {
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