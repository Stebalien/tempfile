#[cfg(unix)]
mod unix_common;

#[cfg(all(not(target_os = "linux"), unix))]
mod unix;

#[cfg(all(not(target_os = "linux"), unix))]
pub use self::unix::*;

#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use self::windows::*;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use self::linux::*;
