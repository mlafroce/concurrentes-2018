use live_objects::{live_object, ship::Ship, passenger::Passenger};

use std::io;
use std::io::BufRead;

pub struct Launcher {}

#[derive(Copy, Clone, Debug)]
pub enum PromptSelection {
  Ship,
  Passenger,
  Exit
}

impl Launcher {
  pub fn prompt() -> Option<PromptSelection> {
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

  pub fn launch(runner: &mut live_object::LiveObjectRunner,
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
}