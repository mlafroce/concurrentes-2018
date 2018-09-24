use libc::ftok as c_ftok;
use libc::key_t;
use std::io::Error;
use std::ffi::CString;

/// System V basic key used for IPC identification
 
pub struct Key {
  pub key: key_t
}

impl Key {
  /// Calls System V `ftok()`
  /// * On success returns `Key` struct with new key.
  /// * On failure returns associated error.
  ///
  /// # Example
  ///
  /// ```rust
  /// const KEY_FILE : &str = "/bin/bash";
  /// ftok(KEY_FILE, 0);
  /// ```
  pub fn ftok(path: &str, proj_id: u8) -> Result<Key, Error> {
    let result;
    let path_wrapper = CString::new(path)?;
    unsafe {
      result = c_ftok(path_wrapper.as_ptr(), i32::from(proj_id));
    }
    if result != -1 {
      Ok(Key{key: result})
    } else {
      Err(Error::last_os_error())
    }
  }
}