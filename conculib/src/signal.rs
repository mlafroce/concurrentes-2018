use std::rc::Rc;
use std::cell::RefCell;
use std::mem;
use std::ptr;

use libc::{sigaction, sigaddset, sigemptyset, sighandler_t};
use libc::{c_void, alarm as c_alarm};

thread_local! {
  /// Vector global con handlers de señales
  static SIG_REGISTER: RefCell<SignalHandlerDispatcher> = RefCell::new(SignalHandlerDispatcher {
    handlers: vec![]
  });
}

/// Crea una alarma con tiempo `secs`. Si `secs` es 0, se desactiva.
/// Si pasan la cantidad de segundos pasada por parámetro, se emite la señal
/// `SIGALRM`
pub fn alarm(secs: u32) {
  unsafe {
    c_alarm(secs);
  }
}

/// Interfaz para todos los manejadores de señales
pub trait SignalHandler {
  fn handle(&mut self);
}

/// Manejador por defector (Nulo)
struct NullSignalHandler {}

impl SignalHandler for NullSignalHandler {
  fn handle(&mut self) {}
}

/// Despachador de manejadores
pub struct SignalHandlerDispatcher {
  /// Vector con las distitnas instancias de SignalHandler
  handlers: Vec<Rc<RefCell<SignalHandler>>>
}


impl SignalHandlerDispatcher {
  /// Registra un manejador de señales
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
      sa.sa_flags = 0;
      sigemptyset(&mut sa.sa_mask);
      sigaddset(&mut sa.sa_mask, signum);
      sigaction(signum, &sa, ptr::null_mut());
    }
  }

  /// Función llamada al recibir una señal. Busca el handler correspondiente
  /// a `signum` y ejecuta `handle()` (método del Trait `SignalHandler`)
  fn dispatch(signum: i32) {
    SIG_REGISTER.with(|cell| {
      let mut dispatcher = cell.borrow_mut();
      let handler_vec = &mut dispatcher.handlers;
      handler_vec[signum as usize - 1].borrow_mut().handle();
    });
  }
}
