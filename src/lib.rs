//! Securely create and manage temporary files. Temporary files created by this create are
//! automatically deleted.
//!
//! This crate provides two temporary file variants: a `tempfile()` function that returns standard
//! `File` objects and `NamedTempFile`. When choosing between the variants, prefer `tempfile()`
//! unless you either need to know the file's path or to be able to persist it.
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

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate libc;
extern crate rand;

#[cfg(windows)]
extern crate winapi;

#[cfg(windows)]
extern crate kernel32;

const NUM_RETRIES: u32 = 1 << 31;
const NUM_RAND_CHARS: usize = 6;

mod util;
mod imp;
mod named;
mod unnamed;

pub use named::{NamedTempFile, NamedTempFileOptions};
pub use unnamed::{tempfile, tempfile_in};
