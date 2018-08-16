use std::rc::Rc;
use std::cell::RefCell;
use std::mem;
use std::ptr;

use libc::{sigaction, sigaddset, sigemptyset, sighandler_t};
use libc::{c_void};

thread_local! {
  static SIG_REGISTER: RefCell<SignalHandlerDispatcher> = RefCell::new(SignalHandlerDispatcher {
    handlers: vec![]
  });
}

pub trait SignalHandler {
  fn handle(&mut self);
}

struct NullSignalHandler {}

impl SignalHandler for NullSignalHandler {
  fn handle(&mut self) {}
}

pub struct SignalHandlerDispatcher {
  handlers: Vec<Rc<RefCell<SignalHandler>>>
}

impl SignalHandlerDispatcher {

  pub fn register(signum: i32, handler: Rc<RefCell<SignalHandler>>) {
    let _signum = signum as usize;
    SIG_REGISTER.with(|cell| {
      let mut dispatcher = cell.borrow_mut();
      let handler_vec = &mut dispatcher.handlers;
      if handler_vec.len() < _signum {
        let default_handler = Rc::new(RefCell::new(NullSignalHandler{}));
        handler_vec.resize(_signum, default_handler);
      }
      handler_vec[_signum - 1] = handler;
    });
    unsafe {
      let mut sa : sigaction = mem::uninitialized();
      sa.sa_sigaction = SignalHandlerDispatcher::dispatch as *mut c_void as sighandler_t;
      sigemptyset(&mut sa.sa_mask);
      sigaddset(&mut sa.sa_mask, signum);
      sigaction(signum, &sa, ptr::null_mut());
    }
  }

  fn dispatch(signum: i32) {
    SIG_REGISTER.with(|cell| {
      let mut dispatcher = cell.borrow_mut();
      let handler_vec = &mut dispatcher.handlers;
      handler_vec[signum as usize - 1].borrow_mut().handle();
    });
  }
}
