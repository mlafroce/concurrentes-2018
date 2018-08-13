extern crate concurrentes;

use concurrentes::process;

fn main() {
  let result = process::fork();
  match result {
    Ok(process::ForkResult::Parent{child}) => {
      println!("Parent process of {:?}", child);
      process::waitpid(child).expect(format!("Error while waiting {}", child).as_str());
      println!("Child joined")
    },
    Ok(process::ForkResult::Child) => {
      println!("Child process");
    },
    Err(err) => {
      println!("{:?}", err.kind());
    }
  }
}