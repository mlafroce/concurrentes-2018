use rand;
use rand::Rng;

use libc;

use concurrentes::ipc::Key;
use concurrentes::ipc::flock::FileLock;
use concurrentes::ipc::semaphore::Semaphore;
use concurrentes::ipc::named_pipe;
use concurrentes::log::{GLOBAL_LOG, LogSeverity};
use concurrentes::signal::SignalHandlerDispatcher;

use handlers::signal_handler::GenericHandler;

use live_objects::lake::Lake;
use live_objects::live_object::LiveObject;

use std::cell::RefCell;
use std::io;
use std::io::{Error, BufRead, BufReader, Write};
use std::rc::Rc;
use std::time::Duration;
use std::thread::sleep;

pub struct Ship {
  current_capacity: u32,
  destination: u32,
  passenger_vec: Vec<u32>,
  sigusr_handler: Rc<RefCell<GenericHandler>>,
  status: Status
}

#[derive(Debug)]
enum Status {
  Travel,
  LeavePassengers,
  PickPassengers,
  Disembark
}

impl LiveObject for Ship {
  fn tick(&mut self, lake: &RefCell<Lake>) -> Result<(), Error> {
    match self.status {
      Status::Travel => self.travel(lake)?,
      Status::LeavePassengers => self.leave_passenger(lake)?,
      Status::PickPassengers => {
        if self.current_capacity > 0 {
          self.pick_passenger(lake);
        } else {
          self.status = Status::Disembark;
        }
      },
      Status::Disembark => self.disembark(lake)?
    }
    Ok(())
  }
}


impl Ship {
  pub fn new(current_capacity: u32, destination: u32) -> Ship {
    // Acá me recontra abuso del supuesto de que hay un sólo barco por proceso
    let sigusr_handler = Rc::new(RefCell::new(GenericHandler::new()));
    SignalHandlerDispatcher::register(libc::SIGUSR1, sigusr_handler.clone());
    Ship {current_capacity, destination, sigusr_handler,
      status: Status::Travel, passenger_vec: Vec::new()}
  }

  fn travel(&mut self, lake: &RefCell<Lake>) -> io::Result<()> {
    let mut rng = rand::thread_rng();
    let msecs = rng.gen::<u32>() % 1000;
    let travel_time = Duration::from_millis(u64::from(msecs));
    let msg = format!("Viajando {} msecs al puerto {}",
      msecs, self.destination);
    log!(msg.as_str(), &LogSeverity::INFO);
    sleep(travel_time);
    lake.borrow_mut().lock_port(self.destination)?;
    log!("Puerto bloqueado", &LogSeverity::DEBUG);
    self.status = Status::LeavePassengers;
    Ok(())
  }

  fn disembark(&mut self, lake: &RefCell<Lake>) -> io::Result<()> {
    let mut rng = rand::thread_rng();
    let msecs = (rng.gen::<u32>() % 1000) + 500;
    let disembark_time = Duration::from_millis(u64::from(msecs));
    let msg = format!("Desembarcando en {} msecs, {} lugares libres",
      msecs, self.current_capacity);
    log!(msg.as_str(), &LogSeverity::INFO);
    sleep(disembark_time);
    lake.borrow_mut().unlock_port(self.destination)?;
    log!("Puerto desbloqueado", &LogSeverity::DEBUG);
    self.destination = lake.borrow_mut().get_next_port(self.destination);
    self.status = Status::Travel;
    Ok(())
  }

  fn leave_passenger(&mut self, lake: &RefCell<Lake>) -> io::Result<()>{
    let mut left_passengers = Vec::new();
    for passenger in &self.passenger_vec {
      log!(format!("Notificando pasajero {}", passenger).as_str(), &LogSeverity::DEBUG);
      let pipe_path = format!("passenger-{:?}.fifo", passenger);
      let lock_pipe_path = format!("passenger-{:?}.fifo.lock", passenger);
      FileLock::create(lock_pipe_path.clone()).unwrap();
      let key = Key::ftok(&lock_pipe_path, 0).unwrap();
      log!(format!("Obteniendo semaforo {}", passenger).as_str(), &LogSeverity::DEBUG);
      let sem = Semaphore::get(&key, 0).unwrap();
      sem.signal()?;
      log!(format!("Abriendo FIFO {} para escribir puerto", pipe_path).as_str(), &LogSeverity::DEBUG);
      let mut writer = named_pipe::NamedPipeWriter::open(pipe_path.as_str())?;
      log!(format!("Pipe abierto {}", passenger).as_str(), &LogSeverity::DEBUG);
      write!(writer, "{}\n", self.destination)?;
      log!(format!("Enviado puerto {}", self.destination).as_str(), &LogSeverity::DEBUG);
      if let Some(reply) = self.read_passenger_reply(lake)? {
        log!(format!("Descargando pasajero {:?}", reply).as_str(), &LogSeverity::INFO);
        left_passengers.push(reply);
      }
      log!(format!("Terminé de notificar pasajero {}", passenger).as_str(), &LogSeverity::DEBUG);
    }
    for discarded in left_passengers {
      log!(format!("Desanotando pasajero {:?}", discarded).as_str(), &LogSeverity::DEBUG);
      &self.passenger_vec.iter()
        .position(|&n| n == discarded)
        .map(|e| self.passenger_vec.remove(e));
      self.current_capacity += 1;
    }
    self.status = Status::PickPassengers;
    Ok(())
  }

  fn pick_passenger(&mut self, lake: &RefCell<Lake>) -> Option<u32> {
    log!("Obteniendo fifo", &LogSeverity::DEBUG);
    let pipe_reader = lake.borrow_mut().get_board_pipe_reader(self.destination);
    match pipe_reader {
      Ok(reader) => {
        let parsed_data = self.parse_passenger(reader);
        if let Some(passenger) =  parsed_data {
          self.passenger_vec.push(passenger);
        }
        parsed_data
      }
      Err(e) => {
        let msg = format!("Error al esperar pasajero en el puerto {}: {:?}",
          self.destination, e);
        log!(msg.as_str(), &LogSeverity::ERROR);
        None
      }
    };
    if self.sigusr_handler.borrow().get_handled() {
      self.status = Status::Disembark;
      self.sigusr_handler.borrow_mut().reset();
    }
    let msg = format!("Hay lugar para {:?} pasajeros",
      self.current_capacity);
    log!(msg.as_str(), &LogSeverity::DEBUG);
    None
  }

  fn read_passenger_reply(&self, lake: &RefCell<Lake>) -> io::Result<Option<u32>>{
    let reader = lake.borrow_mut().get_confirmation_pipe_reader(self.destination)?;
    let mut buf_line = String::new();
    let mut buf_reader = BufReader::new(reader);
    buf_reader.read_line(&mut buf_line)?;
    let msg = format!("Notificando pasajero, leido {:?}.", buf_line);
    log!(msg.as_str(), &LogSeverity::DEBUG);
    buf_line.pop();
    let reply = buf_line.parse::<u32>().expect(&format!("Error al parsear {:?}", buf_line.as_str()));
    if reply == 0 {
      Ok(None)
    } else {
      Ok(Some(reply))
    }
  }

  fn parse_passenger(&mut self, reader: named_pipe::NamedPipeReader) -> Option<u32> {
    let mut buf_line = String::new();
    let mut buf_reader = BufReader::new(reader);
    let bytes_read = buf_reader.read_line(&mut buf_line);
    let msg = format!("Levantando pasajero, leido {:?}.",
      buf_line);
    log!(msg.as_str(), &LogSeverity::DEBUG);
    match bytes_read {
      Ok(0) => None,
      Ok(_) => {
        buf_line.pop();
        let passenger_id = buf_line.parse::<u32>().unwrap();
        let msg = format!("Abordó el pasajero {:?}",
          passenger_id);
        log!(msg.as_str(), &LogSeverity::INFO);
        self.current_capacity -= 1;
        Some(passenger_id)
      },
      Err(e) => {
        log!(format!("{:?}", e).as_str(), &LogSeverity::WARN);
        None
      }
    }
  }
}
