use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::io;

use crate::error::IoResultExt;

fn calculate_rand_buf_len(alphanumeric_len: usize) -> usize {
    let expected_non_alphanumeric_chars = alphanumeric_len / 32;
    (alphanumeric_len + expected_non_alphanumeric_chars) * 3 / 4 + 3
}

fn calculate_base64_len(binary_len: usize) -> usize {
    binary_len * 4 / 3 + 4
}

fn fill_with_random_base64(rand_buf: &mut [u8], char_buf: &mut Vec<u8>) {
    getrandom::getrandom(rand_buf).expect("calling getrandom failed");
    char_buf.resize(calculate_base64_len(rand_buf.len()), 0);
    base64::encode_config_slice(rand_buf, base64::STANDARD_NO_PAD, char_buf);
}

fn tmpname(prefix: &OsStr, suffix: &OsStr, rand_len: usize) -> OsString {
    let mut buf = OsString::with_capacity(prefix.len() + suffix.len() + rand_len);
    buf.push(prefix);

    let mut rand_buf = vec![0; calculate_rand_buf_len(rand_len)];
    let mut char_buf = vec![0; calculate_base64_len(rand_buf.len())];
    let mut remaining_chars = rand_len;
    loop {
        fill_with_random_base64(&mut rand_buf, &mut char_buf);
        char_buf.retain(|&c| (c != b'+') & (c != b'/') & (c != 0));
        if char_buf.len() >= remaining_chars {
            buf.push(std::str::from_utf8(&char_buf[..remaining_chars]).unwrap());
            break;
        } else {
            buf.push(std::str::from_utf8(&char_buf).unwrap());
            remaining_chars -= char_buf.len();
        }
    }

    buf.push(suffix);
    buf
}

pub fn create_helper<F, R>(
    base: &Path,
    prefix: &OsStr,
    suffix: &OsStr,
    random_len: usize,
    mut f: F,
) -> io::Result<R>
where
    F: FnMut(PathBuf) -> io::Result<R>,
{
    let num_retries = if random_len != 0 {
        crate::NUM_RETRIES
    } else {
        1
    };

    for _ in 0..num_retries {
        let path = base.join(tmpname(prefix, suffix, random_len));
        return match f(path) {
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
