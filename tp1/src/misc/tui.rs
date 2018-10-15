use std::collections::HashMap;
use ncurses;

use std::char::from_u32;
use std::ops::Drop;
use std::cell::RefCell;

/// Opciones disponibles en la interfaz de usuario
#[derive(Copy, Clone, Debug)]
pub enum PromptSelection {
  Ship,
  Passenger,
  Exit
}

/// Interfaz de texto de usuario, hecha con ncurses
pub struct Tui {
  counters: RefCell<HashMap<String, i32>>,
}

impl Tui {
  /// Inicializa la pantalla de ncurses y asigna el mapa de contadores a la interfaz
  pub fn new(counters: RefCell<HashMap<String, i32>>) -> Tui {
    ncurses::initscr();
    Tui { counters }
  }

  /// Escribe el menú del usuario y pregunta por una opción
  pub fn prompt(&self) -> Option<PromptSelection> {
    ncurses::mvprintw(1, 0, "Ingrese un tipo de proceso a lanzar");
    ncurses::mvprintw(2, 0, "1) Barco");
    ncurses::mvprintw(3, 0, "2) Pasajero");
    ncurses::mvprintw(4, 0, "3) Salir");
    ncurses::mv(6, 0);
    ncurses::refresh();
    let input_raw = ncurses::getch();
    if let Some(input) = from_u32(input_raw as u32) {
      match input {
        '1' => Some(PromptSelection::Ship),
        '2' => Some(PromptSelection::Passenger),
        '3' => Some(PromptSelection::Exit),
        _ => None
      }
    } else {
      None
    }
  }

  /// Informa qué tipo de proceso fue lanzado
  pub fn print_launch(&self, selection: PromptSelection, pid: i32) {
    ncurses::mv(8, 0);
    ncurses::clrtoeol();
    let msg = match selection {
      PromptSelection::Passenger => format!("Lanzado pasajero {}", pid),
      PromptSelection::Ship => format!("Lanzado barco {}", pid),
      PromptSelection::Exit => unreachable!()
    };
    ncurses::printw(msg.as_str());
    ncurses::refresh();
  }

  /// Informa entrada inválida
  pub fn print_invalid_input(&self) {
    ncurses::mv(8, 0);
    ncurses::clrtoeol();
    ncurses::printw("El valor ingresado es incorrecto");
    ncurses::refresh();
  }
}

impl Drop for Tui {
  /// Libera ncurses
  fn drop(&mut self) {
    ncurses::endwin();
  }
}
