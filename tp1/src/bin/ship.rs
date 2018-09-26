extern crate concurrentes;
extern crate rand;
extern crate libc;
extern crate tp1;

use tp1::live_objects::{live_object, ship::Ship};
use std::io;

fn main() -> io::Result<()> {
  let runner = live_object::LiveObjectRunner::new()?;
  runner.run::<Ship>()?;
  runner.exit()
}
