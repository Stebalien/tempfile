#[cfg(unix)]
mod unix;

#[cfg(unix)]
pub use self::unix::*;

#[cfg(windows)]
mod unix;

#[cfg(windows)]
pub use self::windows::*;

