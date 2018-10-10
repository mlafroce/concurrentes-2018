use std::fs::{File, remove_file};
use std::fs::OpenOptions;
//use std::ops::Drop;
use std::io;
use std::io::Error;
use std::os::unix::io::AsRawFd;
use libc;
use ipc;

/*
pub struct FileLockGuard<'a> {
  lock: &'a mut FileLock
}*/

pub struct FileLock {
  pub file: File,
  pub path: String
}

impl FileLock {
  pub fn new_with_options(path: String, options: &OpenOptions)  -> io::Result<FileLock> {
    let file = options.open(path.as_str())?;
    Ok(FileLock {file, path})
  }

  pub fn create(path: String) -> io::Result<FileLock> {
    let file = OpenOptions::new().read(true).write(true).create(true).open(path.as_str())?;
    Ok(FileLock {file, path})
  }

  pub fn lock_exclusive(&mut self) -> io::Result<()> {
    self.flock(ipc::F_WRLCK)
  }

  pub fn lock_shared(&mut self) -> io::Result<()> {
    self.flock(ipc::F_RDLCK)
  }

  pub fn unlock(&mut self) -> io::Result<()> {
    self.flock(ipc::F_UNLCK)
  }

  fn flock(&mut self, operation: i32) -> io::Result<()> {
    let fd = self.file.as_raw_fd();
    let data = libc::flock{
      l_type: operation as i16, l_whence: libc::SEEK_SET as i16, l_start: 0, l_len: 0, l_pid: 0
    };
    let result;
    unsafe {
      result = libc::fcntl(fd, libc::F_SETLKW, &data);
    }
    if result == 0 {
      Ok(())
    } else {
      Err(Error::last_os_error())
    }
  }

  pub fn destroy(&mut self) -> io::Result<()> {
    println!("Destroy");
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