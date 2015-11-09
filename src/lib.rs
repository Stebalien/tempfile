//! Securely create and manage temporary files. Temporary files created by this create are
//! automatically deleted.
//!
//! This crate provides two temporary file variants: `TempFile` and `NamedTempFile`. When choosing
//! between the variants, prefer `TempFile` unless you either need to know the file's path or to be
//! able to persist it.
//!
//! # Differences
//!
//! ## Resource Leaking
//!
//! `TempFile` will (almost) never fail to cleanup temporary files but `NamedTempFile` will if its
//! destructor doesn't run. This is because `TempFile` relies on the OS to cleanup the underlying
//! file so the file while `NamedTempFile` relies on its destructor to do so.
//!
//! ## Security
//!
//! In the presence of pathological temporary file cleaner, relying on file paths is unsafe because
//! a temporary file cleaner could delete the temporary file which an attacker could then replace.
//!
//! `TempFile` doesn't rely on file paths so this isn't an issue. However, `NamedTempFile` does
//! rely on file paths.
//!
extern crate libc;
extern crate rand;

#[cfg(windows)]
extern crate winapi;

#[cfg(windows)]
extern crate kernel32;

const NUM_RETRIES: u32 = 1 << 31;
const NUM_RAND_CHARS: usize = 12;

mod imp;
mod named;
mod unnamed;

pub use ::named::{NamedTempFile, CustomNamedTempFile};
pub use ::unnamed::TempFile;
