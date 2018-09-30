use live_objects::lake::Lake;
use live_objects::live_object::LiveObject;

use std::io::{Error, Write};
use std::cell::RefCell;

pub struct Passenger {}

impl LiveObject for Passenger {
  fn tick(&mut self, lake: &RefCell<Lake>) -> Result<(), Error> {
    self.take_ship(lake);
    Ok(())
  }
}


impl Passenger {
  pub fn new() -> Passenger {
    Passenger {}
  }

  fn take_ship(&mut self, lake: &RefCell<Lake>) {
    let mut writer = lake.borrow_mut().
      get_passenger_pipe_writer(0).expect("Failed to get pipe");
    writer.write_all(b"Test").expect("Failed to  write");
  }
}
