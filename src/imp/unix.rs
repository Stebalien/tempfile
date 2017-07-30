use libc::{rename, link, linkat, unlink, c_char, c_int, O_EXCL, O_RDWR, O_CREAT, O_CLOEXEC};
use std::os::unix::ffi::OsStrExt;
use std::ffi::CString;
use std::io;
use std::os::unix::io::{RawFd, FromRawFd, AsRawFd};
use std::fs::{self, File, OpenOptions};
use std::path::Path;
use util;

#[cfg(all(lfs_support, target_os = "linux"))]
use libc::open64 as open;
#[cfg(all(lfs_support, target_os = "linux"))]
use libc::fstat64 as fstat;
#[cfg(all(lfs_support, target_os = "linux"))]
use libc::stat64 as stat_t;

#[cfg(not(all(lfs_support, target_os = "linux")))]
use libc::open;
#[cfg(not(all(lfs_support, target_os = "linux")))]
use libc::fstat;
#[cfg(not(all(lfs_support, target_os = "linux")))]
use libc::stat as stat_t;

// Stolen from std.
pub fn cstr(path: &Path) -> io::Result<CString> {
    // TODO: Use OsStr::to_cstring (convert)
    CString::new(path.as_os_str().as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "path contained a null"))
}

pub fn create_named(path: &Path) -> io::Result<File> {
    unsafe {
        let path = try!(cstr(path));
        match open(path.as_ptr() as *const c_char,
                   O_CLOEXEC | O_EXCL | O_RDWR | O_CREAT,
                   0o600) {
            -1 => Err(io::Error::last_os_error()),
            fd => Ok(FromRawFd::from_raw_fd(fd)),
        }
    }
}

const O_TMPFILE: c_int = 0o20200000;

#[cfg(target_os = "linux")]
pub fn create(dir: &Path) -> io::Result<File> {
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

#[cfg(target_os = "linux")]
pub fn create_persistable(dir: &Path) -> io::Result<(c_int, File)> {
    match unsafe {
        let path = try!(cstr(dir));
        open(path.as_ptr() as *const c_char,
             O_CLOEXEC | O_TMPFILE | O_RDWR,
             0o600)
    } {
        -1 => Err(io::ErrorKind::InvalidInput.into()),
        fd => Ok(unsafe { (fd, FromRawFd::from_raw_fd(fd)) }),
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
    if fstat(fd, &mut meta as *mut stat_t) != 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(meta)
    }
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

pub fn persist(old_path: &Path, new_path: &Path, overwrite: bool, deleted: bool) -> io::Result<()> {
    unsafe {
        let mut old_path = try!(cstr(old_path));
        let new_path_str = try!(cstr(new_path));

        if deleted {
            if overwrite {
                // we can't rename the file directly into the right place, so we have to ...
                // create a new named temporary file, then rename that. :/

                let dir = if let Some(parent) = new_path.parent() {
                    parent
                } else {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, "destination file must be in a directory"))
                };

                old_path = try!(create_named_link_in(&old_path, dir));
                // fall through, and let the existing rename code handle this

            } else {
                return link_symlink_fd_at(&old_path, &new_path_str);
            }
        }

        if overwrite {
            if rename(old_path.as_ptr() as *const c_char,
                      new_path_str.as_ptr() as *const c_char) != 0 {
                return Err(io::Error::last_os_error());
            }
        } else {
            if link(old_path.as_ptr() as *const c_char,
                    new_path_str.as_ptr() as *const c_char) != 0 {
                return Err(io::Error::last_os_error());
            }
            // Ignore unlink errors. Can we do better?
            // On recent linux, we can use renameat2 to do this atomically.
            let _ = unlink(old_path.as_ptr() as *const c_char);
        }
        Ok(())
    }
}

/// Very much like creating a named temporary file, except `link_symlink_fd_at` is already
/// atomic/exclusive.
unsafe fn create_named_link_in(old_path: &CString, dir: &Path) -> io::Result<CString> {
    for _ in 0..::NUM_RETRIES {
        let path = dir.join(util::tmpname(".persist-", ".tmp", 6));
        let new_path = try!(cstr(&path));
        return match link_symlink_fd_at(&old_path, &new_path) {
            Ok(()) => Ok(new_path),
            Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => Err(e),
        }
    }
    Err(io::Error::new(io::ErrorKind::AlreadyExists,
                       "too many temporary files exist"))
}

/// Attempt to link an old symlink to a file back into the filesystem.
unsafe fn link_symlink_fd_at(old_path: &CString, new_path: &CString) -> io::Result<()> {
    const AT_FDCWD: c_int = -100;
    const AT_SYMLINK_FOLLOW: c_int = 0x400;

    if linkat(AT_FDCWD,
              old_path.as_ptr() as *const c_char,
              AT_FDCWD,
              new_path.as_ptr() as *const c_char,
              AT_SYMLINK_FOLLOW) != 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(never)]
unsafe fn renameat2(old_dir_fd: c_int, old_path: *const c_char, new_dir_fd: c_int, new_path: *const c_char, flags: c_uint) -> c_long {
    use libc::{syscall, SYS_renameat2};
    syscall(SYS_renameat2, old_dir_fd, old_path, new_dir_fd, new_path, flags)
}
