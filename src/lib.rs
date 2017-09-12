//! Temporary files and directories.
//!
//! - Use the [`tempfile`] function for temporary files
//! - Use the [`tempdir`][fn.tempdir] function for temporary directories.
//!
//! # Design
//!
//! This crate provides several approaches to creating temporary files and directories.
//! [`tempfile`] relies on the OS to remove the temporary file once the last handle is closed.
//! [`TempDir`][struct.TempDir] and [`NamedTempFile`] both rely on Rust destructors for cleanup.
//!
//! When choosing between the temporary file variants, prefer `tempfile`
//! unless you either need to know the file's path or to be able to persist it.
//!
//! ## Resource Leaking
//!
//! `tempfile` will (almost) never fail to cleanup temporary resources but `TempDir` and `NamedTempFile` will if
//! their destructor doesn't run. This is because `tempfile` relies on the OS to cleanup the
//! underlying file so the file while `TempDir` and `NamedTempFile` rely on their destructor to do so.
//!
//! ## Security
//!
//! In the presence of pathological temporary file cleaner, relying on file paths is unsafe because
//! a temporary file cleaner could delete the temporary file which an attacker could then replace.
//!
//! `tempfile` doesn't rely on file paths so this isn't an issue. However, `NamedTempFile` does
//! rely on file paths.
//!
//! ## Examples
//!
//! Create a temporary file and write some data into it:
//!
//! ```
//! # extern crate tempfile;
//! use tempfile::tempfile;
//! use std::io::{self, Write};
//!
//! # fn main() {
//! #     if let Err(_) = run() {
//! #         ::std::process::exit(1);
//! #     }
//! # }
//! # fn run() -> Result<(), io::Error> {
//! // Create a file inside of `std::env::temp_dir()`.
//! let mut file = tempfile()?;
//!
//! writeln!(file, "Brian was here. Briefly.")?;
//! # Ok(())
//! # }
//! ```
//!
//! Create a temporary directory and add a file to it:
//!
//! ```
//! # extern crate tempfile;
//! use tempfile::tempdir;
//! use std::fs::File;
//! use std::io::{self, Write};
//!
//! # fn main() {
//! #     if let Err(_) = run() {
//! #         ::std::process::exit(1);
//! #     }
//! # }
//! # fn run() -> Result<(), io::Error> {
//! // Create a directory inside of `std::env::temp_dir()`, named with
//! // the prefix "example".
//! let dir = tempdir("example")?;
//!
//! let file_path = dir.path().join("my-temporary-note.txt");
//! let mut file = File::create(file_path)?;
//! writeln!(file, "Brian was here. Briefly.")?;
//!
//! // By closing the `TempDir` explicitly, we can check that it has
//! // been deleted successfully. If we don't close it explicitly,
//! // the directory will still be deleted when `dir` goes out
//! // of scope, but we won't know whether deleting the directory
//! // succeeded.
//! drop(file);
//! dir.close()?;
//! # Ok(())
//! # }
//! ```
//!
//! [fn.tempdir]: fn.tempdir.html
//! [`tempfile`]: fn.tempfile.html
//! [struct.TempDir]: struct.TempDir.html
//! [`NamedTempFile`]: struct.NamedTempFile.html
//! [`std::env::temp_dir()`]: https://doc.rust-lang.org/std/env/fn.temp_dir.html

#![doc(html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
       html_favicon_url = "https://www.rust-lang.org/favicon.ico",
       html_root_url = "https://docs.rs/tempdir/0.3.5")]
#![cfg_attr(test, deny(warnings))]

extern crate remove_dir_all;
extern crate rand;

#[cfg(unix)]
extern crate libc;

#[cfg(windows)]
extern crate winapi;

#[cfg(target_os = "redox")]
extern crate syscall;

const NUM_RETRIES: u32 = 1 << 31;
const NUM_RAND_CHARS: usize = 6;

mod dir;
mod file;

pub use file::{tempfile, tempfile_in, NamedTempFile, NamedTempFileBuilder, PersistError};
pub use dir::{tempdir, TempDir};
