extern crate concurrentes;

use concurrentes::ipc;
use concurrentes::ipc::Key;
use concurrentes::ipc::shmem::Shmem;
use concurrentes::process;
use std::io;

const KEY_FILE : &str = "/bin/bash";

fn main() -> io::Result<()> {
  let key = Key::ftok(KEY_FILE, 0)?;
  println!("Key obtained: {}", key.key);
  let flags = ipc::IPC_CREAT | ipc::IPC_EXCL | 0o660;
  let mut shmem = Shmem::<i32>::get(&key, flags)
    .expect("Error creating shmem");
  shmem.attach(0)?;

  // Parent reads shared memory after child writes
  let result = process::fork()?;
  match result {
    process::ForkResult::Parent{child} => {
      println!("Parent process of {:?}", child);
      process::waitpid(child).expect(format!("Error while waiting {}", child).as_str());
      println!("Child joined");
      println!("Parent read {}", shmem.get_data());
      shmem.detach()?;
      shmem.destroy()?;
    },
    process::ForkResult::Child => {
      println!("Child process");
      shmem.set_data(42);
      println!("Child wrote {}", shmem.get_data());
      shmem.detach()?;
    }
  }
  Ok(())
}
