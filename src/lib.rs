//! Securely create and manage temporary files. Temporary files created by this create are
//! automatically deleted.
//!
//! This crate provides two temporary file variants: a `tempfile()` function that returns standard
//! `File` objects and `NamedTempFile`. When choosing between the variants, prefer `tempfile()`
//! unless you either need to know the file's path or to be able to persist it.
//!
//! # Example
//!
//! ```
//! use tempfile::tempfile;
//! use std::fs::File;
//!
//! let mut file: File = tempfile().expect("failed to create temporary file");
//! ```
//!
//! # Differences
//!
//! ## Resource Leaking
//!
//! `tempfile()` will (almost) never fail to cleanup temporary files but `NamedTempFile` will if
//! its destructor doesn't run. This is because `tempfile()` relies on the OS to cleanup the
//! underlying file so the file while `NamedTempFile` relies on its destructor to do so.
//!
//! ## Security
//!
//! In the presence of pathological temporary file cleaner, relying on file paths is unsafe because
//! a temporary file cleaner could delete the temporary file which an attacker could then replace.
//!
//! `tempfile()` doesn't rely on file paths so this isn't an issue. However, `NamedTempFile` does
//! rely on file paths.
//!

extern crate rand;

#[cfg(unix)]
extern crate libc;

#[cfg(windows)]
extern crate winapi;

#[cfg(windows)]
extern crate kernel32;

#[cfg(target_os = "redox")]
extern crate syscall;

const NUM_RETRIES: u32 = 1 << 31;
const NUM_RAND_CHARS: usize = 6;

mod util;
mod imp;
mod named;
mod unnamed;

pub use named::{NamedTempFile, NamedTempFileOptions, PersistError};
pub use unnamed::{tempfile, tempfile_in};
