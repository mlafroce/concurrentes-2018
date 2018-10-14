//! Esta biblioteca contiene algunos wrappers propios a las primitivas de C
//! utilizadas en la materia.
extern crate libc;
extern crate chrono;

/// Contiene algunos de los IPCs utilizados en la materia:
///
/// * FileLocks
/// * Memoria compartida
/// * Semaforos
/// * FIFOs (NamedPipes)
///
/// También posee varias constantes necesarias para interactuar con las primitivas de libc
pub mod ipc;
/// Contiene un wrapper para fork y waitpid
pub mod process;
/// Contiene un handler de señales con un diseño de clases similar al propuesto en la materia
pub mod signal;
/// Contiene un log que utiliza un FileLock para poder ser usado por distintos procesos. También
/// posee un macro para facilitar el formato del log.
pub mod log;
