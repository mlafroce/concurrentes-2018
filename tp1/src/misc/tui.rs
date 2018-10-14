use std::collections::HashMap;
use ncurses;

use std::char::from_u32;
use std::ops::Drop;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Copy, Clone, Debug)]
pub enum PromptSelection {
  Ship,
  Passenger,
  Exit
}

pub struct Tui {
  counters: RefCell<HashMap<String, i32>>,
  closed: bool
}

impl Tui {
  pub fn new(counters: RefCell<HashMap<String, i32>>) -> Tui {
    ncurses::initscr();
    let closed = false;
    Tui { counters, closed }
  }

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

  pub fn print_invalid_input(&self) {
    ncurses::mv(8, 0);
    ncurses::clrtoeol();
    ncurses::printw("El valor ingresado es incorrecto");
    ncurses::refresh();
  }

  pub fn close(&mut self) {
    if !self.closed {
      ncurses::endwin();
    }
    self.closed = true;
  }
}

impl Drop for Tui {
  fn drop(&mut self) {
    if !self.closed {
      ncurses::endwin();
    }
  }
}
