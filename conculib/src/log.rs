use super::ipc::flock::FileLock;

use chrono::Local;

use std::fs::OpenOptions;
use std::io;
use std::process;
use std::io::{Write};
use std::cell::RefCell;
use std::str::FromStr;

/// Severidad del mensaje
///
/// Permite 4 niveles de severidad standard: `Error`, `WARN`, `INFO`, `DEBUG`
#[derive(Debug)]
pub enum LogSeverity {
  ERROR,
  WARN,
  INFO,
  DEBUG
}

/// Log
///
/// Posee un FileLock para sincronizar la escritura entre los distintos procesos
pub struct Log {
  file_lock: FileLock
}

thread_local! {
  /// Instancia única del log accesible en toda la aplicación
  pub static GLOBAL_LOG: RefCell<Log> = RefCell::new(
    Log::create(String::from_str("tp.log").unwrap()).unwrap()
  );
}

/// Recibe un mensaje y severidad del mismo para escribirlos en el log global
///
/// **Importante**: para usar este macro se debe incluir la variable GLOBAL_LOG
#[macro_export]
macro_rules! log {
  ($msg: expr, $severity: expr) => {{
    GLOBAL_LOG.with(|log_cell| {
      let mut log = log_cell.borrow_mut();
      log.log($msg, $severity);
    })
  }}
}

impl Log {
  /// Abre un archivo de log (lo crea si no existe) e instancia un FileLock 
  /// y lo deja listo para escribir en él
  ///
  /// # Argumentos
  ///
  /// * `path`: ruta al archivo sobre el que se escribirá el log
  pub fn create(path: String) -> io::Result<Log> {
    let mut options = OpenOptions::new();
    options.append(true).create(true);
    let file_lock = FileLock::new_with_options(path, &options)?;
    Ok(Log{file_lock})
  }

  /// Recibe un mensaje y un tipo (severidad) de mensaje, y lo escribe en el log compartido
  /// con el siguiente format!
  ///
  /// `<fecha> <process-id> <severidad> - mensaje`
  /// 
  /// Se utiliza un lock exclusivo sincronizar la escritura
  pub fn log(&mut self, message: &str, severity: &LogSeverity) -> io::Result<()>{
    self.file_lock.lock_exclusive()?;
    let date = Local::now();
    let date_str = date.format("%Y-%m-%d %H:%M:%S");
    let id = process::id();
    let fmt_msg = format!("{} [{}] [{:?}] - {}\n", date_str, id, severity, message);
    self.file_lock.file.write_all(fmt_msg.as_bytes())?;
    self.file_lock.unlock()
  }
}
