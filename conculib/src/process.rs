use libc::pid_t;
use libc::fork as c_fork;
use libc::waitpid as c_waitpid;
use std::io::Error;
use std::ptr;

pub const ANY_CHILD: pid_t = -1;

/// libc `fork()` result wrapper

pub enum ForkResult {
  Parent {child: pid_t},
  Child
}

/// libc `fork()`
/// * On parent process returns ForkResult::Parent with the new child's pid_t.
/// * On child process returns ForkResult::Child.
/// * On failure returns associated error.

pub fn fork() -> Result<ForkResult, Error> {
  let pid;
  unsafe {
    pid = c_fork();
  }
  if pid == 0 {
    Ok(ForkResult::Child)
  } else if pid > 0 {
    Ok(ForkResult::Parent{child: pid})
  } else {
    Err(Error::last_os_error())
  }
}

/// Espera a que el proceso con pid `child` termine la ejecución, y libera sus recursos.
/// Se puede utilizar `process::ANYCHILD` para esperar a cualquier proceso.
pub fn waitpid(child: pid_t) -> Result<pid_t, Error> {
  let pid;
  unsafe {
    pid = c_waitpid(child, ptr::null_mut(), 0);
  }
  if pid > 0 {
    Ok(pid)
  } else {
    Err(Error::last_os_error())
  }
}