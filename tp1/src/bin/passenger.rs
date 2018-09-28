extern crate concurrentes;
extern crate rand;
extern crate libc;
extern crate tp1;

use tp1::live_objects::{live_object, passenger::Passenger};
use std::io;

fn main() -> io::Result<()> {
  let passenger = Passenger::new();
  let runner = live_object::LiveObjectRunner::new()?;
  runner.run(passenger)?;
  runner.exit()
}
