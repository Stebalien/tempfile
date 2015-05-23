use ::libc::{self, c_int, O_EXCL, O_RDWR, O_CREAT};
use ::libc::types::os::arch::posix01::stat as stat_t;
use std::os::unix::ffi::OsStrExt;
use std::ffi::CString;
use std::io;
use std::os::unix::io::{RawFd, FromRawFd, AsRawFd};
use std::fs::{self, File, OpenOptions};
use std::path::Path;
use ::util::tmpname;

pub const O_CLOEXEC: libc::c_int = 0o2000000;

// Stolen from std.
#[inline]
pub fn cstr(path: &Path) -> io::Result<CString> {
    // TODO: Use OsStr::to_cstring (convert)
    CString::new(path.as_os_str().as_bytes()).map_err(|_|
        io::Error::new(io::ErrorKind::InvalidInput, "path contained a null"))
}

pub fn create_named(path: &Path) -> io::Result<File> {
    return match unsafe {
        libc::open(try!(cstr(&path)).as_ptr(), O_CLOEXEC | O_EXCL | O_RDWR | O_CREAT, 0o600)
    } {
        -1 => Err(io::Error::last_os_error()),
        fd => Ok(unsafe { FromRawFd::from_raw_fd(fd) }),
    }
}


#[cfg(target_os = "linux")]
pub fn create(dir: &Path) -> io::Result<File> {
    const O_TMPFILE: libc::c_int = 0o20200000;
    match unsafe {
        libc::open(try!(cstr(dir)).as_ptr(), O_CLOEXEC | O_EXCL | O_TMPFILE | O_RDWR, 0o600)
    } {
        -1 => create_unix(dir),
        fd => Ok(unsafe { FromRawFd::from_raw_fd(fd) }),
    }
}

#[inline(always)]
#[cfg(not(target_os = "linux"))]
pub fn create(dir: &Path) -> io::Result<File> {
    create_unix(dir)
}

fn create_unix(dir: &Path) -> io::Result<File> {
    for _ in 0..::NUM_RETRIES {
        let tmp_path = dir.join(&tmpname());
        return match create_named(&tmp_path) {
            Ok(file) => {
                // I should probably tell the user this failed but the temporary file creation
                // didn't really fail...
                let _ = fs::remove_file(tmp_path);
                Ok(file)
            },
            Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => Err(e),
        }
    }
    Err(io::Error::new(io::ErrorKind::AlreadyExists,
                       "too many temporary directories already exist"))
}

unsafe fn stat(fd: RawFd) -> io::Result<stat_t> {
    let mut meta: stat_t = ::std::mem::zeroed();
    if libc::fstat(fd, &mut meta as *mut stat_t) != 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(meta)
    }
}

// Helper for ensuring that the temporary file gets deleted.
struct DeleteGuard<'a>(&'a Path);

impl<'a> Drop for DeleteGuard<'a> {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.0);
    }
}

// One can do this on linux using proc but that doesn't play well with sandboxing...
pub fn create_shared(dir: &Path, count: usize) -> io::Result<Vec<File>> {
    let mut opts = OpenOptions::new();
    opts.read(true).write(true).create(false);

    if count == 0 {
        return Ok(vec![]);
    }
    'outer: for _ in 0..::NUM_RETRIES {
        let tmp_path = dir.join(&tmpname());
        return match unsafe {
            libc::open(try!(cstr(&tmp_path)).as_ptr(), O_CLOEXEC | O_EXCL | O_RDWR | O_CREAT, 0o600)
        } {
            -1 => {
                let err = io::Error::last_os_error();
                if err.kind() == io::ErrorKind::AlreadyExists {
                    continue;
                } else {
                    Err(err)
                }
            },
            fd => unsafe {
                let first = FromRawFd::from_raw_fd(fd);
                let _dg = DeleteGuard(&tmp_path);

                let target_meta = try!(stat(fd));
                let mut files: Vec<File> = try!((1..count).map(|_| opts.open(&tmp_path)).collect());
                for file in &files {
                    let meta = try!(stat(file.as_raw_fd()));
                    if meta.st_dev != target_meta.st_dev ||
                       meta.st_ino != target_meta.st_ino ||
                       // Even if the device information get's reused, the owner should actually be
                       // sufficient.
                       meta.st_uid != target_meta.st_uid ||
                       meta.st_gid != target_meta.st_gid {

                        // Error? Panic? If we hit this, we're likely under attack (or a hardware
                        // glitch/reconfiguration?.
                        continue 'outer;
                    }

                }
                files.push(first);
                Ok(files)
            },
        }
    }
    Err(io::Error::new(io::ErrorKind::AlreadyExists,
                       "too many temporary directories already exist"))
}

pub fn persist(old_path: &Path, new_path: &Path) -> io::Result<()> {
    fs::rename(old_path, new_path)
}
