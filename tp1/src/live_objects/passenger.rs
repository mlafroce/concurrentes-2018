use live_objects::lake::Lake;
use live_objects::live_object::LiveObject;

use std::cell::RefCell;
use std::io::{Error, Write};
use std::process;

pub struct Passenger {
  destination: u32,
  id: u32
}

impl LiveObject for Passenger {
  fn tick(&mut self, lake: &RefCell<Lake>) -> Result<(), Error> {
    self.take_ship(lake);
    Ok(())
  }
}


impl Passenger {
  pub fn new() -> Passenger {
    let destination = 0;
    let id = process::id();
    Passenger {destination, id}
  }

  fn take_ship(&mut self, lake: &RefCell<Lake>) {
    let mut writer = lake.borrow_mut().
      get_passenger_pipe_writer(0).expect("Failed to get pipe");
    writer.write_all(self.id.to_string().as_bytes()).expect("Failed to  write");
  }
}
