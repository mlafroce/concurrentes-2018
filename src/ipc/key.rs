use libc::ftok as c_ftok;
use libc::key_t;
use std::io::Error;
use std::ffi::CString;

pub struct Key {
  pub key: key_t
}

impl Key {
  pub fn ftok(path: &str, proj_id: u8) -> Result<Key, Error> {
    let result;
    let path_wrapper = CString::new(path)?;
    unsafe {
      result = c_ftok(path_wrapper.as_ptr(), proj_id as i32);
    }
    if result != -1 {
      return Ok(Key{key: result});
    } else {
      return Err(Error::last_os_error());
    }
  }
}