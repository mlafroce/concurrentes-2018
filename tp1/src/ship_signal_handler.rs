use concurrentes::signal::SignalHandler;

pub struct ShipSigIntHandler {
  quit: bool
}

impl SignalHandler for ShipSigIntHandler {
  fn handle(&mut self) {
    println!("SigInt handled");
    self.quit = true;
  }

}

impl ShipSigIntHandler {
  pub fn new() -> ShipSigIntHandler {
    ShipSigIntHandler {quit: false}
  }

  pub fn has_graceful_quit(&self) -> bool {
    self.quit
  }
}