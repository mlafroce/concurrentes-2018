use libc::pid_t;
use libc::fork as c_fork;
use libc::waitpid as c_waitpid;
use std::io::Error;
use std::ptr;

pub enum ForkResult {
  Parent {child: pid_t},
  Child
}

pub fn fork() -> Result<ForkResult, Error> {
  let pid;
  unsafe {
    pid = c_fork();
  }
  if pid == 0 {
    return Ok(ForkResult::Child);
  } else if pid > 0 {
    return Ok(ForkResult::Parent{child: pid});
  } else {
    return Err(Error::last_os_error());
  }
}

pub fn waitpid(child: pid_t) -> Result<pid_t, Error> {
  let pid;
  unsafe {
    pid = c_waitpid(child, ptr::null_mut(), 0);
  }
  if pid > 0 {
    return Ok(pid);
  } else {
    return Err(Error::last_os_error());
  }
}