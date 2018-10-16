use misc::config::Config;

use rand;
use rand::Rng;

use concurrentes::ipc;
use concurrentes::ipc::flock::FileLock;
use concurrentes::ipc::named_pipe;
use concurrentes::ipc::Key;
use concurrentes::ipc::shmem::Shmem;
use concurrentes::log::{GLOBAL_LOG, LogSeverity};
use std::io;
use std::fs::remove_file;
use std::process;

const NUM_PORTS_PARAM: &str = "lake ports";
const STATUS_FILE: &str = "status.lock";

/// Contenedor de los IPCs fijos del lago
///
/// La clase Lake se encarga de crear (y destruir al cierre) varios IPCs:
///
/// * *lake_ports*: FileLocks que representan un puerto en donde pueden anclar
/// los barcos. Cuando un barco llega, intenta tomar el lock. Si está vacío,
/// el barco puede interactuar con el puerto. Si está tomado debe esperar a que
/// el otro zarpe.
///
/// * *boarding_pipes*: Nombres de los FIFOs correspondientes a cada puerto.
/// Estos pipes sirven para que los **Pasajeros** puedan comunicarle a cada 
/// **Barco** que van a subir a viajar.
///
/// * *boarding_locks*: Nombres de los semáforos con los que se limita a uno la
/// cantidad de pasajeros accediendo al puerto. De esta forma se evita que dos
/// pasajeros o más escriban en el FIFO en simultaneo.
///
/// * *confirmation_pipes*: Nombres de los FIFOs por los cuales un pasajero le
/// dice al barco si se baja o no al llegar a determinado puerto.
/// Este pipe no necesita semáforo que lo proteja ya que el barco le pregunta
/// a cada pasajero *de a uno* si baja o no.
///
/// * *status_mem*: Memoria compartida con los pids de los barcos anclados,
/// necesario para que el inspector sepa a quién inspeccionar
pub struct Lake {
  lake_ports: Vec<FileLock>,
  boarding_locks: Vec<String>,
  boarding_pipes: Vec<String>,
  confirmation_pipes: Vec<String>,
  status_lock: FileLock,
  status_mem: Shmem<u32>
}

impl Lake {
  /// Toma una configuración base y comienza a cargar los nombres de los IPCs
  /// correspondientes a cada puerto.
  /// TODO: mover la creación de FileLocks
  pub fn new(config: &Config) -> Lake {
    log!("Iniciando lago", &LogSeverity::INFO);
    let num_ports_str = config.get(NUM_PORTS_PARAM).expect("Lake ports missing");
    let num_ports = num_ports_str.parse::<u32>().expect("Lake ports invalid");
    let status_lock = FileLock::create(STATUS_FILE.to_string()).unwrap();
    // intento crearla, si fallo intento abrir la existente
    let mut status_mem = get_status_mem(num_ports).unwrap();
    status_mem.attach(0).unwrap();
    let mut lake_ports = Vec::new();
    let mut boarding_pipes = Vec::new();
    let mut boarding_locks = Vec::new();
    let mut confirmation_pipes = Vec::new();
    // Almaceno los nombres de los ipcs a crear
    for port in 0..num_ports {
      let port_lock_path = format!("port-{:?}.lock", port);
      let boarding_pipe_path = format!("port-{:?}-board.fifo", port);
      // Mover esto al TDA fifo?
      let boarding_lock_path = format!("port-{:?}-board.fifo.lock", port);
      let confirmation_pipe_path = format!("port-{:?}-confirm.fifo", port);
      boarding_pipes.push(boarding_pipe_path);
      confirmation_pipes.push(confirmation_pipe_path);
      boarding_locks.push(boarding_lock_path);
      // TODO: mover a create_ipcs
      let port_lock = FileLock::create(port_lock_path).unwrap();
      lake_ports.push(port_lock);
    }
    Lake {lake_ports, boarding_pipes, boarding_locks, confirmation_pipes, status_lock, status_mem}
  }

  /// Crea los IPCs en caso de que no existan
  pub fn create_ipcs(&self) -> io::Result<()> {
    for lock in &self.boarding_locks {
      FileLock::create(lock.to_string())?;
    }
    for pipe in &self.boarding_pipes {
      named_pipe::NamedPipe::create(pipe.as_str(), 0o0644)?;
    }
    for pipe in &self.confirmation_pipes {
      named_pipe::NamedPipe::create(pipe.as_str(), 0o0644)?;
    }
    Ok(())
  }

  /// Destruye los IPCs asociados al lago
  pub fn destroy(&mut self) -> io::Result<()> {
    self.status_mem.destroy()?;
    for mut port in &mut self.lake_ports {
      port.destroy()?;
    }
    for lock in &self.boarding_locks {
      remove_file(lock)?;
    }
    for pipe in &self.boarding_pipes {
      named_pipe::NamedPipe::unlink(pipe.as_str())?;
    }
    for pipe in &self.confirmation_pipes {
      named_pipe::NamedPipe::unlink(pipe.as_str())?;
    }
    self.status_lock.destroy()?;
    Ok(())
  }

  /// Abre y devuelve un FIFO correspondiente al puerto, y especializado
  /// para lectura. Estos FIFOs son usados para que el barco levante pasajeros
  pub fn get_board_pipe_reader(&mut self, current_port: u32)
    -> io::Result<named_pipe::NamedPipeReader> {
    let board_pipe_path = &self.boarding_pipes[current_port as usize];
    named_pipe::NamedPipeReader::open(board_pipe_path.as_str())
  }

  /// Abre y devuelve un FIFO correspondiente al puerto, y especializado
  /// para escritura. Estos FIFOs son usados para que el pasajero se registre
  /// en un barco.
  pub fn get_board_pipe_writer(&mut self, current_port: u32)
    -> io::Result<named_pipe::NamedPipeWriter> {
    let board_pipe_path = &self.boarding_pipes[current_port as usize];
    named_pipe::NamedPipeWriter::open(board_pipe_path.as_str())
  }

  /// Abre y devuelve un FIFO correspondiente al puerto, y especializado
  /// para lectura. Estos FIFOs son usados para que el barco sepa si los
  /// pasajeros descienden.
  pub fn get_confirmation_pipe_reader(&mut self, current_port: u32)
    -> io::Result<named_pipe::NamedPipeReader> {
    let confirm_pipe_path = &self.confirmation_pipes[current_port as usize];
    named_pipe::NamedPipeReader::open(confirm_pipe_path.as_str())
  }

  /// Abre y devuelve un FIFO correspondiente al puerto, y especializado
  /// para escritura. Estos FIFOs son usados para que el pasajero confirme su
  /// descenso de un barco, enviando su pid, o 0 si no se baja.
  pub fn get_confirmation_pipe_writer(&mut self, current_port: u32)
    -> io::Result<named_pipe::NamedPipeWriter> {
    let confirm_pipe_path = &self.confirmation_pipes[current_port as usize];
    named_pipe::NamedPipeWriter::open(confirm_pipe_path.as_str())
  }

  /// Devuelve el puerto siguiente al pasado por parámetro
  pub fn get_next_port(&self, current_port: u32) -> u32{
    let num_ports = self.lake_ports.len();
    (current_port + 1) % num_ports as u32
  }

  /// Devuelve un puerto aleatorio
  pub fn get_random_port(&self) -> u32{
    let mut rng = rand::thread_rng();
    let num_ports = self.lake_ports.len() as u32;
    rng.gen::<u32>() % num_ports
  }

  /// Reserva un puerto, o se bloquea esperando que se libere.
  /// Luego escribe el pid del barco en memoria compartida, para que los
  /// inspectores puedan actuar
  pub fn lock_port(&mut self, port: u32) -> io::Result<()> {
    self.lake_ports[port as usize].lock_exclusive()?;
    self.status_lock.lock_exclusive()?;
    self.status_mem.set_item(port as isize, process::id());
    self.status_lock.unlock()
  }

  /// Elimina el pid de la memoria compartida y libera el puerto.
  pub fn unlock_port(&mut self, port: u32) -> io::Result<()> {
    self.status_lock.lock_exclusive()?;
    self.status_mem.set_item(port as isize, 0);
    self.status_lock.unlock();
    self.lake_ports[port as usize].unlock()
  }

  pub fn get_boarding_lock(&self, port: u32) -> io::Result<FileLock>{
    let boarding_lock_path = self.boarding_locks[port as usize].clone();
    FileLock::create(boarding_lock_path)
  }

  pub fn get_ship_at(&mut self, port: u32) -> Option<u32> {
    self.status_lock.lock_exclusive().unwrap();
    let ship_pid = *self.status_mem.get_item(port as isize);
    self.status_lock.unlock();
    if ship_pid == 0 {
      Some(ship_pid)
    } else {
      None
    }
  }
}

/// Rust exige que tenga la memoria compartida creada. Como puede haberlo
/// creado mi proceso u otro proceso, intento crearlo, y si fallo, intento
/// abrir la memoria existente
fn get_status_mem(num_ports: u32) -> io::Result<Shmem<u32>> {
  let shmem_key = Key::ftok(STATUS_FILE, 0)?;
  let flags = ipc::IPC_CREAT | ipc::IPC_EXCL | 0o660;
  match Shmem::<u32>::get(&shmem_key, num_ports as usize, flags) {
    Ok(shmem) => Ok(shmem),
    // Si falla al crear, intento abrir el que existe
    Err(_) => {
      let flags = 0o660;
      Shmem::<u32>::get(&shmem_key, num_ports as usize, flags)
    }
  }
}
