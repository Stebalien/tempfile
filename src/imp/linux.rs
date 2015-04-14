use ::libc::{self, c_int, O_EXCL, O_RDWR};
use std::io;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::fs::{File, OpenOptions};
use std::path::Path;
use ::util::cstr;
use super::unix_common::O_CLOEXEC;
use super::unix_common::create as create_unix;

const O_TMPFILE: libc::c_int = 0o20200000;

pub fn create(dir: &Path) -> io::Result<File> {
    match unsafe {
        libc::open(try!(cstr(dir)).as_ptr(), O_CLOEXEC | O_EXCL | O_TMPFILE | O_RDWR, 0o600)
    } {
        -1 => create_unix(dir),
        fd => Ok(unsafe { FromRawFd::from_raw_fd(fd) }),
    }
}

pub fn create_shared(dir: &Path, count: usize) -> io::Result<Vec<File>> {
    if count == 0 {
        return Ok(vec![]);
    }
    let first = try!(create(dir));
    let mut files: Vec<File> = try!((1..count).map(|_| reopen(&first)).collect());
    files.push(first);
    Ok(files)
}

pub fn reopen(f: &File) -> io::Result<File> {
    OpenOptions::new().read(true).write(true).create(false).open(format!("/dev/fd/{}", f.as_raw_fd()))
}
