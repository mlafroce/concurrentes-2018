extern crate concurrentes;

use concurrentes::ipc;
use concurrentes::ipc::Key;
use concurrentes::ipc::semaphore::Semaphore;
use concurrentes::process;

use std::io;
use std::io::{Read, Write};

use std::{thread, time};

const PIPE_PATH : &str = "04.fifo";

fn main() -> io::Result<()> {
  let key = Key::ftok(file!(), 0)?;
  println!("Key obtained: {}", key.key);
  let flags = ipc::IPC_CREAT | ipc::IPC_EXCL | 0o660;
  let mut semaphore = Semaphore::get(&key, 0, flags).unwrap();

  // Parent reads shared memory after child writes
  let fork_result = process::fork()?;
  return match fork_result {
    process::ForkResult::Parent{child} => {
      println!("Parent process of {:?}", child);
      semaphore.wait();
      println!("Signal received");
      process::waitpid(child).expect(format!("Error while waiting {}", child).as_str());
      semaphore.remove();
      println!("Child joined");
      Ok(())
    },
    process::ForkResult::Child => {
      println!("Child process");
      let millis = time::Duration::from_millis(500);
      thread::sleep(millis);
      println!("Signal send");
      semaphore.signal()
    }
  }
  
}
