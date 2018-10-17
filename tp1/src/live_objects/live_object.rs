use handlers::signal_handler::QuitHandler;

use live_objects::main_lock::MainLock;
use live_objects::lake::Lake;
use misc::config::Config;

use std::cell::RefCell;
use std::io;
use std::rc::Rc;

const MAIN_LOCK_FILENAME : &str = "tp1.lock";
const MAIN_CONFIG_FILENAME: &str = "config.cfg";

pub trait LiveObject {
  fn tick(&mut self, &RefCell<Lake>) -> Result<(), io::Error>;
}

/// Esta clase se encargaba de, mediante locks y timestamps, crear y destruir
/// los IPCs relacionados a los distintos procesos del TP.
/// En particular, cuando el tp inició, no existía el lanzador de procesos,
/// por lo que se podía ejecutar distintas instancias de los pasajeros y barcos
/// sin necesidad de que el padre sea el mismo. Para garantizar que los IPCs
/// estuvieran siempre disponibles, se armó el esquema de MainLock
/// Con la aparición del launcher esta clase pasó simplemente a proveer
/// acceso a los ipcs ya existentes.
/// También chequea el tiempo de modificación del lock contra el del archivo
/// de configuración, de forma que no se pueda cambiar la configuración si
/// hay instancias del lago corriendo.
pub struct LiveObjectRunner {
  quit_handler: Rc<RefCell<QuitHandler>>,
  lake: RefCell<Lake>
}

impl LiveObjectRunner {
  /// 
  pub fn new(quit_handler: Rc<RefCell<QuitHandler>>) -> io::Result<(LiveObjectRunner)> {
    
    // Lock principal
    let mut main_lock = MainLock::new(MAIN_LOCK_FILENAME)?;
    main_lock.lock.lock_exclusive()?;
    // Levanto información
    let mut lock_info = main_lock.get_info();
    let lake_config = Config::new(MAIN_CONFIG_FILENAME, &lock_info)?;

    let mut lake = Lake::new(&lake_config);
    // Si soy el primer proceso, creo los IPCS
    if lock_info.is_counter_zero() {
      lake.create_ipcs()?; 
    }
    // Marco que hay un proceso más usando los IPCs
    lock_info.counter_inc();
    lock_info.save(MAIN_LOCK_FILENAME)?;
    main_lock.lock.unlock()?;
    Ok(LiveObjectRunner{quit_handler, lake: RefCell::new(lake)})
  }

  /// Main loop
  pub fn run<T: LiveObject>(&self, mut object: T) -> io::Result<()> {
    // Start object
    while !self.quit_handler.borrow().has_graceful_quit() {
      object.tick(&self.lake)?;
    }
    Ok(())
  }
  
  /// La contraparte de new. Levanta información del archivo MainLock para 
  /// saber cuantos procesos hay abiertos. Si es el último proceso, elimina
  /// todos los IPCs
  pub fn exit(&self) -> io::Result<()> { 
    // Abro el archivo lock
    let mut main_lock = MainLock::new(MAIN_LOCK_FILENAME)?;
    main_lock.lock.lock_exclusive()?;
    let mut lock_info = main_lock.get_info();
    lock_info.counter_dec();
    // Guardo que estoy cerrando el proceso
    lock_info.save(MAIN_LOCK_FILENAME)?;
    // Si soy el último, elimino IPCs
    if lock_info.is_counter_zero() {
      self.lake.borrow_mut().destroy()?;
    }
    main_lock.lock.unlock()?;
    // Exit
    Ok(())
  }

  /// Obtiene un número de puerto aleatorio
  pub fn get_random_port(&self) -> i32 {
    self.lake.borrow().get_random_port()
  }
}
