use rand;
use rand::Rng;

use concurrentes::log::{GLOBAL_LOG, Log, LogSeverity};

use live_objects::lake::Lake;
use live_objects::live_object::LiveObject;

use std::io::{Error, Read};
use std::time::Duration;
use std::thread::sleep;
use std::cell::RefCell;

pub struct Ship {
  current_capacity: u32,
  destination: u32
}

impl LiveObject for Ship {
  fn tick(&mut self, lake: &RefCell<Lake>) -> Result<(), Error> {
    self.travel();
    lake.borrow_mut().lock_port(self.destination)?;
    self.pick_passengers(lake);
    self.disembark();
    lake.borrow_mut().unlock_port(self.destination)?;
    self.destination = lake.borrow_mut().get_next_port(self.destination);
    Ok(())
  }
}


impl Ship {
  pub fn new(current_capacity: u32, destination: u32) -> Ship {
    Ship {current_capacity: 2, destination: 0}
  }

  fn travel(&self) {
    let mut rng = rand::thread_rng();
    let msecs = (rng.gen::<u32>() % 1000);
    let travel_time = Duration::from_millis(msecs as u64);
    let msg = format!("Viajando {} msecs al puerto {}",
      msecs, self.destination);
    log!(msg.as_str(), LogSeverity::INFO);
    sleep(travel_time);
  }

  fn disembark(&mut self) {
    let mut rng = rand::thread_rng();
    let msecs = (rng.gen::<u32>() % 2000) + 500;
    let disembark_time = Duration::from_millis(msecs as u64);
    self.current_capacity = 2;
    let msg = format!("Desembarcando en {} msecs, {} lugares libres",
      msecs, self.current_capacity);
    log!(msg.as_str(), LogSeverity::INFO);
    sleep(disembark_time);
  }

  fn pick_passengers(&mut self, lake: &RefCell<Lake>) {
    log!("Obteniendo fifo", LogSeverity::DEBUG);
    match lake.borrow_mut().get_passenger_pipe_reader(self.destination) {
      Ok(mut reader) => {
        while self.current_capacity > 0 {
          let mut buf = String::new();
          let bytes_read = reader.read_to_string(&mut buf);
          match bytes_read {
            Ok(_bytes) => {
              let passenger_id = buf.parse::<u32>();
              let msg = format!("Abordó el pasajero {:?}",
                passenger_id);
              log!(msg.as_str(), LogSeverity::INFO);
              self.current_capacity -= 1;
            },
            Err(e) => log!(format!("{:?}", e).as_str(), LogSeverity::WARN),
          }
        }
      }
      Err(e) => {
        let msg = format!("Error al esperar pasajeros en el puerto {}: {:?}",
          self.destination, e);
        log!(msg.as_str(), LogSeverity::ERROR);;
      }
    }
  }
}
