use libc::c_void;
use libc::shmget as c_shmget;
use libc::shmctl as c_shmctl;
use libc::shmat as c_shmat;
use libc::shmdt as c_shmdt;
use libc::shmid_ds;
use ipc::IPC_RMID;
use ipc::key::Key;
use std::io::Error;
use std::mem;
use std::ptr;
use std::slice::{from_raw_parts, from_raw_parts_mut};
use std::cell::RefCell;

/// Wrapper para trabajar con memoria compartida
///
/// Posee un id del IPC y un puntero hacia la memoria compartida. La estructura es un template para
/// reservar un buffer del tamaño de `num` unidades de `<T>`
pub struct Shmem<T> {
  id: i32,
  num: usize,
  data: *mut T
}

impl <T> Shmem<T> {
  /// Obtiene un id de memoria compartida correspondiente a la clave `key`
  pub fn get(key: &Key, num: usize, flags: i32) -> Result<Shmem<T>, Error> {
    let id;
    unsafe {
      id = c_shmget(key.key, mem::size_of::<T>() * num, flags);
    }
    if id != -1 {
      Ok(Shmem{id, data: ptr::null_mut(), num})
    } else {
      Err(Error::last_os_error())
    }
  }

  /// Realiza operaciones de control
  pub unsafe fn control(&self, cmd: i32, buf: *mut shmid_ds) -> Result<(), Error> {
    let result;
    result = c_shmctl(self.id, cmd, buf);
    if result != -1 {
      Ok(())
    } else {
      Err(Error::last_os_error())
    }
  }

  /// Adosa la memoria compartida obtenida
  pub fn attach(&mut self, flags:i32) -> Result<(), Error> {
    unsafe {
      self.data = c_shmat(self.id, ptr::null(), flags) as *mut T;
      if self.data as i32 != -1 {
        Ok(())
      } else {
        Err(Error::last_os_error())
      }
    }
  }

  /// Desliga la memoria compartida adosada previamente
  pub fn detach(&mut self) -> Result<(), Error> {
    let result;
    unsafe {
      result = c_shmdt(self.data as *const c_void);
    }
    if result != -1 {
      Ok(())
    } else {
      Err(Error::last_os_error())
    }
  }

  /// Elimina el IPC de memoria compartida
  pub fn destroy(&self) -> Result<(), Error> {
    unsafe {
      self.control(IPC_RMID, ptr::null_mut())
    }
  }

  /// Obtiene una copia del dato al que apunta la memoria compartida
  pub fn get_item(&self, idx: isize) -> &T {
    unsafe {
      &*self.data.offset(idx)
    }
  }

  /// Almacena el dato en memoria compartida
  pub fn set_item(&mut self, idx: isize, data: T) {
    unsafe {
      *self.data.offset(idx) = data;
    }
  }

  /// Obtiene los datos de la memoria compartida en forma de array constante
  pub fn get_array(&self) -> RefCell<&[T]> {
    let slice = unsafe { from_raw_parts(self.data, self.num) };
    RefCell::new(slice)
  }

  /// Obtiene los datos de la memoria compartida en forma de array mutable
  pub fn get_array_mut(&mut self) -> RefCell<&mut [T]> {
    let slice = unsafe { from_raw_parts_mut(self.data, self.num) };
    RefCell::new(slice)
  }
}
