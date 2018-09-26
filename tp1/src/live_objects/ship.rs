use rand;
use rand::Rng;
use live_objects::lake::Lake;
use live_objects::live_object::LiveObject;

use std::io::{Error, Read};
use std::time::Duration;
use std::thread::sleep;

pub struct Ship {
  lake: Lake,
  current_capacity: u32,
  destination: u32
}

impl LiveObject for Ship {
  fn new(lake: Lake) -> Ship {
    Ship {lake, current_capacity: 2, destination: 0}
  }

  fn tick(&mut self) -> Result<(), Error> {
    self.travel();
    self.lake.lock_port(self.destination)?;
    self.pick_passengers();
    self.disembark();
    self.lake.unlock_port(self.destination)?;
    self.destination = self.lake.get_next_port(self.destination);
    Ok(())
  }
}

impl Ship {
  fn travel(&self) {
    let mut rng = rand::thread_rng();
    let msecs = (rng.gen::<u32>() % 2000) + 500;
    let travel_time = Duration::from_millis(msecs as u64);
    println!("Port {:?} Travelling {:?} msecs", self. destination, travel_time);
    sleep(travel_time);
  }

  fn disembark(&mut self) {
    let mut rng = rand::thread_rng();
    let msecs = (rng.gen::<u32>() % 2000) + 500;
    let disembark_time = Duration::from_millis(msecs as u64);
    println!("Port {:?} Disembarking {:?} msecs", self. destination, disembark_time);
    self.current_capacity = 2;
    sleep(disembark_time);
  }

  fn pick_passengers(&mut self) {
    let mut reader = self.lake.get_passenger_pipe_reader(self.destination).expect("Failed to get pipe");
    while self.current_capacity > 0 {
      let mut buf = String::new();
      let bytes_read = reader.read_to_string(&mut buf);
      match bytes_read {
        Ok(_bytes) => {
          let passenger_id = buf.parse::<u32>();
          println!("Abordó {:?}", passenger_id);
          self.current_capacity -= 1;
        },
        Err(e) => println!("{:?}", e),
      }
    }
  }
}
