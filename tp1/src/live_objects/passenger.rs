use rand;
use rand::Rng;

use concurrentes::ipc;
use concurrentes::ipc::Key;
use concurrentes::ipc::named_pipe;
use concurrentes::ipc::flock::FileLock;
use concurrentes::ipc::semaphore::Semaphore;

use live_objects::lake::Lake;
use live_objects::live_object::LiveObject;

use std::cell::RefCell;
use std::io;
use std::io::{Write, BufRead, BufReader};
use std::process;
use std::time::Duration;
use std::thread::sleep;

use std::ops::Drop;

use concurrentes::log::{GLOBAL_LOG, LogSeverity};

pub struct Passenger {
  destination: u32,
  current_port: u32,
  id: u32,
  status: Status,
  sem: Semaphore
}

#[derive(Debug)]
enum Status {
  WaitShip,
  WaitDestination,
  AskDestination,
  AtDestination,
  Arrive
}

impl LiveObject for Passenger {
  fn tick(&mut self, lake: &RefCell<Lake>) -> io::Result<()> {
    match self.status {
      Status::WaitShip => self.take_ship(lake)?,
      Status::WaitDestination => self.wait_for_destination()?,
      Status::AskDestination => self.ask_destination()?,
      Status::AtDestination => self.at_destination(lake)?,
      Status::Arrive => self.arrive(lake)?
    }
    Ok(())
  }
}

impl Passenger {
  pub fn new(current_port: u32, destination: u32) -> Passenger {
    let id = process::id();
    let flags = ipc::IPC_CREAT | ipc::IPC_EXCL | 0o660;
    let pipe_path = format!("passenger-{:?}.fifo", id);
    let lock_pipe_path = format!("passenger-{:?}.fifo.lock", id);
    named_pipe::NamedPipe::create(pipe_path.as_str(), flags).unwrap();
    FileLock::create(lock_pipe_path.clone()).unwrap();
    let key = Key::ftok(&lock_pipe_path, 0).unwrap();
    let sem = Semaphore::get(&key, flags).unwrap();
    let status = Status::WaitShip;
    let msg = format!("Pasajero {}: desde el puerto {} a {}", id, current_port, destination);
        log!(msg.as_str(), &LogSeverity::INFO);
    Passenger {current_port, destination, id, status, sem}
  }

  fn wait_for_destination(&mut self) -> io::Result<()>{
    let msg = format!("Esperando a llegar a destino {}",
      self.destination);
    log!(msg.as_str(), &LogSeverity::INFO);
    // Acá meto un semáforo porque sino tendría que cambiar todos los open
    self.sem.wait()?;
    self.status = Status::AskDestination;
    Ok(())
  }
  
  fn ask_destination(&mut self) -> io::Result<()>{
    let pipe_path = format!("passenger-{:?}.fifo", self.id);
    log!(format!("Abriendo FIFO {} para saber a que puerto llegué", pipe_path).as_str(), &LogSeverity::DEBUG);
    let reader = named_pipe::NamedPipeReader::open(pipe_path.as_str())?;
    log!("FIFO Abierto", &LogSeverity::DEBUG);
    self.current_port = self.read_current_port(reader)?;
    self.status = Status::AtDestination;
    Ok(())
  }

  fn at_destination (&mut self, lake: &RefCell<Lake>) -> io::Result<()>{
    log!(format!("Avisandole al barco si me bajo o no").as_str(), &LogSeverity::DEBUG);
    let mut writer = lake.borrow_mut().
      get_confirmation_pipe_writer(self.current_port)?;
    if self.current_port == self.destination {
      log!(format!("Llegó a destino").as_str(), &LogSeverity::DEBUG);
      write!(writer, "{}\n", self.id)?;
      self.status = Status::Arrive
    } else {
      log!(format!("Sigue esperando").as_str(), &LogSeverity::DEBUG);
      write!(writer, "0\n")?;
      self.status = Status::WaitDestination;
    }
    Ok(())
  }

  fn take_ship(&mut self, lake: &RefCell<Lake>) -> io::Result<()>{
    let msg = format!("Tomando el barco en el puerto {}, destino {}",
      self.current_port, self.destination);
    log!(msg.as_str(), &LogSeverity::INFO);
    // Mover esto al TDA fifo?
    let lock_pipe_path = format!("port-{:?}-board.fifo.lock", self.current_port);
    // En cierta forma el lock es un molinete :D
    let mut lock = FileLock::create(lock_pipe_path)?;
    lock.lock_exclusive()?;
    log!("Obteniendo fifo", &LogSeverity::DEBUG);
    let mut writer = lake.borrow_mut().
      get_board_pipe_writer(self.current_port)?;
    log!("Obtenido fifo", &LogSeverity::DEBUG);
    write!(writer, "{}\n", self.id)?;
    let writer_msg = format!("Datos enviados: {}", self.id.to_string());
    log!(writer_msg.as_str(), &LogSeverity::DEBUG);
    lock.unlock()?;
    self.status = Status::WaitDestination;
    Ok(())
  }

  fn read_current_port(&mut self, reader: named_pipe::NamedPipeReader) -> io::Result<(u32)> {
    let mut buf_line = String::new();
    let mut buf_reader = BufReader::new(reader);
    log!(format!("Leyendo puerto con buffer").as_str(), &LogSeverity::DEBUG);
    let mut read_port = false;
    while !read_port {
      buf_reader.read_line(&mut buf_line)?;
      buf_line.pop();
      log!(format!("Leido {:?}.", buf_line).as_str(), &LogSeverity::DEBUG);
      read_port = !buf_line.is_empty();
      if !read_port {
        let travel_time = Duration::from_millis(1000);
        sleep(travel_time);
      }
    }
    let port_id = buf_line.parse::<u32>().expect("Error al leer el puerto actual");
    let msg = format!("Llega al destino {:?}",
      port_id);
    log!(msg.as_str(), &LogSeverity::INFO);
    Ok(port_id)
  }

  fn arrive(&mut self, lake: &RefCell<Lake>) -> io::Result<()> {
    let mut rng = rand::thread_rng();
    let msecs = rng.gen::<u32>() % 1000;
    let travel_time = Duration::from_millis(u64::from(msecs));
    let msg = format!("Pasea {} msecs", msecs);
    log!(msg.as_str(), &LogSeverity::INFO);
    sleep(travel_time);
    self.destination = lake.borrow().get_random_port();
    self.status = Status::WaitShip;
    Ok(())
  }
}

impl Drop for Passenger {
  fn drop(&mut self) {
    let pipe_path = format!("passenger-{:?}.fifo", self.id);
    let lock_pipe_path = format!("passenger-{:?}.fifo.lock", self.id);
    self.sem.remove();
    named_pipe::NamedPipe::unlink(pipe_path.as_str()).unwrap();
    named_pipe::NamedPipe::unlink(lock_pipe_path.as_str()).unwrap();
  }
}
