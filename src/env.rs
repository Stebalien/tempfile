use std::env;
use std::path::{Path, PathBuf};

// Once rust 1.70 is wide-spread (Debian stable), we can use OnceLock from stdlib.
use once_cell::sync::OnceCell as OnceLock;

static DEFAULT_TEMPDIR: OnceLock<PathBuf> = OnceLock::new();

/// Override the default temporary directory (defaults to [`std::env::temp_dir`]. This function
/// changes the _global_ default temporary directory for the entire program and should not be called
/// except in exceptional cases where it's not configured correctly by the platform.
///
/// Only the first call to this function will succeed, returning `Ok(path)`. All further calls will
/// fail with `Err(path)`. In both cases, `path` is the default temporary directory (on success, the
/// new one; on failure, the existing one).
pub fn override_temp_dir(path: &Path) -> Result<&'static Path, &'static Path> {
    let mut we_set = false;
    let val = DEFAULT_TEMPDIR.get_or_init(|| {
        we_set = true;
        path.to_path_buf()
    });
    if we_set {
        Ok(val)
    } else {
        Err(val)
    }
}

/// Returns the default temporary directory, used for both temporary directories and files if no
/// directory is explicitly specified.
///
/// This function simply delegates to [`std::env::temp_dir`] unless the default temporary directory
/// has been override by a call to [`override_temp_dir`].
pub fn temp_dir() -> PathBuf {
    DEFAULT_TEMPDIR
        .get()
        .map(|p| p.to_owned())
        // Don't cache this in case the user uses std::env::set to change the temporary directory.
        .unwrap_or_else(env::temp_dir)
}
