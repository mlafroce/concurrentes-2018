extern crate concurrentes;

use concurrentes::ipc::named_pipe::{NamedPipe, NamedPipeReader, NamedPipeWriter};
use concurrentes::process;
use std::io;
use std::io::{Read, Write};

const PIPE_PATH : &str = "04.fifo";

fn main() -> io::Result<()> {
  NamedPipe::create(PIPE_PATH, 0o644)?;
  // Parent reads shared memory after child writes
  let fork_result = process::fork()?;
  return match fork_result {
    process::ForkResult::Parent{child} => {
      println!("Parent process of {:?}", child);
      let mut read_pipe = NamedPipeReader::open(PIPE_PATH)?;
      process::waitpid(child).expect(format!("Error while waiting {}", child).as_str());
      let mut buf = String::new();
      read_pipe.read_to_string(&mut buf)?;
      println!("Child joined");
      println!("Parent read {}", buf);
      NamedPipe::unlink(PIPE_PATH)
    },
    process::ForkResult::Child => {
      println!("Child process");
      let mut write_pipe = NamedPipeWriter::open(PIPE_PATH)?;
      write_pipe.write_all(b"Hi! I'm child process")
    }
  }
  
}
