use ::libc::{self, c_int, O_EXCL, O_RDWR, O_CREAT};
use ::libc::types::os::arch::posix01::stat as stat_t;
use std::io;
use std::os::unix::io::{RawFd, FromRawFd};
use std::fs::{self, File, OpenOptions};
use std::path::Path;
use std::ffi::CString;
use ::util::{tmpname, cstr};
use super::unix_common::O_CLOEXEC;
pub use super::unix_common::create;

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

pub fn create_shared(dir: &Path, count: usize) -> io::Result<Vec<File>> {
    let mut opts = OpenOptions::new();
    opts.read(true).write(true).create(false);

    if count == 0 {
        return Ok(vec![]);
    }
    'outer: loop {
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
                let dg = DeleteGuard(&tmp_path);

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
}
