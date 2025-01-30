use std::env;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, OnceLock};

#[cfg(doc)]
use crate::{tempdir_in, tempfile_in, Builder};

static ENV_TEMPDIR: LazyLock<PathBuf> = LazyLock::new(env::temp_dir);
static DEFAULT_TEMPDIR: OnceLock<PathBuf> = OnceLock::new();
static DEFAULT_PREFIX: OnceLock<OsString> = OnceLock::new();

/// Override the default temporary directory (defaults to [`std::env::temp_dir`]). This function
/// changes the _global_ default temporary directory for the entire program and should not be called
/// except in exceptional cases where it's not configured correctly by the platform. Applications
/// should first check if the path returned by [`env::temp_dir`] is acceptable.
///
/// If you're writing a library and want to control where your temporary files are placed, you
/// should instead use the `_in` variants of the various temporary file/directory constructors
/// ([`tempdir_in`], [`tempfile_in`], the so-named functions on [`Builder`], etc.).
///
/// Only the **first** call to this function will succeed and return `Ok(path)` where `path` is a
/// static reference to the temporary directory. All further calls will fail with `Err(path)` where
/// `path` is the previously set default temporary directory override.
///
/// **NOTE:** This function does not check if the specified directory exists and/or is writable.
pub fn override_temp_dir(path: impl Into<PathBuf>) -> Result<&'static Path, &'static Path> {
    let mut path = Some(path.into());
    let val = DEFAULT_TEMPDIR.get_or_init(|| path.take().unwrap());
    match path {
        Some(_) => Err(val),
        None => Ok(val),
    }
}

/// Returns the default temporary directory, used for both temporary directories and files if no
/// directory is explicitly specified.
///
/// This function simply delegates to [`std::env::temp_dir`] unless the default temporary directory
/// has been override by a call to [`override_temp_dir`].
///
/// **NOTE:**
///
/// 1. This function does not check if the returned directory exists and/or is writable.
/// 2. This function caches the result of [`std::env::temp_dir`]. Any future changes to, e.g., the
///    `TMPDIR` environment variable won't have any effect.
pub fn temp_dir() -> &'static Path {
    DEFAULT_TEMPDIR.get().unwrap_or_else(|| &ENV_TEMPDIR)
}

/// Override the default prefix for new temporary files (defaults to "tmp"). This function changes
/// the _global_ default prefix used by the entire program and should only be used by the top-level
/// application. It's recommended that the top-level application call this function to specify an
/// application-specific prefix to make it easier to identify temporary files belonging to the
/// application.
///
/// Only the **first** call to this function will succeed and return `Ok(prefix)` where `prefix` is
/// a static reference to the default temporary file prefix. All further calls will fail with
/// `Err(prefix)` where `prefix` is the previously set default temporary file prefix.
pub fn override_default_prefix(
    prefix: impl Into<OsString>,
) -> Result<&'static OsStr, &'static OsStr> {
    let mut prefix = Some(prefix.into());
    let val = DEFAULT_PREFIX.get_or_init(|| prefix.take().unwrap());
    match prefix {
        Some(_) => Err(val),
        None => Ok(val),
    }
}

/// Returns the default prefix used for new temporary files if no prefix is explicitly specified via
/// [`Builder::prefix`].
pub fn default_prefix() -> &'static OsStr {
    DEFAULT_PREFIX.get().map(|p| &**p).unwrap_or("tmp".as_ref())
}
