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
  let mut shared_ints = Shmem::<i32>::get(&key, 5, flags)
    .expect("Error creating shared_ints");
  shared_ints.attach(0)?;

  // Parent reads shared memory after child writes
  let result = process::fork()?;
  match result {
    process::ForkResult::Parent{child} => {
      println!("Parent process of {:?}", child);
      process::waitpid(child).expect(format!("Error while waiting {}", child).as_str());
      println!("Child joined");
      println!("Parent read {}", shared_ints.get_item());
      println!("Items: {:?}", shared_ints.get_array());
      shared_ints.detach()?;
      shared_ints.destroy()?;
    },
    process::ForkResult::Child => {
      println!("Child process");
      shared_ints.set_item(42);
      { // Van a otro scope para reducir el scope del borrowing de los miembros
        let mut array_cell = shared_ints.get_array_mut();
        let mut shared_array = array_cell.borrow_mut();
        shared_array[1] = 10;
        shared_array[2] = 20;
        shared_array[3] = 30;
        shared_array[4] = 40;
        println!("Child wrote {}", shared_ints.get_item());
        println!("Item array: {:?}", shared_ints.get_array());
      }
      shared_ints.detach()?;
    }
  }
  Ok(())
}
