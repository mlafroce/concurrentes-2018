use std::fs::File;
use std::fs::OpenOptions;
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
  pub file: File
}

impl FileLock {
  pub fn create(path: &str) -> io::Result<FileLock> {
    let file = OpenOptions::new().read(true).write(true).create(true).open(path)?;
    Ok(FileLock {file})
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