#[macro_use(log)]
extern crate concurrentes;
extern crate getopts;
extern crate libc;
extern crate ncurses;
extern crate rand;

/// Clases que representan entidades del lago
pub mod live_objects;
/// Handlers de señales
pub mod handlers;
/// Clases utilitarias
pub mod misc;
