use std::io;

use live_objects::{live_object, ship::Ship, passenger::Passenger};

use misc::tui::PromptSelection;

/// "Lanzador" de objetos. Ejecuta una de las entidades del lago
/// 
/// Las entidades que puede lanzar son:
/// * Barco
/// * Pasajero
/// * Inspector
pub struct Launcher;

impl Launcher {
  pub fn launch(runner: &mut live_object::LiveObjectRunner,
    selection: PromptSelection) -> io::Result<()> {
    match selection {
      PromptSelection::Ship => {
        let current_port = runner.get_random_port();
        let ship = Ship::new(1, current_port);
        runner.run(ship)
      },
      PromptSelection::Passenger => {
        let destination = runner.get_random_port();
        let current_port = runner.get_random_port();
        let passenger = Passenger::new(current_port, destination);
        runner.run(passenger)
      },
      PromptSelection::Exit => unreachable!()
    }
  }
}
