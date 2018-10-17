use libc;
use rand;
use rand::Rng;

use concurrentes::signal::signal;
use concurrentes::log::{GLOBAL_LOG, LogSeverity};

use live_objects::lake::Lake;
use live_objects::live_object::LiveObject;

use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use std::time::Duration;
use std::thread::sleep;

pub struct Inspector {
  current_port: i32,
  status: Status,
  /// Puede ser inspector (true) o prefectura (false)
  is_inspector: bool,
  /// Ultimo barco inspeccionado
  last_inspection: u32
}

#[derive(Debug)]
enum Status {
  Inspect,
  Travel
}

impl LiveObject for Inspector {
  fn tick(&mut self, lake: &RefCell<Lake>) -> Result<(), io::Error> {
    match self.status {
      Status::Inspect => self.inspect(lake),
      Status::Travel => self.travel(lake)
    }
    Ok(())
  }
}


impl Inspector {
  pub fn new(current_port: i32, is_inspector: bool) -> Inspector {
    log!(format!("Iniciando inspector en el puerto {}", current_port).as_str(), &LogSeverity::INFO);
    let status = Status::Inspect;
    let last_inspection = 0;
    Inspector { current_port, status, is_inspector, last_inspection }
  }

  pub fn travel(&mut self, lake: &RefCell<Lake>) {
    let travel_time = Duration::from_millis(1000);
    log!(format!("Paseando {} msecs", 1000).as_str(), &LogSeverity::INFO);
    sleep(travel_time);
    self.current_port = lake.borrow_mut().get_random_port();
    self.status = Status::Inspect;
  }

  /// Ejecuta una inspección en el puerto donde se encuentra
  pub fn inspect(&mut self, lake: &RefCell<Lake>) {
    if let Some(ship) = lake.borrow_mut().get_ship_at(0) {
      if self.last_inspection != ship {
        log!(format!("Iniciando inspección del barco {}", ship).as_str(), &LogSeverity::INFO);
        self.last_inspection = ship;
        if self.is_inspector {
          signal(ship as i32, libc::SIGUSR1);
        } else {
          signal(ship as i32, libc::SIGUSR2);
        }
      }
    } else {
      log!("No encontró ningún barco", &LogSeverity::INFO);
    }
    self.status = Status::Travel;
  }
}
