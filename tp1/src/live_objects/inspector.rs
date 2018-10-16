use libc;
use rand;
use rand::Rng;

use concurrentes::signal::signal;
use concurrentes::log::{GLOBAL_LOG, LogSeverity};

use live_objects::lake::Lake;
use live_objects::live_object::LiveObject;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use std::thread::sleep;

pub struct Inspector {
  current_port: i32,
  status: Status
}

#[derive(Debug)]
enum Status {
  Inspect,
  Travel
}

impl LiveObject for Inspector {
  fn tick(&mut self, lake: &RefCell<Lake>) -> Result<(), Error> {
    match self.status {
      Status::Inspect => self.inspect(lake)?,
      Status::Travel => self.travel(lake)?
    }
    Ok(())
  }
}


impl Inspector {
  pub fn new(current_port: u32) -> Inspector {
    log!(format!("Iniciando inspector en el puerto {}", current_port).as_str());
    let status = Status::Inspect;
    Inspector { current_port, status }
  }

  pub fn travel(&mut self, lake: &RefCell<Lake>) {
    let travel_time = Duration::from_millis(1000);
    sleep(travel_time);
    self.status = Status::Inspect;
  }

  pub fn inspect(&mut self, lake: &RefCell<Lake>) {
    if let Some(ship) = lake.borrow_mut().get_ship_at(self.current_port) {
      log!(format!("Iniciando inspección del barco {}", ship).as_str());
      signal(ship, libc::SIGUSR1);
    } else {
      log!("No encontró ningún barco");
    }
    self.status = Status::Travel;
  }
}
