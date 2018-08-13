use libc::shmget as c_shmget;
use libc::shmctl as c_shmctl;
use libc::shmat as c_shmat;
use libc::shmdt as c_shmdt;
use libc::shmid_ds;
use ipc::key::Key;
use ipc::ipc_base::IPC_RMID;
use std::io::Error;
use std::mem;
use std::ptr;
use libc::c_void;

pub struct Shmem<T> {
  id: i32,
  data: *mut T
}

impl <T> Shmem<T> {
  pub fn get(key: Key, flags: i32) -> Result<Shmem<T>, Error> {
    let id;
    unsafe {
      id = c_shmget(key.key, mem::size_of::<T>(), flags);
    }
    if id != -1 {
      return Ok(Shmem{id: id, data: ptr::null_mut()});
    } else {
      return Err(Error::last_os_error());
    }
  }

  pub fn control(&self, cmd: i32, buf: *mut shmid_ds) -> Result<(), Error> {
    let result;
    unsafe {
      result = c_shmctl(self.id, cmd, buf);
    }
    if result != -1 {
      return Ok(());
    } else {
      return Err(Error::last_os_error());
    }
  }

  pub fn attach(&mut self, flags:i32) -> Result<(), Error> {
    unsafe {
      self.data = c_shmat(self.id, ptr::null(), flags) as *mut T;
      if self.data as i32 != -1 {
        return Ok(());
      } else {
        return Err(Error::last_os_error());
      }
    }
  }

  pub fn detach(&mut self) -> Result<(), Error> {
    let result;
    unsafe {
      result = c_shmdt(self.data as *const c_void);
    }
    if result != -1 {
      return Ok(());
    } else {
      return Err(Error::last_os_error());
    }
  }

  pub fn destroy(self) -> Result<(), Error> {
    self.control(IPC_RMID, ptr::null_mut())
  }

  pub fn get_data(&self) -> & T {
    unsafe {
      &*self.data
    }
  }

  pub fn set_data(&mut self, data: T) {
    unsafe {
      *self.data = data;
    }
  }
}
