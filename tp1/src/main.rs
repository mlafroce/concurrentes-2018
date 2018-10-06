#[macro_use(log)]
extern crate concurrentes;
extern crate libc;
extern crate rand;

mod handlers;
mod live_objects;
mod misc;

use concurrentes::process;
use concurrentes::log::{GLOBAL_LOG, LogSeverity};
use concurrentes::signal::SignalHandlerDispatcher;

use handlers::signal_handler::QuitHandler;

use live_objects::live_object;

use misc::launcher::{Launcher, PromptSelection};

use std::cell::RefCell;
use std::io;
use std::process::id as pid;
use std::rc::Rc;


fn main() -> io::Result<()> {
  let quit_handler = Rc::new(RefCell::new(QuitHandler::new()));
  let mut exit = false;
  log!("Iniciando la aplicación", &LogSeverity::INFO);
  SignalHandlerDispatcher::register(libc::SIGINT, quit_handler.clone());
  SignalHandlerDispatcher::register(libc::SIGTERM, quit_handler.clone());
  let mut child_result = None;
  let mut child_counter = 0;
  let mut runner = live_object::LiveObjectRunner::new(quit_handler.clone())?;
  while !exit {
    let selection = Launcher::prompt();
    match selection {
      Some(PromptSelection::Exit) => exit = true,
      Some(value) => {
        let result = process::fork();
        match result {
          Ok(process::ForkResult::Parent{child}) => {
            log!(format!("El hijo {:?} fue lanzado", child).as_str(), &LogSeverity::INFO);
            child_counter += 1;
          },
          Ok(process::ForkResult::Child) => {
            child_result = Some(Launcher::launch(&mut runner, value));
          },
          Err(_e) => {
            child_result = Some(Err(io::Error::last_os_error()));
          }
        }
      }
      None => println!("Linea inválida")
    }
    exit = exit || quit_handler.borrow().has_graceful_quit();
  }

  if let Some(result) = child_result {
    let msg = format!("El proceso {:?} fue terminó con resultado {:?}", pid(), result);
    log!(msg.as_str(), &LogSeverity::INFO);
    result
  } else {
    for _ in 0..child_counter {
      let child_pid = process::waitpid(process::ANY_CHILD)?;
      log!(format!("El hijo {:?} fue unido", child_pid).as_str(), &LogSeverity::INFO);
    }
    log!("Terminando la aplicación", &LogSeverity::INFO);
    runner.exit()
  }
}
