use rand;
use rand::Rng;
use Lake;

use std::io::Error;
use std::time::Duration;
use std::thread::sleep;

pub struct Ship {
  lake: Lake,
  destination: u32
}

impl Ship {
  pub fn new(lake: Lake) -> Ship {
    Ship {lake: lake, destination: 0}
  }

  pub fn run(&mut self) -> Result<(), Error> {
    while(true) {
      self.travel();
      self.lake.lock_port(self.destination)?;
      self.disembark();
      self.lake.unlock_port(self.destination)?;
      self.destination = self.lake.get_next_port(self.destination);
    }
    Ok(())
  }

  fn travel(&self) {
    let mut rng = rand::thread_rng();
    let msecs = (rng.gen::<u32>() % 2000) + 500;
    let travel_time = Duration::from_millis(msecs as u64);
    println!("Port {:?} Travelling {:?} msecs", self. destination, travel_time);
    sleep(travel_time);
  }

  fn disembark(&self) {
    let mut rng = rand::thread_rng();
    let msecs = (rng.gen::<u32>() % 2000) + 500;
    let disembark_time = Duration::from_millis(msecs as u64);
    println!("Port {:?} Disembarking {:?} msecs", self. destination, disembark_time);
    sleep(disembark_time);
  }
}