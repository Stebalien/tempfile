#[cfg(not(target_os = "redox"))]
use libc::{rename, link, unlink, c_char, c_int, O_EXCL, O_RDWR, O_CREAT, O_CLOEXEC};
use std::os::unix::ffi::OsStrExt;
use std::ffi::CString;
use std::io;
use std::os::unix::io::{RawFd, FromRawFd, AsRawFd};
use std::fs::{self, File, OpenOptions};
use std::path::Path;
use util;

#[cfg(all(lfs_support, target_os = "linux"))]
use libc::{open64 as open, fstat64 as fstat, stat64 as stat_t};

#[cfg(not(any(all(lfs_support, target_os = "linux"), target_os = "redox")))]
use libc::{open, fstat, stat as stat_t};

#[cfg(target_os = "redox")]
use syscall::{self, open, fstat, Stat as stat_t, O_EXCL, O_RDWR, O_CREAT, O_CLOEXEC};

#[cfg(not(target_os = "redox"))]
#[inline(always)]
pub fn cvt_err(result: c_int) -> io::Result<c_int> {
    if result == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(result)
    }
}

#[cfg(target_os = "redox")]
#[inline(always)]
pub fn cvt_err(result: Result<usize, syscall::Error>) -> io::Result<usize> {
    result.map_err(|err| io::Error::from_raw_os_error(err.errno))
}

// Stolen from std.
pub fn cstr(path: &Path) -> io::Result<CString> {
    // TODO: Use OsStr::to_cstring (convert)
    CString::new(path.as_os_str().as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "path contained a null"))
}

#[cfg(not(target_os = "redox"))]
pub fn create_named(path: &Path) -> io::Result<File> {
    unsafe {
        let path = try!(cstr(path));
        let fd = try!(cvt_err(open(path.as_ptr() as *const c_char,
                                   O_CLOEXEC | O_EXCL | O_RDWR | O_CREAT,
                                   0o600)));
        Ok(FromRawFd::from_raw_fd(fd))
    }
}

#[cfg(target_os = "redox")]
pub fn create_named(path: &Path) -> io::Result<File> {
    unsafe {
        let fd = try!(cvt_err(open(path.as_os_str().as_bytes(),
                                   O_CLOEXEC | O_EXCL | O_RDWR | O_CREAT | 0o600)));
        Ok(FromRawFd::from_raw_fd(fd))
    }
}

#[cfg(target_os = "linux")]
pub fn create(dir: &Path) -> io::Result<File> {
    const O_TMPFILE: c_int = 0o20200000;
    match unsafe {
        let path = try!(cstr(dir));
        open(path.as_ptr() as *const c_char,
             O_CLOEXEC | O_EXCL | O_TMPFILE | O_RDWR,
             0o600)
    } {
        -1 => create_unix(dir),
        fd => Ok(unsafe { FromRawFd::from_raw_fd(fd) }),
    }
}

#[cfg(not(target_os = "linux"))]
pub fn create(dir: &Path) -> io::Result<File> {
    create_unix(dir)
}

fn create_unix(dir: &Path) -> io::Result<File> {
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
    try!(cvt_err(fstat(fd, &mut meta)));
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

#[cfg(not(target_os = "redox"))]
pub fn persist(old_path: &Path, new_path: &Path, overwrite: bool) -> io::Result<()> {
    unsafe {
        let old_path = try!(cstr(old_path));
        let new_path = try!(cstr(new_path));
        if overwrite {
            try!(cvt_err(rename(old_path.as_ptr() as *const c_char,
                                new_path.as_ptr() as *const c_char)));
        } else {
            try!(cvt_err(link(old_path.as_ptr() as *const c_char,
                              new_path.as_ptr() as *const c_char)));
            // Ignore unlink errors. Can we do better?
            // On recent linux, we can use renameat2 to do this atomically.
            let _ = unlink(old_path.as_ptr() as *const c_char);
        }
        Ok(())
    }
}

#[cfg(target_os = "redox")]
pub fn persist(old_path: &Path, new_path: &Path, overwrite: bool) -> io::Result<()> {
    // XXX implement when possible
    Err(io::Error::from_raw_os_error(syscall::ENOSYS))
}
