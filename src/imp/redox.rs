use std::os::unix::ffi::OsStrExt;
use std::io;
use std::os::unix::io::{RawFd, FromRawFd, AsRawFd};
use std::fs::{self, File, OpenOptions};
use std::path::Path;
use util;

use syscall::{self, open, fstat, Stat as stat_t, O_EXCL, O_RDWR, O_CREAT, O_CLOEXEC};

pub fn cvt(result: Result<usize, syscall::Error>) -> io::Result<usize> {
    result.map_err(|err| io::Error::from_raw_os_error(err.errno))
}

pub fn create_named(path: &Path) -> io::Result<File> {
    let fd = cvt(open(path.as_os_str().as_bytes(),
                  O_CLOEXEC | O_EXCL | O_RDWR | O_CREAT | 0o600))?;
    unsafe { Ok(FromRawFd::from_raw_fd(fd)) }
}

pub fn create(dir: &Path) -> io::Result<File> {
    for _ in 0..::NUM_RETRIES {
        let tmp_path = dir.join(util::tmpname(".tmp", "", ::NUM_RAND_CHARS));
        return match create_named(&tmp_path) {
            Ok(file) => {
                // I should probably tell the user this failed but the temporary file creation
                // didn't really fail...
                let _ = fs::remove_file(tmp_path);
                Ok(file)
            }
            Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => Err(e),
        };
    }
    Err(io::Error::new(io::ErrorKind::AlreadyExists,
                       "too many temporary directories already exist"))
}

unsafe fn stat(fd: RawFd) -> io::Result<stat_t> {
    let mut meta: stat_t = ::std::mem::zeroed();
    cvt(fstat(fd, &mut meta))?;
    Ok(meta)
}

pub fn reopen(file: &File, path: &Path) -> io::Result<File> {
    let new_file = try!(OpenOptions::new().read(true).write(true).open(path));
    unsafe {
        let old_meta = try!(stat(file.as_raw_fd()));
        let new_meta = try!(stat(new_file.as_raw_fd()));
        if old_meta.st_dev != new_meta.st_dev || old_meta.st_ino != new_meta.st_ino {
            return Err(io::Error::new(io::ErrorKind::NotFound,
                                      "original tempfile has been replaced"));
        }
        Ok(new_file)
    }
}

pub fn persist(old_path: &Path, new_path: &Path, overwrite: bool) -> io::Result<()> {
    // XXX implement in better way when possible
    if !overwrite && new_path.exists() {
        return Err(io::Error::new(io::ErrorKind::AlreadyExists, "destination exists"));
    }
    fs::rename(old_path, new_path)
}
