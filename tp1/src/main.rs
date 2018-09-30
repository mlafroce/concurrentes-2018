extern crate concurrentes;
extern crate libc;
extern crate rand;

mod handlers;
mod live_objects;
mod misc;

use concurrentes::process;
use concurrentes::signal::SignalHandlerDispatcher;

use handlers::signal_handler::QuitHandler;

use live_objects::{live_object, ship::Ship};

use std::cell::RefCell;
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
  SignalHandlerDispatcher::register(libc::SIGINT, quit_handler.clone());
  SignalHandlerDispatcher::register(libc::SIGTERM, quit_handler.clone());
  let mut runner = live_object::LiveObjectRunner::new(quit_handler.clone())?;
  
  while !exit {
    let selection = prompt();
    match selection {
      Some(PromptSelection::Exit) => exit = true,
      Some(value) => {
        launch(&mut runner, value)?
      }
      None => println!("Linea inválida")
    }
    exit = exit || quit_handler.borrow().has_graceful_quit();
  }

  runner.exit()
}

fn prompt() -> Option<PromptSelection> {
  println!("Ingrese un tipo de proceso a lanzar");
  println!("1) Barco");
  println!("2) Pasajero");
  println!("3) Salir");
  let stdin = io::stdin();
  if let Some(line) = stdin.lock().lines().next() {
    let value = line.expect("Value failed");
    println!("Linea leida {:?}.", value);
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
  /*match result {
    Ok(process::ForkResult::Parent{child}) => {
      println!("Parent process of {:?}", child);
      process::waitpid(child).expect(format!("Error while waiting {}", child).as_str());
      println!("Child joined")
    },
    Ok(process::ForkResult::Child) => {
      println!("Child process");
    },
    Err(err) => {
      println!("{:?}", err.kind());
    }
  }*/
  match selection {
    PromptSelection::Ship => {
      let ship = Ship::new(2, 0);
      runner.run(ship)
    },
    PromptSelection::Passenger => Ok(println!("Elegiste un pasajero")),
    PromptSelection::Exit => unimplemented!()
  }
}