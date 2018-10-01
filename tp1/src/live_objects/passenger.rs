use concurrentes::ipc::named_pipe;

use live_objects::lake::Lake;
use live_objects::live_object::LiveObject;

use std::cell::RefCell;
use std::io::{Error, Write, Read, BufRead, BufReader};
use std::process;

use std::ops::Drop;

use concurrentes::log::{GLOBAL_LOG, Log, LogSeverity};

pub struct Passenger {
  destination: u32,
  current_port: u32,
  id: u32
}

impl LiveObject for Passenger {
  fn tick(&mut self, lake: &RefCell<Lake>) -> Result<(), Error> {
    self.take_ship(lake);
    self.wait_for_destination(lake);
    Ok(())
  }
}


impl Passenger {
  pub fn new() -> Passenger {
    let current_port = 0;
    let destination = 2;
    let id = process::id();
    let pipe_path = format!("passenger-{:?}.fifo", id);
    named_pipe::NamedPipe::create(
      pipe_path.as_str(), 0o0644).expect("Failed to create pipe");
    Passenger {current_port, destination, id}
  }

  fn wait_for_destination(&mut self, lake: &RefCell<Lake>) {
    let msg = format!("Esperando a llegar a destino {}",
      self.destination);
    log!(msg.as_str(), LogSeverity::INFO);
    let pipe_path = format!("passenger-{:?}.fifo", self.id);
    if let Ok(reader) = named_pipe::NamedPipeReader::open(pipe_path.as_str()) {
      let mut buf_line = String::new();
      let mut buf_reader = BufReader::new(reader);
      let bytes_read = buf_reader.read_line(&mut buf_line);
    }
  }

  fn take_ship(&mut self, lake: &RefCell<Lake>) {
    let msg = format!("Tomando el barco en el puerto {}, destino {}",
      self.current_port, self.destination);
    log!(msg.as_str(), LogSeverity::INFO);
    let mut writer = lake.borrow_mut().
      get_passenger_pipe_writer(self.current_port).expect("Failed to get pipe");
    log!("Obtenido fifo", LogSeverity::DEBUG);
    writer.write_all(self.id.to_string().as_bytes()).expect("Failed to  write");
    let writer_msg = format!("Datos enviados: {}", self.id.to_string());
    log!(writer_msg.as_str(), LogSeverity::DEBUG);
  }
}

impl Drop for Passenger {
  fn drop(&mut self) {
    let pipe_path = format!("passenger-{:?}.fifo", self.id);
    named_pipe::NamedPipe::unlink(pipe_path.as_str()).unwrap();
  }
}
