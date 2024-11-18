use std::env;
use std::path::{Path, PathBuf};

// Once rust 1.70 is wide-spread (Debian stable), we can use OnceLock from stdlib.
use once_cell::sync::OnceCell as OnceLock;

static DEFAULT_TEMPDIR: OnceLock<PathBuf> = OnceLock::new();

/// Override the default temporary directory (defaults to [`std::env::temp_dir`]). This function
/// changes the default temporary directory **globally**, as in **process-wide**.
///
/// It only be called in exceptional cases where the temporary directory is not configured
/// correctly by the platform. Only the first call to this function will succeed. All
/// further calls will fail with `Err(path)` where `path` is the previously set default
/// temporary directory override.
///
/// You may want to use [`crate::Builder::tempdir_in`] instead, particularly if your software
/// project follows a more "object-oriented" architecture paradigm. For example, when working
/// with multiple struct objects that include a separate temporary directory field.
///
/// **NOTE:** This function does not check if the specified directory exists and/or is writable.
pub fn override_temp_dir(path: &Path) -> Result<(), PathBuf> {
    let mut we_set = false;
    let val = DEFAULT_TEMPDIR.get_or_init(|| {
        we_set = true;
        path.to_path_buf()
    });
    if we_set {
        Ok(())
    } else {
        Err(val.to_owned())
    }
}

/// Returns the default temporary directory, used for both temporary directories and files if no
/// directory is explicitly specified.
///
/// This function simply delegates to [`std::env::temp_dir`] unless the default temporary directory
/// has been override by a call to [`override_temp_dir`].
///
/// **NOTE:** This function does check if the returned directory exists and/or is writable.
pub fn temp_dir() -> PathBuf {
    DEFAULT_TEMPDIR
        .get()
        .map(|p| p.to_owned())
        // Don't cache this in case the user uses std::env::set to change the temporary directory.
        .unwrap_or_else(env::temp_dir)
}
