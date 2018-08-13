extern crate libc;

use std::ptr;
use libc::{fork, waitpid};
use std::{thread, time};

fn main() {
  let pid;
  unsafe {
    pid = fork();
    if pid == 0 {
      println!("Child process");
      let millis = time::Duration::from_millis(500);

      thread::sleep(millis);
    } else if pid > 0 {
      println!("Parent process");
      println!("Waiting for {:?}", pid);
      waitpid(pid, ptr::null_mut(), 0);
      println!("joined");
    } else {
      unimplemented!();
    }
  }
}
