use std::io;

use live_objects::{live_object, ship::Ship, passenger::Passenger, inspector::Inspector};

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
        let ship = Ship::new(2, current_port);
        runner.run(ship)
      },
      PromptSelection::Passenger => {
        let destination = runner.get_random_port();
        let current_port = runner.get_random_port();
        let passenger = Passenger::new(current_port, destination);
        runner.run(passenger)
      },
      PromptSelection::Inspector => {
        let current_port = runner.get_random_port();
        let inspector = Inspector::new(current_port, true);
        runner.run(inspector)
      },
      PromptSelection::Navy => {
        let current_port = runner.get_random_port();
        let inspector = Inspector::new(current_port, false);
        runner.run(inspector)
      },
      PromptSelection::Exit => unreachable!()
    }
  }
}
