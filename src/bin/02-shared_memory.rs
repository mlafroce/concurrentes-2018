extern crate concurrentes;

use concurrentes::ipc::Key;
use concurrentes::ipc::shmem::Shmem;
use concurrentes::ipc::ipc_base;
use concurrentes::process;

const KEY_FILE : &str = "/bin/bash";

fn main() {
  let key = Key::ftok(KEY_FILE, 0).unwrap();
  println!("Key obtained: {}", key.key);
  let flags = ipc_base::IPC_CREAT | ipc_base::IPC_EXCL;
  let mut shmem = Shmem::<i32>::get(key, flags)
    .expect("Error creating shmem");
  shmem.attach(0).unwrap();

  // Parent reads shared memory after child writes
  let result = process::fork();
  match result {
    Ok(process::ForkResult::Parent{child}) => {
      println!("Parent process of {:?}", child);
      process::waitpid(child).expect(format!("Error while waiting {}", child).as_str());
      println!("Child joined");
      println!("Parent read {}", shmem.get_data());
      shmem.detach().unwrap();
      shmem.destroy().unwrap();
    },
    Ok(process::ForkResult::Child) => {
      println!("Child process");
      shmem.set_data(42);
      println!("Child wrote {}", shmem.get_data());
      shmem.detach().unwrap();
    },
    Err(err) => {
      println!("{:?}", err.kind());
    }
  }
}
