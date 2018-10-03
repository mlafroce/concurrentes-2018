use libc;
use libc::{mode_t, c_int, c_void, O_WRONLY, O_RDONLY};
use std::io;
use std::io::{Error, Write, Read};
use std::ffi::CString;
use std::ops::Drop;

/// System V basic key used for IPC identification
 
pub struct NamedPipe {
  pub fd: c_int
}

pub struct NamedPipeReader {
  named_pipe: NamedPipe
}

pub struct NamedPipeWriter {
  named_pipe: NamedPipe
}

impl NamedPipe {
  /// Calls libc `mkfifo(path, mode)`
  /// * On success returns `NamedPipe` struct with fd.
  /// * On failure returns associated error.
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

  pub fn close(&self) {
    let _result;
    unsafe {
      _result = libc::close(self.fd);
    } 
  }

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
  pub fn open(path: &str) -> io::Result<NamedPipeWriter> {
    let named_pipe = NamedPipe::open(path, O_WRONLY)?;
    Ok(NamedPipeWriter{named_pipe})
  }
}

impl Write for NamedPipeWriter {
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

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

impl NamedPipeReader {
  pub fn open(path: &str) -> io::Result<NamedPipeReader> {
    let named_pipe = NamedPipe::open(path, O_RDONLY)?;
    Ok(NamedPipeReader{named_pipe})
  }
}

impl Read for NamedPipeReader {
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
  fn drop(&mut self) {
    self.named_pipe.close();
  }
}

impl Drop for NamedPipeReader {
  fn drop(&mut self) {
    self.named_pipe.close();
  }
}
