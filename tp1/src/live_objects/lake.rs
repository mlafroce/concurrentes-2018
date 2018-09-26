use misc::config::Config;

use concurrentes::ipc::flock::FileLock;
use concurrentes::ipc::named_pipe;
use std::io;
use std::io::Error;

const NUM_PORTS_PARAM: &str = "lake ports";
const PASSENGER_PIPE_PATH: &str = "port-{:?}-board.fifo";

pub struct Lake {
  lake_ports: Vec<FileLock>,
}

impl Lake {
  pub fn init(config: &Config) -> Lake {
    let num_ports_str = config.get(NUM_PORTS_PARAM).expect("Lake ports missing");
    let num_ports = num_ports_str.parse::<u32>().expect("Lake ports invalid");
    let mut lake_ports = Vec::new();
    for port in 0..num_ports {
      let port_lock_path = format!("port-{:?}.lock", port);
      let passenger_pipe_path = format!("port-{:?}-board.fifo", port);
      let lock = FileLock::create(port_lock_path.as_str()).unwrap();
      named_pipe::NamedPipe::create(passenger_pipe_path.as_str(), 0o0644).expect("Failed to create pipes");
      lake_ports.push(lock);
    }
    Lake {lake_ports: lake_ports}
  }

  pub fn load(config: &Config) -> Lake {
    let num_ports_str = config.get(NUM_PORTS_PARAM).expect("Lake ports missing");
    let num_ports = num_ports_str.parse::<u32>().expect("Lake ports invalid");
    let mut lake_ports = Vec::new();
    for port in 0..num_ports {
      let port_lock_path = format!("port-{:?}.lock", port);
      let lock = FileLock::create(port_lock_path.as_str()).unwrap();
      lake_ports.push(lock);
    }
    Lake {lake_ports}
  }

  pub fn destroy(&mut self) -> io::Result<()> {
    println!("Lake destroy!");
    for mut port in &mut self.lake_ports {
      port.destroy()?;
    }
    Ok(())
  }

  pub fn get_passenger_pipe_reader(&mut self, current_port: u32)
    -> io::Result<named_pipe::NamedPipeReader> {
    let passenger_pipe_path = format!("port-{:?}-board.fifo", current_port);
    named_pipe::NamedPipeReader::open(passenger_pipe_path.as_str())
  }

  pub fn get_next_port(&self, current_port: u32) -> u32{
    let num_ports = self.lake_ports.len();
    (current_port + 1) % num_ports as u32
  }

  pub fn lock_port(&mut self, port: u32) -> Result<(), Error> {
    self.lake_ports[port as usize].lock_exclusive()
  }

  pub fn unlock_port(&mut self, port: u32) -> Result<(), Error> {
    self.lake_ports[port as usize].unlock()
  }
}