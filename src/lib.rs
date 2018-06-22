//! Temporary files and directories.
//!
//! - Use the [`tempfile()`] function for temporary files
//! - Use the [`tempdir()`] function for temporary directories.
//!
//! # Design
//!
//! This crate provides several approaches to creating temporary files and directories.
//! [`tempfile()`] relies on the OS to remove the temporary file once the last handle is closed.
//! [`TempDir`] and [`NamedTempFile`] both rely on Rust destructors for cleanup.
//!
//! When choosing between the temporary file variants, prefer `tempfile`
//! unless you either need to know the file's path or to be able to persist it.
//!
//! ## Resource Leaking
//!
//! `tempfile` will (almost) never fail to cleanup temporary resources, but `TempDir` and `NamedTempFile` will if
//! their destructors don't run. This is because `tempfile` relies on the OS to cleanup the
//! underlying file, while `TempDir` and `NamedTempFile` rely on their destructors to do so.
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
//! // Create a directory inside of `std::env::temp_dir()`.
//! let dir = tempdir()?;
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
//! [`tempfile()`]: fn.tempfile.html
//! [`tempdir()`]: fn.tempdir.html
//! [`TempDir`]: struct.TempDir.html
//! [`NamedTempFile`]: struct.NamedTempFile.html
//! [`std::env::temp_dir()`]: https://doc.rust-lang.org/std/env/fn.temp_dir.html

#![doc(html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
       html_favicon_url = "https://www.rust-lang.org/favicon.ico",
       html_root_url = "https://docs.rs/tempfile/2.2.0")]
#![cfg_attr(test, deny(warnings))]

extern crate rand;
extern crate remove_dir_all;

#[cfg(unix)]
extern crate libc;

#[cfg(windows)]
extern crate winapi;

#[cfg(target_os = "redox")]
extern crate syscall;

const NUM_RETRIES: u32 = 1 << 31;
const NUM_RAND_CHARS: usize = 6;

use std::path::Path;
use std::{env, io};

mod dir;
mod file;
mod util;

pub use dir::{tempdir, tempdir_in, TempDir};
pub use file::{tempfile, tempfile_in, NamedTempFile, PersistError, TempPath};

/// Create a new temporary file or directory with custom parameters.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Builder<'a, 'b> {
    world_accessible: bool,
    random_len: usize,
    prefix: &'a str,
    suffix: &'b str,
}

impl<'a, 'b> Default for Builder<'a, 'b> {
    fn default() -> Self {
        Builder {
            world_accessible: false,
            random_len: ::NUM_RAND_CHARS,
            prefix: ".tmp",
            suffix: "",
        }
    }
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
    ///     .tempfile()?;
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
        Self::default()
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
    ///     .tempfile()?;
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
    ///     .tempfile()?;
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
    ///     .tempfile()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn rand_bytes(&mut self, rand: usize) -> &mut Self {
        self.random_len = rand;
        self
    }

    /// Set whether anyone should be able to read and write to the temporary
    /// file.
    ///
    /// Default: `false`.
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
    ///     .world_accessible(true)
    ///     .tempfile()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn world_accessible(&mut self, world_accessible: bool) -> &mut Self {
        self.world_accessible = world_accessible;
        self
    }

    /// Create the named temporary file.
    ///
    /// # Security
    ///
    /// See [the security][security] docs on `NamedTempFile`.
    ///
    /// # Resource leaking
    ///
    /// See [the resource leaking][resource-leaking] docs on `NamedTempFile`.
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
    /// let tempfile = Builder::new().tempfile()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [security]: struct.NamedTempFile.html#security
    /// [resource-leaking]: struct.NamedTempFile.html#resource-leaking
    pub fn tempfile(&self) -> io::Result<NamedTempFile> {
        self.tempfile_in(&env::temp_dir())
    }

    /// Create the named temporary file in the specified directory.
    ///
    /// # Security
    ///
    /// See [the security][security] docs on `NamedTempFile`.
    ///
    /// # Resource leaking
    ///
    /// See [the resource leaking][resource-leaking] docs on `NamedTempFile`.
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
    /// let tempfile = Builder::new().tempfile_in("./")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [security]: struct.NamedTempFile.html#security
    /// [resource-leaking]: struct.NamedTempFile.html#resource-leaking
    pub fn tempfile_in<P: AsRef<Path>>(&self, dir: P) -> io::Result<NamedTempFile> {
        util::create_helper(
            dir.as_ref(),
            self.world_accessible,
            self.prefix,
            self.suffix,
            self.random_len,
            file::create_named,
        )
    }

    /// Attempts to make a temporary directory inside of `env::temp_dir()` whose
    /// name will have the prefix, `prefix`. The directory and
    /// everything inside it will be automatically deleted once the
    /// returned `TempDir` is destroyed.
    ///
    /// # Resource leaking
    ///
    /// See [the resource leaking][resource-leaking] docs on `TempDir`.
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
    /// [resource-leaking]: struct.TempDir.html#resource-leaking
    pub fn tempdir(&self) -> io::Result<TempDir> {
        self.tempdir_in(&env::temp_dir())
    }

    /// Attempts to make a temporary directory inside of `dir`.
    /// The directory and everything inside it will be automatically
    /// deleted once the returned `TempDir` is destroyed.
    ///
    /// # Resource leaking
    ///
    /// See [the resource leaking][resource-leaking] docs on `TempDir`.
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
    /// [resource-leaking]: struct.TempDir.html#resource-leaking
    pub fn tempdir_in<P: AsRef<Path>>(&self, dir: P) -> io::Result<TempDir> {
        let storage;
        let mut dir = dir.as_ref();
        if !dir.is_absolute() {
            let cur_dir = env::current_dir()?;
            storage = cur_dir.join(dir);
            dir = &storage;
        }

        util::create_helper(
            dir,
            self.world_accessible,
            self.prefix,
            self.suffix,
            self.random_len,
            dir::create,
        )
    }
}
