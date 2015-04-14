use ::libc::{self, c_int, O_EXCL, O_RDWR, O_CREAT};
use std::io;
use std::os::unix::io::FromRawFd;
use std::fs::{self, File};
use std::path::Path;
use ::util::{tmpname, cstr};

pub const O_CLOEXEC: libc::c_int = 0o2000000;

pub fn create(dir: &Path) -> io::Result<File> {
    loop {
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
            }
            fd => {
                let f = unsafe { FromRawFd::from_raw_fd(fd) };
                // I should probably tell the user this failed but the temporary file creation
                // didn't really fail...
                let _ = fs::remove_file(tmp_path);
                Ok(f)
            }
        }
    }
}

