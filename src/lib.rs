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
//! `tempfile` will (almost) never fail to cleanup temporary resources. However `TempDir` and `NamedTempFile` will
//! fail if their destructors don't run. This is because `tempfile` relies on the OS to cleanup the
//! underlying file, while `TempDir` and `NamedTempFile` rely on rust destructors to do so.
//! Destructors may fail to run if the process exits through an unhandled signal interrupt (like `SIGINT`),
//! or if the instance is declared statically (like with [`lazy_static`]), among other possible
//! reasons.
//!
//! ## Security
//!
//! In the presence of pathological temporary file cleaner, relying on file paths is unsafe because
//! a temporary file cleaner could delete the temporary file which an attacker could then replace.
//!
//! `tempfile` doesn't rely on file paths so this isn't an issue. However, `NamedTempFile` does
//! rely on file paths for _some_ operations. See the security documentation on
//! the `NamedTempFile` type for more information.
//!
//! ## Early drop pitfall
//!
//! Because `TempDir` and `NamedTempFile` rely on their destructors for cleanup, this can lead
//! to an unexpected early removal of the directory/file, usually when working with APIs which are
//! generic over `AsRef<Path>`. Consider the following example:
//!
//! ```no_run
//! # use tempfile::tempdir;
//! # use std::io;
//! # use std::process::Command;
//! # fn main() {
//! #     if let Err(_) = run() {
//! #         ::std::process::exit(1);
//! #     }
//! # }
//! # fn run() -> Result<(), io::Error> {
//! // Create a directory inside of `std::env::temp_dir()`.
//! let temp_dir = tempdir()?;
//!
//! // Spawn the `touch` command inside the temporary directory and collect the exit status
//! // Note that `temp_dir` is **not** moved into `current_dir`, but passed as a reference
//! let exit_status = Command::new("touch").arg("tmp").current_dir(&temp_dir).status()?;
//! assert!(exit_status.success());
//!
//! # Ok(())
//! # }
//! ```
//!
//! This works because a reference to `temp_dir` is passed to `current_dir`, resulting in the
//! destructor of `temp_dir` being run after the `Command` has finished execution. Moving the
//! `TempDir` into the `current_dir` call would result in the `TempDir` being converted into
//! an internal representation, with the original value being dropped and the directory thus
//! being deleted, before the command can be executed.
//!
//! The `touch` command would fail with an `No such file or directory` error.
//!
//! ## Examples
//!
//! Create a temporary file and write some data into it:
//!
//! ```
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
//! Create a named temporary file and open an independent file handle:
//!
//! ```
//! use tempfile::NamedTempFile;
//! use std::io::{self, Write, Read};
//!
//! # fn main() {
//! #     if let Err(_) = run() {
//! #         ::std::process::exit(1);
//! #     }
//! # }
//! # fn run() -> Result<(), io::Error> {
//! let text = "Brian was here. Briefly.";
//!
//! // Create a file inside of `std::env::temp_dir()`.
//! let mut file1 = NamedTempFile::new()?;
//!
//! // Re-open it.
//! let mut file2 = file1.reopen()?;
//!
//! // Write some test data to the first handle.
//! file1.write_all(text.as_bytes())?;
//!
//! // Read the test data using the second handle.
//! let mut buf = String::new();
//! file2.read_to_string(&mut buf)?;
//! assert_eq!(buf, text);
//! # Ok(())
//! # }
//! ```
//!
//! Create a temporary directory and add a file to it:
//!
//! ```
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
//! [`lazy_static`]: https://github.com/rust-lang-nursery/lazy-static.rs/issues/62

#![doc(
    html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
    html_favicon_url = "https://www.rust-lang.org/favicon.ico",
    html_root_url = "https://docs.rs/tempfile/3.1.0"
)]
#![cfg_attr(test, deny(warnings))]
#![deny(rust_2018_idioms)]
#![allow(clippy::redundant_field_names)]
#![cfg_attr(all(feature = "nightly", target_os = "wasi"), feature(wasi_ext))]

#[cfg(doctest)]
doc_comment::doctest!("../README.md");

const NUM_RETRIES: u32 = 1 << 31;
const NUM_RAND_CHARS: usize = 6;

use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::path::Path;
use std::{env, io};

mod dir;
mod error;
mod file;
mod spooled;
mod util;

pub use crate::dir::{tempdir, tempdir_in, TempDir};
pub use crate::file::{
    tempfile, tempfile_in, NamedTempFile, PathPersistError, PersistError, TempPath,
};
pub use crate::spooled::{spooled_tempfile, SpooledTempFile};

/// Create a new temporary file or directory with custom parameters.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Builder<'a, 'b> {
    random_len: usize,
    prefix: &'a OsStr,
    suffix: &'b OsStr,
    append: bool,
    permissions: Option<std::fs::Permissions>,
}

impl<'a, 'b> Default for Builder<'a, 'b> {
    fn default() -> Self {
        Builder {
            random_len: crate::NUM_RAND_CHARS,
            prefix: OsStr::new(".tmp"),
            suffix: OsStr::new(""),
            append: false,
            permissions: None,
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
    ///
    /// Create a temporary directory with a chosen prefix under a chosen folder:
    ///
    /// ```ignore
    /// let dir = Builder::new()
    ///     .prefix("my-temporary-dir")
    ///     .tempdir_in("folder-with-tempdirs")?;
    /// ```
    #[must_use]
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
    pub fn prefix<S: AsRef<OsStr> + ?Sized>(&mut self, prefix: &'a S) -> &mut Self {
        self.prefix = prefix.as_ref();
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
    pub fn suffix<S: AsRef<OsStr> + ?Sized>(&mut self, suffix: &'b S) -> &mut Self {
        self.suffix = suffix.as_ref();
        self
    }

    /// Set the number of random bytes.
    ///
    /// Default: `6`.
    ///
    /// # Examples
    ///
    /// ```
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

    /// Set the file to be opened in append mode.
    ///
    /// Default: `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io;
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// # use tempfile::Builder;
    /// let named_tempfile = Builder::new()
    ///     .append(true)
    ///     .tempfile()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }

    /// The permissions to create the tempfile or [tempdir](Self::tempdir) with.
    /// This allows to them differ from the default mode of `0o600` on Unix.
    ///
    /// # Security
    ///
    /// By default, the permissions of tempfiles on unix are set for it to be
    /// readable and writable by the owner only, yielding the greatest amount
    /// of security.
    /// As this method allows to widen the permissions, security would be
    /// reduced in such cases.
    ///
    /// # Platform Notes
    /// ## Unix
    ///
    /// The actual permission bits set on the tempfile or tempdir will be affected by the
    /// `umask` applied by the underlying syscall.
    ///
    ///
    /// ## Windows and others
    ///
    /// This setting is unsupported and trying to set a file or directory read-only
    /// will cause an error to be returned..
    ///
    /// # Examples
    ///
    /// Create a named temporary file that is world-readable.
    ///
    /// ```
    /// # use std::io;
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// # use tempfile::Builder;
    /// #[cfg(unix)]
    /// {
    ///     use std::os::unix::fs::PermissionsExt;
    ///     let all_read_write = std::fs::Permissions::from_mode(0o666);
    ///     let tempfile = Builder::new().permissions(all_read_write).tempfile()?;
    ///     let actual_permissions = tempfile.path().metadata()?.permissions();
    ///     assert_ne!(
    ///         actual_permissions.mode() & !0o170000,
    ///         0o600,
    ///         "we get broader permissions than the default despite umask"
    ///     );
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Create a named temporary directory that is restricted to the owner.
    ///
    /// ```
    /// # use std::io;
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// # use tempfile::Builder;
    /// #[cfg(unix)]
    /// {
    ///     use std::os::unix::fs::PermissionsExt;
    ///     let owner_rwx = std::fs::Permissions::from_mode(0o700);
    ///     let tempdir = Builder::new().permissions(owner_rwx).tempdir()?;
    ///     let actual_permissions = tempdir.path().metadata()?.permissions();
    ///     assert_eq!(
    ///         actual_permissions.mode() & !0o170000,
    ///         0o700,
    ///         "we get the narrow permissions we asked for"
    ///     );
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn permissions(&mut self, permissions: std::fs::Permissions) -> &mut Self {
        self.permissions = Some(permissions);
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
        self.tempfile_in(env::temp_dir())
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
            self.prefix,
            self.suffix,
            self.random_len,
            self.permissions.as_ref(),
            |path, permissions| {
                file::create_named(path, OpenOptions::new().append(self.append), permissions)
            },
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
        self.tempdir_in(env::temp_dir())
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
            self.prefix,
            self.suffix,
            self.random_len,
            self.permissions.as_ref(),
            dir::create,
        )
    }

    /// Attempts to create a temporary file (or file-like object) using the
    /// provided closure. The closure is passed a temporary file path and
    /// returns an [`std::io::Result`]. The path provided to the closure will be
    /// inside of [`std::env::temp_dir()`]. Use [`Builder::make_in`] to provide
    /// a custom temporary directory. If the closure returns one of the
    /// following errors, then another randomized file path is tried:
    ///  - [`std::io::ErrorKind::AlreadyExists`]
    ///  - [`std::io::ErrorKind::AddrInUse`]
    ///
    /// This can be helpful for taking full control over the file creation, but
    /// leaving the temporary file path construction up to the library. This
    /// also enables creating a temporary UNIX domain socket, since it is not
    /// possible to bind to a socket that already exists.
    ///
    /// Note that [`Builder::append`] is ignored when using [`Builder::make`].
    ///
    /// # Security
    ///
    /// This has the same [security implications][security] as
    /// [`NamedTempFile`], but with additional caveats. Specifically, it is up
    /// to the closure to ensure that the file does not exist and that such a
    /// check is *atomic*. Otherwise, a [time-of-check to time-of-use
    /// bug][TOCTOU] could be introduced.
    ///
    /// For example, the following is **not** secure:
    ///
    /// ```
    /// # use std::io;
    /// # use std::fs::File;
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// # use tempfile::Builder;
    /// // This is NOT secure!
    /// let tempfile = Builder::new().make(|path| {
    ///     if path.is_file() {
    ///         return Err(io::ErrorKind::AlreadyExists.into());
    ///     }
    ///
    ///     // Between the check above and the usage below, an attacker could
    ///     // have replaced `path` with another file, which would get truncated
    ///     // by `File::create`.
    ///
    ///     File::create(path)
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    /// Note that simply using [`std::fs::File::create`] alone is not correct
    /// because it does not fail if the file already exists:
    /// ```
    /// # use std::io;
    /// # use std::fs::File;
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// # use tempfile::Builder;
    /// // This could overwrite an existing file!
    /// let tempfile = Builder::new().make(|path| File::create(path))?;
    /// # Ok(())
    /// # }
    /// ```
    /// For creating regular temporary files, use [`Builder::tempfile`] instead
    /// to avoid these problems. This function is meant to enable more exotic
    /// use-cases.
    ///
    /// # Resource leaking
    ///
    /// See [the resource leaking][resource-leaking] docs on `NamedTempFile`.
    ///
    /// # Errors
    ///
    /// If the closure returns any error besides
    /// [`std::io::ErrorKind::AlreadyExists`] or
    /// [`std::io::ErrorKind::AddrInUse`], then `Err` is returned.
    ///
    /// # Examples
    /// ```
    /// # use std::io;
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// # use tempfile::Builder;
    /// # #[cfg(unix)]
    /// use std::os::unix::net::UnixListener;
    /// # #[cfg(unix)]
    /// let tempsock = Builder::new().make(|path| UnixListener::bind(path))?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [TOCTOU]: https://en.wikipedia.org/wiki/Time-of-check_to_time-of-use
    /// [security]: struct.NamedTempFile.html#security
    /// [resource-leaking]: struct.NamedTempFile.html#resource-leaking
    pub fn make<F, R>(&self, f: F) -> io::Result<NamedTempFile<R>>
    where
        F: FnMut(&Path) -> io::Result<R>,
    {
        self.make_in(env::temp_dir(), f)
    }

    /// This is the same as [`Builder::make`], except `dir` is used as the base
    /// directory for the temporary file path.
    ///
    /// See [`Builder::make`] for more details and security implications.
    ///
    /// # Examples
    /// ```
    /// # use std::io;
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// # use tempfile::Builder;
    /// # #[cfg(unix)]
    /// use std::os::unix::net::UnixListener;
    /// # #[cfg(unix)]
    /// let tempsock = Builder::new().make_in("./", |path| UnixListener::bind(path))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn make_in<F, R, P>(&self, dir: P, mut f: F) -> io::Result<NamedTempFile<R>>
    where
        F: FnMut(&Path) -> io::Result<R>,
        P: AsRef<Path>,
    {
        util::create_helper(
            dir.as_ref(),
            self.prefix,
            self.suffix,
            self.random_len,
            None,
            move |path, _permissions| {
                Ok(NamedTempFile::from_parts(
                    f(&path)?,
                    TempPath::from_path(path),
                ))
            },
        )
    }
}
