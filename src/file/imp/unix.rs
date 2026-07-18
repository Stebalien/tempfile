use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

use crate::error::IoResultExt;
use crate::util;

use {
    rustix::fs::{rename, unlink},
    std::fs::hard_link,
};

pub fn create_named(
    path: &Path,
    open_options: &mut OpenOptions,
    #[cfg_attr(target_os = "wasi", allow(unused))] permissions: Option<&std::fs::Permissions>,
) -> io::Result<File> {
    open_options.read(true).write(true).create_new(true);

    #[cfg(not(target_os = "wasi"))]
    {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        open_options.mode(permissions.map(|p| p.mode()).unwrap_or(0o600));
    }

    open_options.open(path)
}

#[cfg(target_os = "linux")]
pub fn create(dir: &Path) -> io::Result<File> {
    use rustix::{fs::OFlags, io::Errno};
    use std::os::unix::fs::OpenOptionsExt;
    OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(OFlags::TMPFILE.bits() as i32) // do not mix with `create_new(true)`
        .open(dir)
        .or_else(|e| {
            match Errno::from_io_error(&e) {
                // These are the three "not supported" error codes for O_TMPFILE.
                Some(Errno::OPNOTSUPP) | Some(Errno::ISDIR) | Some(Errno::NOENT) => {
                    create_unix(dir)
                }
                _ => Err(e),
            }
        })
}

#[cfg(not(target_os = "linux"))]
pub fn create(dir: &Path) -> io::Result<File> {
    create_unix(dir)
}

fn create_unix(mut dir: &Path) -> io::Result<File> {
    use rustix::fs::{Mode, OFlags};

    // We can't just use O_RDONLY on platforms without O_PATH
    // because we might not have read-access to the directory containing
    // the temporary file.
    #[allow(unused)]
    const O_SEARCH: OFlags = if cfg!(target_os = "wasi") {
        OFlags::from_bits_retain(0x8000000)
    } else if cfg!(target_vendor = "apple") {
        OFlags::from_bits_retain(0x40000000)
    } else {
        OFlags::empty()
    };

    const DIR_OFLAGS: OFlags = OFlags::DIRECTORY.union(OFlags::CLOEXEC).union({
        #[cfg(any(
            target_os = "android",
            target_os = "freebsd",
            target_os = "linux",
            target_os = "redox",
        ))]
        {
            OFlags::PATH
        }
        #[cfg(not(any(
            target_os = "android",
            target_os = "freebsd",
            target_os = "linux",
            target_os = "redox",
        )))]
        {
            O_SEARCH
        }
    });

    const FILE_OFLAGS: OFlags = OFlags::RDWR
        .union(OFlags::CREATE)
        .union(OFlags::EXCL)
        .union(OFlags::CLOEXEC);
    const FILE_MODE: Mode = Mode::RUSR.union(Mode::WUSR);

    if dir == Path::new("") {
        dir = Path::new(".")
    }

    let dirfd = rustix::fs::open(dir, DIR_OFLAGS, Mode::empty())
        .map_err(|e| e.into())
        .with_err_path(|| dir)?;
    util::create_helper(
        crate::env::default_prefix(),
        OsStr::new(""),
        crate::NUM_RAND_CHARS,
        move |fname| {
            let f = rustix::fs::openat(&dirfd, &fname, FILE_OFLAGS, FILE_MODE)?;
            let _ = rustix::fs::unlinkat(&dirfd, fname, rustix::fs::AtFlags::empty());
            Ok(f.into())
        },
    )
    .with_err_path(|| dir)
}

pub fn reopen(file: &File, path: &Path) -> io::Result<File> {
    let new_file = OpenOptions::new().read(true).write(true).open(path)?;
    let old_meta = rustix::fs::fstat(file)?;
    let new_meta = rustix::fs::fstat(&new_file)?;
    if old_meta.st_dev != new_meta.st_dev || old_meta.st_ino != new_meta.st_ino {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "original tempfile has been replaced",
        ));
    }
    Ok(new_file)
}

pub fn persist(old_path: &Path, new_path: &Path, overwrite: bool) -> io::Result<()> {
    if overwrite {
        rename(old_path, new_path)?;
    } else {
        // On Linux, apple and redox operating systems, use `renameat_with` to avoid overwriting an
        // existing name, if the kernel and the filesystem support it.
        #[cfg(any(
            target_os = "android",
            target_os = "linux",
            target_os = "redox",
            target_vendor = "apple",
        ))]
        {
            use rustix::fs::{CWD, RenameFlags, renameat_with};
            use rustix::io::Errno;
            use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

            static NOSYS: AtomicBool = AtomicBool::new(false);
            if !NOSYS.load(Relaxed) {
                match renameat_with(CWD, old_path, CWD, new_path, RenameFlags::NOREPLACE) {
                    Ok(()) => return Ok(()),
                    Err(Errno::NOSYS) => NOSYS.store(true, Relaxed),
                    Err(Errno::INVAL) => {}
                    Err(e) => return Err(e.into()),
                }
            }
        }

        // Otherwise use `hard_link` to create the new filesystem name, which
        // will fail if the name already exists, and then `unlink` to remove
        // the old name.
        hard_link(old_path, new_path)?;

        // Ignore unlink errors. Can we do better?
        let _ = unlink(old_path);
    }
    Ok(())
}

pub fn keep(_: &Path) -> io::Result<()> {
    Ok(())
}
