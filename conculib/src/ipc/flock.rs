use std::os::unix::io::AsRawFd;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Error;
use libc::flock as c_flock;
use ipc;


pub struct FileLock {
  pub file: File
}

impl FileLock {
  pub fn create(path: &str) -> Result<FileLock, Error> {
    let file = OpenOptions::new().read(true).write(true).create(true).open(path)?;
    Ok(FileLock {file})
  }

  pub fn lock_exclusive(&mut self) -> Result<(), Error> {
    self.flock(ipc::LOCK_EX)
  }

  pub fn lock_shared(&mut self) -> Result<(), Error> {
    self.flock(ipc::LOCK_SH)
  }

  pub fn unlock(&mut self) -> Result<(), Error> {
    self.flock(ipc::LOCK_UN)
  }

  fn flock(&mut self, operation: i32) -> Result<(), Error> {
    let fd = self.file.as_raw_fd();
    let result;
    unsafe {
      result = c_flock(fd, operation);
    }
    if result == 0 {
      Ok(())
    } else {
      Err(Error::last_os_error())
    }
  }
}