use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::{io, iter::repeat_with};

use crate::error::IoResultExt;

fn tmpname(
    rng: &mut fastrand::Rng,
    prefix: &OsStr,
    suffix: &OsStr,
    rand_len: usize,
) -> io::Result<OsString> {
    let capacity = prefix
        .len()
        .saturating_add(suffix.len())
        .saturating_add(rand_len);
    let mut buf = OsString::with_capacity(capacity);
    buf.push(prefix);
    let mut char_buf = [0u8; 4];
    for c in repeat_with(|| rng.alphanumeric()).take(rand_len) {
        buf.push(c.encode_utf8(&mut char_buf));
    }
    buf.push(suffix);

    check_valid_filename(&buf)?;

    Ok(buf)
}

/// This function is like [`create_helper`] except that it takes a base directory in which
/// to create the temporary file/directory, makes that path absolute, and expands the provided
/// names suggested to that directory.
///
/// This is used for creating named temporary files, directories, sockets, etc.
pub fn create_absolute_helper<R>(
    base: &Path,
    prefix: &OsStr,
    suffix: &OsStr,
    random_len: usize,
    mut f: impl FnMut(PathBuf) -> io::Result<R>,
) -> io::Result<R> {
    // Make the path absolute. Otherwise, changing the current directory can invalidate a stored
    // path (causing issues when cleaning up temporary files).
    let mut base = base; // re-borrow to shrink lifetime
    let base_path_storage; // slot to store the absolute path, if necessary.
    if !base.is_absolute() {
        base_path_storage = std::path::absolute(base)?;
        base = &base_path_storage;
    }
    create_helper(prefix, suffix, random_len, |path| (f)(base.join(path))).with_err_path(|| base)
}

/// This function wraps a temporary file creation operation, suggesting temporary file names and
/// deciding whether or not to try again based on the returned error and the number of attempts.
///
/// This is used for creating unnamed temporary files.
pub fn create_helper<R>(
    prefix: &OsStr,
    suffix: &OsStr,
    random_len: usize,
    mut f: impl FnMut(OsString) -> io::Result<R>,
) -> io::Result<R> {
    let num_retries = if random_len != 0 {
        crate::NUM_RETRIES
    } else {
        1
    };

    // We fork the fastrand rng.
    let mut rng = fastrand::Rng::new();
    for i in 0..num_retries {
        // If we fail to create the file the first three times, re-seed from system randomness in
        // case an attacker is predicting our randomness (fastrand is predictable). If re-seeding
        // doesn't help, either:
        //
        // 1. We have lots of temporary files, possibly created by an attacker but not necessarily.
        //    Re-seeding the randomness won't help here.
        // 2. We're failing to create random files for some other reason. This shouldn't be the case
        //    given that we're checking error kinds, but it could happen.
        #[cfg(all(
            feature = "getrandom",
            any(windows, unix, target_os = "redox", target_os = "wasi")
        ))]
        if i == 3 {
            if let Ok(seed) = getrandom::u64() {
                rng.seed(seed);
            }
        }
        let _ = i; // avoid unused variable warning for the above.

        return match f(tmpname(&mut rng, prefix, suffix, random_len)?) {
            Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists && num_retries > 1 => continue,
            // AddrInUse can happen if we're creating a UNIX domain socket and
            // the path already exists.
            Err(ref e) if e.kind() == io::ErrorKind::AddrInUse && num_retries > 1 => continue,
            res => res,
        };
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "too many temporary files exist",
    ))
}

/// Check that the passed path is a valid file-name (no directories, no nulls, etc.).
fn check_valid_filename(path: impl AsRef<OsStr>) -> io::Result<()> {
    let path = path.as_ref();

    // Make sure the filename isn't empty.
    if path.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "temporary filename is empty",
        ));
    }

    // Check for null bytes, encoding doesn't matter. The OS/libc will do this for us, but we might
    // as well be extra safe.
    if path.as_encoded_bytes().contains(&0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("temporary filename {path:?} contains a null byte"),
        ));
    }

    // Make sure the filename is exactly one path element and make sure that element is a file name.
    // This is the most reliable way to check for path separators.
    if Path::new(path).file_name() != Some(path) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("temporary filename {path:?} is not a valid path component"),
        ));
    }

    Ok(())
}

#[test]
fn test_filename_validation() {
    check_valid_filename("foo").unwrap();
    check_valid_filename("foo.bar").unwrap();
    check_valid_filename("/foo").expect_err("path contains a slash");
    check_valid_filename("foo/bar").expect_err("path contains a slash");
    check_valid_filename("foo/").expect_err("path contains a slash");
    check_valid_filename("/").expect_err("path contains a slash");
    check_valid_filename("\0").expect_err("path contains a null");
}
