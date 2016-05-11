use std::fs::File;
use std::env;
use std::path::Path;
use std::io;

use super::imp;

/// Create an unnamed temporary file.
///
/// This method is secure/reliable in the presence of a pathological temporary file cleaner.
///
/// Deletion:
///
/// Linux >= 3.11: The temporary file is never linked into the filesystem so it can't be leaked.
///
/// Other *nix: The temporary file is immediately unlinked on create. The OS will delete it when
/// the last open copy of it is closed.
///
/// Windows: The temporary file is marked `DeleteOnClose` and, again, will be deleted when the last
/// open copy of it is closed. Unlike *nix operating systems, the file is not immediately unlinked
/// from the filesystem.
pub fn tempfile() -> io::Result<File> {
    tempfile_in(&env::temp_dir())
}

/// Create an unnamed temporary file in the specified directory.
///
/// See `tempfile()` for more information.
pub fn tempfile_in<P: AsRef<Path>>(dir: P) -> io::Result<File> {
    imp::create(dir.as_ref())
}
