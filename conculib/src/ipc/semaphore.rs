use libc;
use libc::sembuf;
use std::io;
use ipc::{IPC_RMID, SETVAL, SEM_UNDO};
use ipc::key::Key;
use std::mem;
use std::ptr;
use libc::c_void;

pub struct Semaphore {
  id: i32,
}

impl Semaphore {
  pub fn get(key: &Key, init_value: i32, flags: i32) -> io::Result<Semaphore> {
    let id;
    unsafe {
      id = libc::semget(key.key, 1, flags);
      if id == -1 {
        Err(io::Error::last_os_error())
      } else {
        let sem = Semaphore{id};
        sem.init(init_value);
        Ok(sem)
      }
    }
  }

  unsafe fn init(&self, init_value: i32) -> io::Result<()> {
    let mut buf = sembuf {
      sem_num: 0,
      sem_op: init_value as libc::c_short,
      sem_flg: 0
    };
    let result = libc::semctl(self.id, 0, SETVAL, buf);
    if result != -1 {
      Ok(())
    } else {
      Err(io::Error::last_os_error())
    }
  }

  pub fn wait(&self) -> io::Result<()> {
    unsafe {
      self.modify(-1)
    }
  }

  pub fn signal(&self) -> io::Result<()> {
    unsafe {
      self.modify(1)
    }
  }

  unsafe fn modify(&self, value: i32) -> io::Result<()> {
    let mut buf = sembuf {
      sem_num: 0,
      sem_op: value as libc::c_short,
      sem_flg: SEM_UNDO as i16
    };
    let result = libc::semop(self.id, &mut buf, 1);
    if result != -1 {
      Ok(())
    } else {
      Err(io::Error::last_os_error())
    }
  }

  pub fn remove(&mut self) {
    unsafe {
      libc::semctl(self.id, 0, IPC_RMID);
    }
  }
}




