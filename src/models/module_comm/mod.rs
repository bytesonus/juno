#[cfg(target_family = "unix")]
mod module_comm_unix;
#[cfg(target_family = "unix")]
pub use module_comm_unix::ModuleComm;

#[cfg(target_family = "windows")]
mod module_comm_windows;
#[cfg(target_family = "windows")]
pub use module_comm_windows::ModuleComm;
