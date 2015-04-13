use ::libc::{self, c_int, O_EXCL, O_RDWR, O_CREAT};
use std::io;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::fs::{self, File, OpenOptions};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ffi::{OsStr, CString};
use ::util::tmpname;

const O_CLOEXEC: libc::c_int = 0o2000000;
#[cfg(target_os = "linux")]
const O_TMPFILE: libc::c_int = 0o20200000;

// Stolen from std.
fn cstr(path: &Path) -> io::Result<CString> {
    path.as_os_str().to_cstring().ok_or(
        io::Error::new(io::ErrorKind::InvalidInput, "path contained a null"))
}

fn create_unix(dir: &Path) -> io::Result<File> {
    loop {
        let name = tmpname();
        let tmp_path = dir.join(OsStr::from_bytes(&name));
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
            }
            fd => {
                let f = FromRawFd::from_raw_fd(fd);
                // I should probably tell the user this failed but the temporary file creation
                // didn't really fail...
                let _ = fs::remove_file(tmp_path);
                Ok(f)
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn create_linux(dir: &Path) -> io::Result<File> {
    let dir = try!(cstr(dir));

    match unsafe {
        libc::open(dir.as_ptr(), O_CLOEXEC | O_EXCL | O_TMPFILE | O_RDWR, 0o600)
    } {
        -1 => Err(io::Error::last_os_error()),
        fd => Ok(FromRawFd::from_raw_fd(fd)),
    }
}

#[cfg(target_os = "linux")]
pub fn create(dir: &Path) -> io::Result<File> {
    // Fallback on unix create in case the kernel version is < 3.11.
    create_linux(dir).or_else(|_| create_unix(dir))
}

#[cfg(not(target_os = "linux"))]
#[inline(always)]
pub fn create(dir: &Path) -> io::Result<File> {
    create_unix(dir)
}

pub fn reopen(f: &File) -> io::Result<File> {
    OpenOptions::new().read(true).write(true).create(false).open(format!("/dev/fd/{}", f.as_raw_fd()))
}
