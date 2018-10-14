use std::fs::{File, remove_file};
use std::fs::OpenOptions;
use std::io;
use std::io::Error;
use std::os::unix::io::AsRawFd;
use libc;
use ipc;

/*
pub struct FileLockGuard<'a> {
  lock: &'a mut FileLock
}*/

/// Wrapper para crear y utilizar FileLocks, estructura utilizada para sincronizar procesos
/// mediante el acceso exclusivo / compartido a un archivo.
///
/// Para aplicar los locks se utiliza la función de la biblioteca libc fcntl().
///
/// # Atributos
///
/// * `file`: Estructura nativa de rust para trabajar con archivos. Posee el método `as_raw_fd`
/// para acceder a su file descriptor (necesario para llamadas a la biblioteca libc)
///
/// * `path`: Ruta correspondiente al FileLock
pub struct FileLock {
  pub file: File,
  pub path: String
}

impl FileLock {
  /// Crea un archivo y devuelve el FileLock correspondiente
  pub fn new_with_options(path: String, options: &OpenOptions)  -> io::Result<FileLock> {
    let file = options.open(path.as_str())?;
    Ok(FileLock {file, path})
  }

  /// Abre, o crea si no existe un archivo, en modo lectura/escritura.
  pub fn create(path: String) -> io::Result<FileLock> {
    let file = OpenOptions::new().read(true).write(true).create(true).open(path.as_str())?;
    Ok(FileLock {file, path})
  }

  /// Aplica un lock exclusivo sobre todo el archivo
  pub fn lock_exclusive(&mut self) -> io::Result<()> {
    self.flock(ipc::F_WRLCK)
  }

  /// Aplica un lock compartido sobre todo el archivo
  pub fn lock_shared(&mut self) -> io::Result<()> {
    self.flock(ipc::F_RDLCK)
  }

  /// Quita el lock aplicado con `lock_shared` o `lock_exclusive` archivo
  pub fn unlock(&mut self) -> io::Result<()> {
    self.flock(ipc::F_UNLCK)
  }

  /// Llamada a la función fcntl de la biblioteca libc
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

  /// Elimina el archivo asociado al lock.
  pub fn destroy(&mut self) -> io::Result<()> {
    remove_file(self.path.as_str())
  }
}

/* TODO? Guards RAII
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
