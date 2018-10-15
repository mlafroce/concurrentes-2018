use std::collections::HashMap;
use getopts::Options;

/// Parser de argumentos
///
/// Interpreta los argumentos con los que es invocada la aplicación.
/// Permite asignar la cantidad inicial de barcos, pasajeros y viajantes.
/// Utiliza un wrapper de getopt (biblioteca de C)
pub struct ArgsParser {
  opts: Options
}

impl ArgsParser {
  /// Crea una instancia del parser.
  pub fn new() -> ArgsParser {
    let mut opts = Options::new();

    opts.optopt("s", "ships", "set initial ships", "<num>");
    opts.optopt("p", "passenger", "set initial passengers", "<num>");
    opts.optopt("t", "traveller", "set initial travellers", "<num>");
    opts.optflag("h", "help", "print this help menu");

    ArgsParser {opts}
  }

  /// Toma un vector de argumentos y extrae de él la configuración inicial
  /// de la aplicación
  pub fn handle(self, args: Vec<String>) -> Option<HashMap<String, i32>> {
    let program = args[0].clone();
    let matches = match self.opts.parse(&args[1..]) {
      Ok(m) => { m }
      Err(_) => {
        self.print_help(&program);
        return None
      }
    };
    if matches.opt_present("h") {
      self.print_help(&program);
      None
    } else {
      let mut map = HashMap::new();
      let num_ships = matches.opt_get_default("s", 0).expect("Invalid ship number");
      let num_passengers = matches.opt_get_default("p", 0).expect("Invalid passenger number");
      let num_travellers = matches.opt_get_default("t", 0).expect("Invalid travellers number");
      map.insert(String::from("ships"), num_ships);
      map.insert(String::from("passengers"), num_passengers);
      map.insert(String::from("travellers"), num_travellers);
      Some(map)
    }
  }

  /// Escribe por salida standard la sintaxis esperada de los argumentos
  pub fn print_help(self, program: &str) {
    let brief = format!("Usage: {} FILE [options]", program);
    let usage = self.opts.usage(&brief);
    println!("{}", usage);
  }
}
