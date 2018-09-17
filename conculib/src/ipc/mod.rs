mod key;
pub mod shmem;
pub mod flock;
pub use self::key::Key;

pub const IPC_RMID   : i32 = 0o0000;   /* remove resource */
pub const IPC_SET    : i32 = 0o0001;   /* set ipc_perm options */
pub const IPC_STAT   : i32 = 0o0002;   /* get ipc_perm options */
pub const IPC_INFO   : i32 = 0o0003;   /* see ipcs */

pub const IPC_CREAT  : i32 = 0o1000;   /* create if key is nonexistent */
pub const IPC_EXCL   : i32 = 0o2000;   /* fail if key exists */
pub const IPC_NOWAIT : i32 = 0o4000;   /* return error on wait */

pub const LOCK_SH    : i32 = 0x0001;        /* Shared lock.  */
pub const LOCK_EX    : i32 = 0x0002;        /* Exclusive lock.  */
pub const LOCK_UN    : i32 = 0x0008;        /* Unlock.  */
pub const LOCK_NB    : i32 = 0x0004;        /* Don't block when locking.  */
