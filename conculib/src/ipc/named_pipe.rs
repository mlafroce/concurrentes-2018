use libc;
use libc::{mode_t, c_int, c_void, O_WRONLY, O_RDONLY};
use std::io;
use std::io::{Error, Write, Read};
use std::ffi::CString;
use std::ops::Drop;

/// Implementación de FIFOs de SystemV 
pub struct NamedPipe {
  pub fd: c_int
}

/// Implementación de wrapper de Fifo especializado en lectura
pub struct NamedPipeReader {
  named_pipe: NamedPipe
}

/// Implementación de wrapper de Fifo especializado en escritura
pub struct NamedPipeWriter {
  named_pipe: NamedPipe
}

impl NamedPipe {
  /// Llama a libc `mkfifo(path, mode)`
  /// * En caso de éxito devuelve un `NamedPipe` con su respectivo fd.
  /// * En caso de error devuelve el error del sistema asociado.
  ///
  /// # Example
  ///
  /// ```rust
  /// use concurrentes::ipc::named_pipe::NamedPipe;
  ///
  /// const NAMED_PIPE_PATH : &str = "/bin/bash";
  /// NamedPipe::create(NAMED_PIPE_PATH, 0666);
  /// ```
  pub fn create(path: &str, mode: i32) -> io::Result<()> {
    let path_wrapper = CString::new(path)?;
    let result;
    unsafe {
      result = libc::mkfifo(path_wrapper.as_ptr(), mode as mode_t);
    }
    if result == 0 {
      Ok(())
    } else {
      Err(Error::last_os_error())
    }
  }

  /// Abre un FIFO ya existente
  pub fn open(path: &str, mode: i32) -> io::Result<NamedPipe> {
    let path_wrapper = CString::new(path)?;
    let fd;
    unsafe {
      fd = libc::open(path_wrapper.as_ptr(), mode);
    }
    if fd != -1 {
      Ok(NamedPipe{fd})
    } else {
      Err(Error::last_os_error())
    }
  }

  /// Cierra el file descriptor asociado al FIFO
  pub fn close(&self) {
    let _result;
    unsafe {
      _result = libc::close(self.fd);
    } 
  }

  /// Elimina el FIFO del sistema
  pub fn unlink(path: &str) -> io::Result<()> {
    let path_wrapper = CString::new(path)?;
    let result;
    unsafe {
      result = libc::unlink(path_wrapper.as_ptr());
    }
    if result == 0 {
      Ok(())
    } else {
      Err(Error::last_os_error())
    }
  }
}

impl NamedPipeWriter {
  /// Abre el FIFO en sólo escritura
  pub fn open(path: &str) -> io::Result<NamedPipeWriter> {
    let named_pipe = NamedPipe::open(path, O_WRONLY)?;
    Ok(NamedPipeWriter{named_pipe})
  }
}

impl Write for NamedPipeWriter {
  /// Utiliza la primitiva de libc `write` para escribir un buffer en el fifo
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    let buf_pointer = &buf[0] as *const u8 as *const c_void;
    let result;
    unsafe {
      result = libc::write(self.named_pipe.fd, buf_pointer, buf.len());
    }
    if result >= 0 {
      Ok(result as usize)
    } else {
      Err(Error::last_os_error())
    }
  }

  /// No hace nada, es requisito del Trait Write
  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

impl NamedPipeReader {
  /// Abre el FIFO en sólo lectura
  pub fn open(path: &str) -> io::Result<NamedPipeReader> {
    let named_pipe = NamedPipe::open(path, O_RDONLY)?;
    Ok(NamedPipeReader{named_pipe})
  }
}

impl Read for NamedPipeReader {
  /// Utiliza la primitiva de libc `read` para leer contenidos del pipe al buffer
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    let buf_pointer = &mut buf[0] as *mut u8 as *mut c_void;
    let result;
    unsafe {
      result = libc::read(self.named_pipe.fd, buf_pointer, buf.len());
    }
    if result >= 0 {
      Ok(result as usize)
    } else {
      Err(Error::last_os_error())
    }
  }
}

impl Drop for NamedPipeWriter {
  /// Destructor: cierra el fifo al salir
  fn drop(&mut self) {
    self.named_pipe.close();
  }
}

impl Drop for NamedPipeReader {
  /// Destructor: cierra el fifo al salir
  fn drop(&mut self) {
    self.named_pipe.close();
  }
}
