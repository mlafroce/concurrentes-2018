use std::fs::{File, remove_file};
use std::fs::OpenOptions;
use std::str::FromStr;
//use std::ops::Drop;
use std::io;
use std::io::Error;
use std::os::unix::io::AsRawFd;
use libc::flock as c_flock;
use ipc;


pub struct FileLockGuard<'a> {
  lock: &'a mut FileLock
}

pub struct FileLock {
  pub file: File,
  pub path: String
}

impl FileLock {
  pub fn new_with_options(path: String, options: OpenOptions)  -> io::Result<FileLock> {
    let file = options.open(path.as_str())?;
    Ok(FileLock {file, path})
  }

  pub fn create(path: String) -> io::Result<FileLock> {
    let file = OpenOptions::new().read(true).write(true).create(true).open(path.as_str())?;
    Ok(FileLock {file, path})
  }

  pub fn lock_exclusive(&mut self) -> io::Result<()> {
    self.flock(ipc::LOCK_EX)
  }

  pub fn lock_shared(&mut self) -> io::Result<()> {
    self.flock(ipc::LOCK_SH)
  }

  pub fn unlock(&mut self) -> io::Result<()> {
    self.flock(ipc::LOCK_UN)
  }

  fn flock(&mut self, operation: i32) -> io::Result<()> {
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

  pub fn destroy(&mut self) -> io::Result<()> {
    remove_file(self.path.as_str())
  }
}

/* TODO
impl<'a> FileLockGuard<'a> {
  pub fn new_exclusive(lock: Rc<FileLock>) -> io::Result<FileLockGuard> {
    lock.lock_exclusive()?;
    Ok(FileLockGuard{lock})
  }

  pub fn new_shared(lock: Rc<FileLock>) -> io::Result<FileLockGuard> {
    lock.lock_shared()?;
    Ok(FileLockGuard{lock})
  }
}

impl<'a> Drop for FileLockGuard<'a> {
  fn drop(&mut self) {
    self.lock.unlock().unwrap();
  }
}*/