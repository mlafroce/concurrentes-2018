use rand;
use rand::Rng;

use libc;

use concurrentes::log::{GLOBAL_LOG, LogSeverity};
use concurrentes::signal::SignalHandlerDispatcher;

use handlers::signal_handler::GenericHandler;

use live_objects::lake::Lake;
use live_objects::live_object::LiveObject;

use std::io::{Error, BufRead, BufReader};
use std::time::Duration;
use std::thread::sleep;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Ship {
  current_capacity: u32,
  destination: u32,
  sigusr_handler: Rc<RefCell<GenericHandler>>,
  status: Status
}

#[derive(Debug)]
enum Status {
  Travel,
  PickPassengers,
  Disembark
}

impl LiveObject for Ship {
  fn tick(&mut self, lake: &RefCell<Lake>) -> Result<(), Error> {
    match self.status {
      Status::Travel => {
        self.travel(lake)?;
        self.status = Status::PickPassengers;
      },
      Status::PickPassengers => {
        if self.current_capacity > 0 {
          self.pick_passenger(lake);
          self.status = Status::Disembark;
        } else {
          self.status = Status::Disembark;
        }
      },
      Status::Disembark => {
        self.disembark(lake)?;
        self.status = Status::Travel;
      }
    }
    Ok(())
  }
}


impl Ship {
  pub fn new(current_capacity: u32, destination: u32) -> Ship {
    // Acá me recontra abuso del supuesto de que hay un sólo barco por proceso
    let sigusr_handler = Rc::new(RefCell::new(GenericHandler::new()));
    SignalHandlerDispatcher::register(libc::SIGUSR1, sigusr_handler.clone());
    Ship {current_capacity, destination, sigusr_handler, status: Status::Travel}
  }

  fn travel(&self, lake: &RefCell<Lake>) -> Result<(), Error> {
    let mut rng = rand::thread_rng();
    let msecs = rng.gen::<u32>() % 1000;
    let travel_time = Duration::from_millis(msecs as u64);
    let msg = format!("Viajando {} msecs al puerto {}",
      msecs, self.destination);
    log!(msg.as_str(), LogSeverity::INFO);
    sleep(travel_time);
    lake.borrow_mut().lock_port(self.destination)?;
    log!("Puerto bloqueado", LogSeverity::DEBUG);
    Ok(())
  }

  fn disembark(&mut self, lake: &RefCell<Lake>) -> Result<(), Error> {
    let mut rng = rand::thread_rng();
    let msecs = (rng.gen::<u32>() % 1000) + 500;
    let disembark_time = Duration::from_millis(msecs as u64);
    self.current_capacity = 2;
    let msg = format!("Desembarcando en {} msecs, {} lugares libres",
      msecs, self.current_capacity);
    log!(msg.as_str(), LogSeverity::INFO);
    sleep(disembark_time);
    lake.borrow_mut().unlock_port(self.destination)?;
    log!("Puerto desbloqueado", LogSeverity::DEBUG);
    self.destination = lake.borrow_mut().get_next_port(self.destination);
    Ok(())
  }

  fn pick_passenger(&mut self, lake: &RefCell<Lake>) -> Option<u32> {
    log!("Obteniendo fifo", LogSeverity::DEBUG);
    let pipe_reader = lake.borrow_mut().get_passenger_pipe_reader(self.destination);
    match pipe_reader {
      Ok(reader) => {
        let mut buf_line = String::new();
        let mut buf_reader = BufReader::new(reader);
        let bytes_read = buf_reader.read_line(&mut buf_line);
        let msg = format!("Levantando pasajero, leido {:?}",
              buf_line);
            log!(msg.as_str(), LogSeverity::DEBUG);
        match bytes_read {
          Ok(0) => None,
          Ok(_) => {
            let passenger_id = buf_line.parse::<u32>().unwrap();
            let msg = format!("Abordó el pasajero {:?}",
              passenger_id);
            log!(msg.as_str(), LogSeverity::INFO);
            self.current_capacity -= 1;
            Some(passenger_id)
          },
          Err(e) => {
            log!(format!("{:?}", e).as_str(), LogSeverity::WARN);
            None
          }
        }
      }
      Err(e) => {
        let msg = format!("Error al esperar pasajero en el puerto {}: {:?}",
          self.destination, e);
        log!(msg.as_str(), LogSeverity::ERROR);
        None
      }
    };
    if self.sigusr_handler.borrow().get_handled() {
      self.status = Status::Disembark;
    }
    let msg = format!("Hay lugar para {:?} pasajeros",
      self.current_capacity);
    log!(msg.as_str(), LogSeverity::DEBUG);
    None
  }
}
