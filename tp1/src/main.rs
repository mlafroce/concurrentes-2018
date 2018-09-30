#[macro_use(log)]
extern crate concurrentes;
extern crate libc;
extern crate rand;

mod handlers;
mod live_objects;
mod misc;

use concurrentes::process;
use concurrentes::log::{GLOBAL_LOG, Log, LogSeverity};
use concurrentes::signal::SignalHandlerDispatcher;

use handlers::signal_handler::QuitHandler;

use live_objects::{live_object, ship::Ship, passenger::Passenger};

use std::cell::RefCell;
use std::str::FromStr;
use std::io;
use std::io::BufRead;
use std::rc::Rc;

#[derive(Debug)]
enum PromptSelection {
  Ship,
  Passenger,
  Exit
}

fn main() -> io::Result<()> {
  let quit_handler = Rc::new(RefCell::new(QuitHandler::new()));
  let mut exit = false;
  log!("Iniciando", LogSeverity::INFO);
  SignalHandlerDispatcher::register(libc::SIGINT, quit_handler.clone());
  SignalHandlerDispatcher::register(libc::SIGTERM, quit_handler.clone());
  let mut runner = live_object::LiveObjectRunner::new(quit_handler.clone())?;
  let mut child_result = None;
  let mut child_counter = 0;
  while !exit {
    let selection = prompt();
    match selection {
      Some(PromptSelection::Exit) => exit = true,
      Some(value) => {
        let result = process::fork();
        match result {
          Ok(process::ForkResult::Parent{child}) => {
            log!(format!("El hijo {:?} fue lanzado", child).as_str(), LogSeverity::INFO);
            child_counter += 1;
          },
          Ok(process::ForkResult::Child) => {
            child_result = Some(launch(&mut runner, value));
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
    result
  } else {
    for i in 0..child_counter {
      process::waitpid(process::ANY_CHILD);
    }
    runner.exit()
  }
}

fn prompt() -> Option<PromptSelection> {
  println!("Ingrese un tipo de proceso a lanzar");
  println!("1) Barco");
  println!("2) Pasajero");
  println!("3) Salir");
  let stdin = io::stdin();
  if let Some(line) = stdin.lock().lines().next() {
    let value = line.expect("Value failed");
    return match value.as_ref() {
      "1" => Some(PromptSelection::Ship),
      "2" => Some(PromptSelection::Passenger),
      "3" => Some(PromptSelection::Exit),
      _ => None
    };
  }
  None
}

fn launch(runner: &mut live_object::LiveObjectRunner,
    selection: PromptSelection) -> io::Result<()> {
  
  match selection {
    PromptSelection::Ship => {
      let ship = Ship::new(2, 0);
      runner.run(ship)
    },
    PromptSelection::Passenger => {
      let passenger = Passenger::new();
      runner.run(passenger)
    },
    PromptSelection::Exit => unreachable!()
  }
}