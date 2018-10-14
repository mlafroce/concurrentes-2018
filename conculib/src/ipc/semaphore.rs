use libc;
use libc::sembuf;
use std::io;
use ipc::{IPC_RMID, SETVAL, SEM_UNDO};
use ipc::key::Key;

/// Wrapper para semáforo SystemV
pub struct Semaphore {
  id: i32,
}

impl Semaphore {
  /// Obtiene un semáforo (array de tamaño 1) según la clave asignada
  pub fn get(key: &Key, flags: i32) -> io::Result<Semaphore> {
    let id;
    unsafe {
      id = libc::semget(key.key, 1, flags);
    }
    if id != -1 {
      Ok(Semaphore{id})
    } else {
      Err(io::Error::last_os_error())
    }
  }

  /// Inicializa un semáforo en el valor pasado por parámetro.
  pub fn init(&self, init_value: i32) -> io::Result<()> {
    let buf = sembuf {
      sem_num: 0,
      sem_op: init_value as libc::c_short,
      sem_flg: 0
    };
    let result;
    unsafe {
      result = libc::semctl(self.id, 0, SETVAL, buf);
    }
    if result != -1 {
      Ok(())
    } else {
      Err(io::Error::last_os_error())
    }
  }

  /// Resta uno al valor del semáforo, y se bloquea si este queda en negativo
  pub fn wait(&self) -> io::Result<()> {
    unsafe {
      self.modify(-1)
    }
  }

  /// Suma uno al valor del semáforo.
  pub fn signal(&self) -> io::Result<()> {
    unsafe {
      self.modify(1)
    }
  }

  /// Llamada a semop para realizar operaciones sobre el semáforo de forma nativa
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

  /// Elimina el IPC del sistema
  pub fn remove(&mut self) {
    unsafe {
      libc::semctl(self.id, 0, IPC_RMID);
    }
  }
}




