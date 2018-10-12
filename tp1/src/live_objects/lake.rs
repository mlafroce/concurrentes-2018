use misc::config::Config;

use rand;
use rand::Rng;

use concurrentes::ipc::flock::FileLock;
use concurrentes::ipc::named_pipe;
use std::io;
use std::io::Error;

const NUM_PORTS_PARAM: &str = "lake ports";

pub struct Lake {
  lake_ports: Vec<FileLock>,
  boarding_pipes: Vec<String>,
  boarding_locks: Vec<String>,
  confirmation_pipes: Vec<String>,
}

impl Lake {
  pub fn init(config: &Config) -> Lake {
    let num_ports_str = config.get(NUM_PORTS_PARAM).expect("Lake ports missing");
    let num_ports = num_ports_str.parse::<u32>().expect("Lake ports invalid");
    let mut lake_ports = Vec::new();
    let mut boarding_pipes = Vec::new();
    let mut boarding_locks = Vec::new();
    let mut confirmation_pipes = Vec::new();
    for port in 0..num_ports {
      let port_lock_path = format!("port-{:?}.lock", port);
      let lock = FileLock::create(port_lock_path).unwrap();
      lake_ports.push(lock);
      let boarding_pipe_path = format!("port-{:?}-board.fifo", port);
      let boarding_lock_path = format!("port-{:?}-board.fifo.lock", port);
      let confirmation_pipe_path = format!("port-{:?}-confirm.fifo", port);
      named_pipe::NamedPipe::create(boarding_pipe_path.as_str(), 0o0644).expect("Failed to create pipe");
      named_pipe::NamedPipe::create(confirmation_pipe_path.as_str(), 0o0644).expect("Failed to create pipe");
      boarding_pipes.push(boarding_pipe_path);
      boarding_pipes.push(boarding_lock_path);
      confirmation_pipes.push(confirmation_pipe_path);
    }
    Lake {lake_ports, boarding_pipes, boarding_locks, confirmation_pipes}
  }

  pub fn load(config: &Config) -> Lake {
    let num_ports_str = config.get(NUM_PORTS_PARAM).expect("Lake ports missing");
    let num_ports = num_ports_str.parse::<u32>().expect("Lake ports invalid");
    let mut lake_ports = Vec::new();
    let mut boarding_pipes = Vec::new();
    let mut boarding_locks = Vec::new();
    let mut confirmation_pipes = Vec::new();
    for port in 0..num_ports {
      let port_lock_path = format!("port-{:?}.lock", port);
      let lock = FileLock::create(port_lock_path).unwrap();
      lake_ports.push(lock);
      let boarding_pipe_path = format!("port-{:?}-board.fifo", port);
      let boarding_lock_path = format!("port-{:?}-board.fifo.lock", port);
      let confirmation_pipe_path = format!("port-{:?}-confirm.fifo", port);
      boarding_pipes.push(boarding_pipe_path);
      boarding_locks.push(boarding_lock_path);
      confirmation_pipes.push(confirmation_pipe_path);
    }
    Lake {lake_ports, boarding_pipes, boarding_locks, confirmation_pipes}
  }

  pub fn destroy(&mut self) -> io::Result<()> {
    println!("Lake destroy!");
    for mut port in &mut self.lake_ports {
      port.destroy()?;
    }
    for pipe in &self.boarding_pipes {
      named_pipe::NamedPipe::unlink(pipe.as_str())?;
    }
    for pipe in &self.confirmation_pipes {
      named_pipe::NamedPipe::unlink(pipe.as_str())?;
    }
    Ok(())
  }

  pub fn get_board_pipe_reader(&mut self, current_port: u32)
    -> io::Result<named_pipe::NamedPipeReader> {
    let board_pipe_path = format!("port-{:?}-board.fifo", current_port);
    named_pipe::NamedPipeReader::open(board_pipe_path.as_str())
  }

  pub fn get_board_pipe_writer(&mut self, current_port: u32)
    -> io::Result<named_pipe::NamedPipeWriter> {
    let board_pipe_path = format!("port-{:?}-board.fifo", current_port);
    named_pipe::NamedPipeWriter::open(board_pipe_path.as_str())
  }

  pub fn get_confirmation_pipe_reader(&mut self, current_port: u32)
    -> io::Result<named_pipe::NamedPipeReader> {
    let confirm_pipe_path = format!("port-{:?}-confirm.fifo", current_port);
    named_pipe::NamedPipeReader::open(confirm_pipe_path.as_str())
  }

  pub fn get_confirmation_pipe_writer(&mut self, current_port: u32)
    -> io::Result<named_pipe::NamedPipeWriter> {
    let confirm_pipe_path = format!("port-{:?}-confirm.fifo", current_port);
    named_pipe::NamedPipeWriter::open(confirm_pipe_path.as_str())
  }

  pub fn get_next_port(&self, current_port: u32) -> u32{
    let num_ports = self.lake_ports.len();
    (current_port + 1) % num_ports as u32
  }

  pub fn get_random_port(&self) -> u32{
    let mut rng = rand::thread_rng();
    let num_ports = self.lake_ports.len() as u32;
    rng.gen::<u32>() % num_ports
  }

  pub fn lock_port(&mut self, port: u32) -> Result<(), Error> {
    self.lake_ports[port as usize].lock_exclusive()
  }

  pub fn unlock_port(&mut self, port: u32) -> Result<(), Error> {
    self.lake_ports[port as usize].unlock()
  }
}