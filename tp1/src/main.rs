#[macro_use(log)]
extern crate concurrentes;
extern crate getopts;
extern crate libc;
extern crate ncurses;
extern crate rand;

mod handlers;
mod live_objects;
mod misc;

use concurrentes::process;
use concurrentes::log::{GLOBAL_LOG, LogSeverity};
use concurrentes::signal::SignalHandlerDispatcher;

use handlers::signal_handler::QuitHandler;

use live_objects::live_object;

use misc::launcher::Launcher;
use misc::tui::{Tui, PromptSelection};
use misc::args_parser::ArgsParser;

use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::io;
use std::process::id as pid;
use std::rc::Rc;


fn main() -> io::Result<()> {
  let args: Vec<String> = env::args().collect();
  let handler = ArgsParser::new();
  let quit_handler = Rc::new(RefCell::new(QuitHandler::new()));
  SignalHandlerDispatcher::register(libc::SIGINT, quit_handler.clone());
  SignalHandlerDispatcher::register(libc::SIGTERM, quit_handler.clone());
  // Si las opciones son válidas, inicio el programa. Si hubo pedido de
  // ayuda o error de sintaxis, no hago nada.
  if let Some(options) = handler.handle(args) {
    run(quit_handler, options)?;
  } 
  
  Ok(())
}


fn run(quit_handler: Rc<RefCell<QuitHandler>>,
    options: HashMap<String, i32>) -> io::Result<()> {
  let mut selection_vector = options_as_vector(&options);
  let options_cell = RefCell::new(options);
  // Inicio la interfaz de texto
  let tui = Tui::new(options_cell);
  let mut quit = false;
  let mut child_result = None;
  let mut child_counter = 0;
  // Objeto que se encarga de crear y destruir IPCs
  // También provee a los hijos de acceso a los IPCs creados por el padre.
  let mut runner = live_object::LiveObjectRunner::new(quit_handler.clone())?;
  while !quit {
    // Levanto las opciones pasadas por argumento
    let mut selection = selection_vector.pop();
    if selection.is_none() {
      // Si ya levanté todas, le permito al usuario
      selection = tui.prompt();
    }
    match selection {
      Some(PromptSelection::Exit) => quit = true,
      // Si tengo una opción válida
      Some(value) => {
        let result = process::fork();
        match result {
          Ok(process::ForkResult::Parent{child}) => {
            log!(format!("El hijo {:?} fue lanzado", child).as_str(), &LogSeverity::INFO);
            tui.print_launch(value, child);
            child_counter += 1;
          },
          Ok(process::ForkResult::Child) => {
            quit = true;
            child_result = Some(Launcher::launch(&mut runner, value));
          },
          Err(_e) => {
            child_result = Some(Err(io::Error::last_os_error()));
          }
        }
      },
      None => tui.print_invalid_input()
    }
    quit = quit || quit_handler.borrow().has_graceful_quit();
  }
  // Fin del programa para los procesos hijos.
  if let Some(result) = child_result {
    let msg = format!("El proceso {:?} fue terminó con resultado {:?}", pid(), result);
    log!(msg.as_str(), &LogSeverity::INFO);
    result
  } else {
  // El padre hace join de todos los hijos.
    for _ in 0..child_counter {
      let child_pid = process::waitpid(process::ANY_CHILD)?;
      log!(format!("El hijo {:?} fue unido", child_pid).as_str(), &LogSeverity::INFO);
    }
    log!("Terminando la aplicación", &LogSeverity::INFO);
    runner.exit()
  }
}

// Convierto el mapa a un vector de selecciones
fn options_as_vector(map: &HashMap<String, i32>) -> Vec<PromptSelection> {
  let mut selections = Vec::new();
  for _ in 0..*map.get("ships").unwrap() {
    selections.push(PromptSelection::Ship);
  }
  for _ in 0..*map.get("passengers").unwrap() {
    selections.push(PromptSelection::Passenger);
  }
  for _ in 0..*map.get("travellers").unwrap() {
    selections.push(PromptSelection::Passenger);
  }
  selections
}