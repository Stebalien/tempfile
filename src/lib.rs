//! Temporary files and directories.
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
//! their destructors don't run. This is because `tempfile` relies on the OS to cleanup the
//! underlying file so the file while `TempDir` and `NamedTempFile` rely on their destructors to do so.
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
//! use tempfile::TempDir;
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
//! let dir = TempDir::new()?;
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
//! [`tempfile`]: fn.tempfile.html
//! [struct.TempDir]: struct.TempDir.html
//! [`NamedTempFile`]: struct.NamedTempFile.html
//! [`std::env::temp_dir()`]: https://doc.rust-lang.org/std/env/fn.temp_dir.html

#![doc(html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
       html_favicon_url = "https://www.rust-lang.org/favicon.ico",
       html_root_url = "https://docs.rs/tempfile/2.1.6")]
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

use std::{io, env};
use std::path::Path;

mod dir;
mod file;
mod util;

pub use file::{tempfile, tempfile_in, NamedTempFile, PersistError};
pub use dir::TempDir;

/// Create a new temporary file or directory with custom parameters.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Builder<'a, 'b> {
    random_len: usize,
    prefix: &'a str,
    suffix: &'b str,
}

impl<'a, 'b> Builder<'a, 'b> {
    /// Create a new `Builder`.
    ///
    /// # Examples
    ///
    /// Create a named temporary file and write some data into it:
    /// 
    /// ```
    /// # extern crate tempfile;
    /// # use std::io;
    /// # use std::ffi::OsStr;
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// use tempfile::Builder;
    ///
    /// let named_tempfile = Builder::new()
    ///     .prefix("my-temporary-note")
    ///     .suffix(".txt")
    ///     .rand_bytes(5)
    ///     .named_tempfile()?;
    ///
    /// let name = named_tempfile
    ///     .path()
    ///     .file_name().and_then(OsStr::to_str);
    ///
    /// if let Some(name) = name {
    ///     assert!(name.starts_with("my-temporary-note"));
    ///     assert!(name.ends_with(".txt"));
    ///     assert_eq!(name.len(), "my-temporary-note.txt".len() + 5);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    /// 
    /// Create a temporary directory and add a file to it:
    /// 
    /// ```
    /// # extern crate tempfile;
    /// # use std::io::{self, Write};
    /// # use std::fs::File;
    /// # use std::ffi::OsStr;
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// use tempfile::Builder;
    ///
    /// let dir = Builder::new()
    ///     .prefix("my-temporary-dir")
    ///     .rand_bytes(5)
    ///     .tempdir()?;
    ///
    /// let file_path = dir.path().join("my-temporary-note.txt");
    /// let mut file = File::create(file_path)?;
    /// writeln!(file, "Brian was here. Briefly.")?;
    ///
    /// // By closing the `TempDir` explicitly, we can check that it has
    /// // been deleted successfully. If we don't close it explicitly,
    /// // the directory will still be deleted when `dir` goes out
    /// // of scope, but we won't know whether deleting the directory
    /// // succeeded.
    /// drop(file);
    /// dir.close()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Self {
        Builder {
            random_len: ::NUM_RAND_CHARS,
            prefix: ".tmp",
            suffix: "",
        }
    }

    /// Set a custom filename prefix.
    ///
    /// Path separators are legal but not advisable.
    /// Default: `.tmp`.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate tempfile;
    /// # use std::io;
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// # use tempfile::Builder;
    /// let named_tempfile = Builder::new()
    ///     .prefix("my-temporary-note")
    ///     .named_tempfile()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn prefix(&mut self, prefix: &'a str) -> &mut Self {
        self.prefix = prefix;
        self
    }

    /// Set a custom filename suffix.
    ///
    /// Path separators are legal but not advisable.
    /// Default: empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate tempfile;
    /// # use std::io;
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// # use tempfile::Builder;
    /// let named_tempfile = Builder::new()
    ///     .suffix(".txt")
    ///     .named_tempfile()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn suffix(&mut self, suffix: &'b str) -> &mut Self {
        self.suffix = suffix;
        self
    }

    /// Set the number of random bytes.
    ///
    /// Default: `6`.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate tempfile;
    /// # use std::io;
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// # use tempfile::Builder;
    /// let named_tempfile = Builder::new()
    ///     .rand_bytes(5)
    ///     .named_tempfile()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn rand_bytes(&mut self, rand: usize) -> &mut Self {
        self.random_len = rand;
        self
    }

    /// Create the named temporary file.
    ///
    /// # Security
    ///
    /// See: [`NamedTempFile::new`]
    /// 
    /// # Resource leaking
    /// 
    /// See: [`NamedTempFile::new`]
    ///
    /// # Errors
    ///
    /// If the file cannot be created, `Err` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate tempfile;
    /// # use std::io;
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// # use tempfile::Builder;
    /// let named_tempfile = Builder::new().named_tempfile()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`NamedTempFile::new`]: struct.NamedTempFile.html#method.new
    pub fn named_tempfile(&self) -> io::Result<NamedTempFile> {
        self.named_tempfile_in(&env::temp_dir())
    }

    /// Create the named temporary file in the specified directory.
    ///
    /// # Security
    ///
    /// See: [`NamedTempFile::new`]
    /// 
    /// # Resource leaking
    /// 
    /// See: [`NamedTempFile::new`]
    /// 
    /// # Errors
    ///
    /// If the file cannot be created, `Err` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate tempfile;
    /// # use std::io;
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// # use tempfile::Builder;
    /// let named_tempfile = Builder::new().named_tempfile_in("./")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`NamedTempFile::new`]: struct.NamedTempFile.html#method.new
    pub fn named_tempfile_in<P: AsRef<Path>>(&self, dir: P) -> io::Result<NamedTempFile> {
        for _ in 0..::NUM_RETRIES {
            let path = dir.as_ref().join(util::tmpname(self.prefix, self.suffix, self.random_len));

            return match file::create_named(path) {
                Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                file => file
            };
        }

        Err(io::Error::new(io::ErrorKind::AlreadyExists,
                           "too many temporary files exist"))

    }

    /// Attempts to make a temporary directory inside of `env::temp_dir()` whose
    /// name will have the prefix, `prefix`. The directory and
    /// everything inside it will be automatically deleted once the
    /// returned `TempDir` is destroyed.
    ///
    /// # Resource leaking
    /// 
    /// See: [`TempDir`]
    /// 
    /// # Errors
    ///
    /// If the directory can not be created, `Err` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::File;
    /// use std::io::Write;
    /// use tempfile::Builder;
    ///
    /// # use std::io;
    /// # fn run() -> Result<(), io::Error> {
    /// let tmp_dir = Builder::new().tempdir()?;
    /// # Ok(())
    /// # }
    /// ```
    /// 
    /// [`TempDir`]: struct.TempDir.html
    pub fn tempdir(&self) -> io::Result<TempDir> {
        self.tempdir_in(&env::temp_dir())
    }

    /// Attempts to make a temporary directory inside of `dir`.
    /// The directory and everything inside it will be automatically 
    /// deleted once the returned `TempDir` is destroyed.
    /// 
    /// # Resource leaking
    /// 
    /// See: [`TempDir`]
    ///
    /// # Errors
    ///
    /// If the directory can not be created, `Err` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::{self, File};
    /// use std::io::Write;
    /// use tempfile::Builder;
    ///
    /// # use std::io;
    /// # fn run() -> Result<(), io::Error> {
    /// let tmp_dir = Builder::new().tempdir_in("./")?;
    /// # Ok(())
    /// # }
    /// ```
    /// 
    /// [`TempDir`]: struct.TempDir.html
    pub fn tempdir_in<P: AsRef<Path>>(&self, dir: P) -> io::Result<TempDir> {
        let storage;
        let mut dir = dir.as_ref();
        if !dir.is_absolute() {
            let cur_dir = env::current_dir()?;
            storage = cur_dir.join(dir);
            dir = &storage;
        }

        for _ in 0..::NUM_RETRIES {
            let path = dir.join(util::tmpname(self.prefix, self.suffix, self.random_len));

            return match dir::create(path) {
                Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                dir => dir
            };
        }

        Err(io::Error::new(io::ErrorKind::AlreadyExists,
                           "too many temporary directories exist"))
    }
}
