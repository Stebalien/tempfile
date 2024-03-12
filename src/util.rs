use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::{io, iter::repeat_with};

use crate::error::IoResultExt;

use fastrand::Rng;

pub fn create_helper<R>(
    base: &Path,
    prefix: &OsStr,
    suffix: &OsStr,
    random_len: usize,
    permissions: Option<&std::fs::Permissions>,
    mut f: impl FnMut(PathBuf, Option<&std::fs::Permissions>) -> io::Result<R>,
) -> io::Result<R> {
    let capacity = prefix
        .len()
        .saturating_add(random_len)
        .saturating_add(suffix.len());
    let mut buf = OsString::with_capacity(capacity);

    if random_len == 0 {
        buf.push(prefix);
        buf.push(suffix);
        let path = base.join(buf);
        f(path, permissions)
    } else {
        let mut char_buf = [0u8; 4];
        let mut rng = Rng::new();

        for _ in 0..crate::NUM_RETRIES {
            buf.push(prefix);
            for c in repeat_with(|| rng.alphanumeric()).take(random_len) {
                buf.push(c.encode_utf8(&mut char_buf));
            }
            buf.push(suffix);
            let path = base.join(&buf);
            buf.clear();
            return match f(path, permissions) {
                Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                // AddrInUse can happen if we're creating a UNIX domain socket and
                // the path already exists.
                Err(ref e) if e.kind() == io::ErrorKind::AddrInUse => continue,
                res => res,
            };
        }

        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "too many temporary files exist",
        ))
        .with_err_path(|| base)
    }
}
