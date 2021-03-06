use rand;
use rand::Rng;

use concurrentes::ipc;
use concurrentes::ipc::Key;
use concurrentes::ipc::named_pipe;
use concurrentes::ipc::flock::FileLock;
use concurrentes::ipc::semaphore::Semaphore;
use concurrentes::log::{GLOBAL_LOG, LogSeverity};

use live_objects::lake::Lake;
use live_objects::live_object::LiveObject;

use std::cell::RefCell;
use std::io;
use std::io::{Write, BufRead, BufReader};
use std::ops::Drop;
use std::process;
use std::time::Duration;
use std::thread::sleep;


/// Entidad pasajero
///
/// Posee los siguientes destinos:
/// * Un destino, que se elige al azar
/// * Un puerto actual, donde se va a tomar el barco
/// * Un id (el pid)
/// * Estado del pasajero
pub struct Passenger {
  destination: i32,
  current_port: i32,
  id: u32,
  status: Status,
  sem: Semaphore,
  inspection: bool,
  navy: bool
}

#[derive(Debug)]
enum Status {
  /// Esperando a que un barco llegue al puerto para levantarlo
  WaitShip,
  /// Dentro del barco, esperando a que el barco llegue a algún puerto
  WaitDestination,
  /// Esperando a que el barco le diga a qué puerto llegaron
  AskDestination,
  /// Esperando a que el barco escuche si se baja o no
  AtDestination,
  /// "Paseando" por la ciudad
  Arrive
}

impl LiveObject for Passenger {
  fn tick(&mut self, lake: &RefCell<Lake>) -> io::Result<()> {
    match self.status {
      Status::WaitShip => self.take_ship(lake)?,
      Status::WaitDestination => self.wait_for_destination()?,
      Status::AskDestination => self.ask_destination()?,
      Status::AtDestination => self.at_destination(lake)?,
      Status::Arrive => self.arrive(lake)?
    }
    Ok(())
  }
}

impl Passenger {
  pub fn new(current_port: i32, destination: i32) -> Passenger {
    let id = process::id();
    let flags = ipc::IPC_CREAT | ipc::IPC_EXCL | 0o660;
    let pipe_path = format!("passenger-{:?}.fifo", id);
    let lock_pipe_path = format!("passenger-{:?}.fifo.lock", id);
    named_pipe::NamedPipe::create(pipe_path.as_str(), flags).unwrap();
    FileLock::create(lock_pipe_path.clone()).unwrap();
    let key = Key::ftok(&lock_pipe_path, 0).unwrap();
    let sem = Semaphore::get(&key, flags).unwrap();
    let status = Status::WaitShip;
    let msg = format!("Pasajero {}: desde el puerto {} a {}", id, current_port, destination);
        log!(msg.as_str(), &LogSeverity::INFO);
    Passenger {current_port, destination, id, status, sem, inspection: false, navy: false}
  }

  fn wait_for_destination(&mut self) -> io::Result<()>{
    let msg = format!("Esperando a llegar a destino {}",
      self.destination);
    log!(msg.as_str(), &LogSeverity::INFO);
    // Acá meto un semáforo porque sino tendría que cambiar todos los open
    self.sem.wait()?;
    self.status = Status::AskDestination;
    Ok(())
  }
  
  /// Abre un fifo para saber en qué puerto está el barco
  /// Casos especiales: -1 es una inspeccion, -2 es prefectura
  fn ask_destination(&mut self) -> io::Result<()>{
    let pipe_path = format!("passenger-{:?}.fifo", self.id);
    log!(format!("Abriendo FIFO {} para saber a que puerto llegué", pipe_path).as_str(), &LogSeverity::DEBUG);
    let reader = named_pipe::NamedPipeReader::open(pipe_path.as_str())?;
    log!("FIFO Abierto", &LogSeverity::DEBUG);
    let read_port = self.read_current_port(reader)?;
    if read_port == -1 {
      self.inspection = true;
    } else if read_port == -2 {
      self.navy = true;
    } else {
      self.current_port = read_port;
    }
    self.status = Status::AtDestination;
    Ok(())
  }

  /// Acción ejecutada al llegar a un destino. El pasajero abre un FIFO
  /// y le notifica al barco si va a descender, enviándole su pid, o si se
  /// queda, enviando un 0
  fn at_destination (&mut self, lake: &RefCell<Lake>) -> io::Result<()>{
    log!("Avisandole al barco si me bajo o no", &LogSeverity::DEBUG);
    let mut writer = lake.borrow_mut().
      get_confirmation_pipe_writer(self.current_port)?;
    log!("Writer", &LogSeverity::INFO);
    // Caso especial -2: aviso de prefectura
    if self.navy {
      log!("Prefectura me hizo descender", &LogSeverity::DEBUG);
      self.ask_destination()?;
      self.status = Status::WaitShip;
      self.navy = false
    }
    // Caso especial -1: aviso de inspector
    if self.inspection {
      log!("El inspector consulta si tengo el boleto válido", &LogSeverity::DEBUG);
      // El boleto es válido aleatoriamente
      let mut rng = rand::thread_rng();
      let ticket = rng.gen::<u32>() % 10;
      // Si está vencido
      if ticket != 0 {
        log!("Mi boleto está vencido", &LogSeverity::DEBUG);
        writeln!(writer, "{}", self.id)?;
        self.destination = self.current_port;
        lake.borrow_mut().report_passenger();
        self.status = Status::WaitShip;
      } else {
        log!("Mi boleto es válido", &LogSeverity::DEBUG);
        writeln!(writer, "0")?;
        self.status = Status::WaitDestination;
      }
      self.inspection = false
    }
    // Llega a un puerto
    if self.current_port == self.destination {
      log!("Llegó a destino", &LogSeverity::DEBUG);
      writeln!(writer, "{}", self.id)?;
      self.status = Status::Arrive
    } else {
      log!("Sigue esperando", &LogSeverity::DEBUG);
      writeln!(writer, "0")?;
      self.status = Status::WaitDestination;
    }
    Ok(())
  }

  /// El pasajero va a tomar el barco. Para esto abre un FIFO, el de abordo
  /// y le escribe al barco que quiere subir. Si no hay barco, el pasajero se queda
  /// bloqueado
  fn take_ship(&mut self, lake: &RefCell<Lake>) -> io::Result<()>{
    let msg = format!("Tomando el barco en el puerto {}, destino {}",
      self.current_port, self.destination);
    log!(msg.as_str(), &LogSeverity::INFO);
    // En cierta forma el lock es un molinete :D
    let mut lock = lake.borrow_mut().get_boarding_lock(self.current_port)?;
    lock.lock_exclusive()?;
    log!("Obteniendo fifo", &LogSeverity::DEBUG);
    let mut writer = lake.borrow_mut().
      get_board_pipe_writer(self.current_port)?;
    log!("Obtenido fifo", &LogSeverity::DEBUG);
    writeln!(writer, "{}", self.id)?;
    let writer_msg = format!("Datos enviados: {}", self.id.to_string());
    log!(writer_msg.as_str(), &LogSeverity::DEBUG);
    lock.unlock()?;
    self.status = Status::WaitDestination;
    Ok(())
  }

  /// Función auxiliar de `at_destination`, lee y parsea el número de puerto
  /// notificado por el barco al llegar a un puerto.
  fn read_current_port(&mut self, reader: named_pipe::NamedPipeReader) -> io::Result<(i32)> {
    let mut buf_line = String::new();
    let mut buf_reader = BufReader::new(reader);
    log!("Leyendo puerto con buffer", &LogSeverity::DEBUG);
    buf_reader.read_line(&mut buf_line)?;
    buf_line.pop();
    log!(format!("Leido {:?}.", buf_line).as_str(), &LogSeverity::DEBUG);
    let port_id = buf_line.parse::<i32>().expect("Error al leer el puerto actual");
    let msg = format!("Llega al destino {:?}",
      port_id);
    log!(msg.as_str(), &LogSeverity::INFO);
    Ok(port_id)
  }

  /// El pasajero ya descendió y pasea por el pueblo.
  /// Comportamiento default: turista que pasea por puertos aleatorios
  fn arrive(&mut self, lake: &RefCell<Lake>) -> io::Result<()> {
    let mut rng = rand::thread_rng();
    let msecs = rng.gen::<u32>() % 1000;
    let travel_time = Duration::from_millis(u64::from(msecs));
    let msg = format!("Pasea {} msecs", msecs);
    log!(msg.as_str(), &LogSeverity::INFO);
    sleep(travel_time);
    self.destination = lake.borrow().get_random_port();
    self.status = Status::WaitShip;
    Ok(())
  }
}

impl Drop for Passenger {
  fn drop(&mut self) {
    let pipe_path = format!("passenger-{:?}.fifo", self.id);
    let lock_pipe_path = format!("passenger-{:?}.fifo.lock", self.id);
    self.sem.remove();
    named_pipe::NamedPipe::unlink(pipe_path.as_str()).unwrap();
    named_pipe::NamedPipe::unlink(lock_pipe_path.as_str()).unwrap();
  }
}
