use libc;

mod key;
/// Mdulo de memoria compartida
pub mod shmem;
/// Módulo de semáforos
pub mod semaphore;
/// Módulo de Filelocks
pub mod flock;
/// Módulo de FIFOs
pub mod named_pipe;
pub use self::key::Key;

pub const IPC_RMID   : i32 = 0o0000;   /* remove resource */
pub const IPC_SET    : i32 = 0o0001;   /* set ipc_perm options */
pub const IPC_STAT   : i32 = 0o0002;   /* get ipc_perm options */
pub const IPC_INFO   : i32 = 0o0003;   /* see ipcs */

pub const IPC_CREAT  : i32 = 0o1000;   /* create if key is nonexistent */
pub const IPC_EXCL   : i32 = 0o2000;   /* fail if key exists */
pub const IPC_NOWAIT : i32 = 0o4000;   /* return error on wait */

pub const SEM_UNDO     : i32 = 0x1000;
pub const SETVAL     : i32 = 16;

pub const F_RDLCK: i32 = 0; /* Shared lock */
pub const F_WRLCK: i32 = 1; /* Exclusive lock */
pub const F_UNLCK: i32 = 2; /* Unlock */

/// Estructura para operar con la biblioteca nativa de semáforos
#[repr(C)]
pub struct sembuf {
    pub sem_num: libc::c_ushort,
    pub sem_op: libc::c_short,
    pub sem_flg: libc::c_short,
}
