use std::io::{self, Read, Write, Seek, SeekFrom};
use std::fs::File;
use std::fmt;
use std::path::Path;
use std::env;
use std;

use super::imp;

/// An unnamed temporary file.
///
/// This variant is secure/reliable in the presence of a pathological temporary file cleaner.
///
/// Deletion:
///
/// Linux >= 3.11: The temporary file is never linked into the filesystem so it can't be leaked.
///
/// Other *nix: The temporary file is immediately unlinked on create. The OS will delete it when
/// the last open copy of it is closed (the last TempFile reference to it is dropped).
///
/// Windows: The temporary file is marked DeleteOnClose and, again, will be deleted when the last
/// open copy of it is closed. Unlike *nix operating systems, the file is not immediately unlinked
/// from the filesystem.
pub struct TempFile(File);

impl fmt::Debug for TempFile {
    #[cfg(unix)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::os::unix::io::AsRawFd;
        write!(f, "TempFile({})", self.0.as_raw_fd())
    }

    #[cfg(windows)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::os::windows::io::AsRawHandle;
        write!(f, "TempFile({})", self.0.as_raw_handle() as usize)
    }
}


#[allow(len_without_is_empty)]
impl TempFile {
    /// Create a new temporary file.
    pub fn new() -> io::Result<TempFile> {
        Self::new_in(&env::temp_dir())
    }

    /// Create a new temporary file in the specified directory.
    pub fn new_in<P: AsRef<Path>>(dir: P) -> io::Result<TempFile> {
        imp::create(dir.as_ref()).map(TempFile)
    }

    /// Create a new temporary file and open it `count` times returning `count` independent
    /// references to the same file.
    ///
    /// This can be useful if you want multiple seek positions in the same temporary file.
    /// Additionally, this function guarantees that all of the returned temporary file objects
    /// refer to the same underlying temporary file even in the presence of a pathological
    /// temporary file cleaner.
    pub fn shared(count: usize) -> io::Result<Vec<TempFile>> {
        Self::shared_in(&env::temp_dir(), count)
    }

    /// Same as `shared` but creates the file in the specified directory.
    pub fn shared_in<P: AsRef<Path>>(dir: P, count: usize) -> io::Result<Vec<TempFile>> {
        imp::create_shared(dir.as_ref(), count).map(|files| {
            files.into_iter().map(TempFile).collect()
        })
    }


    /// Number of bytes in the file.
    pub fn len(&self) -> io::Result<u64> {
        self.0.metadata().map(|m| m.len())
    }

    /// Truncate the file to `size` bytes.
    pub fn set_len(&self, size: u64) -> io::Result<()> {
        self.0.set_len(size)
    }
}

impl Read for TempFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl Write for TempFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl Seek for TempFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.0.seek(pos)
    }
}

#[cfg(unix)]
impl std::os::unix::io::AsRawFd for TempFile {
    #[inline]
    fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        self.0.as_raw_fd()
    }
}

#[cfg(windows)]
impl std::os::windows::io::AsRawHandle for TempFile {
    #[inline]
    fn as_raw_handle(&self) -> std::os::windows::io::RawHandle {
        self.0.as_raw_handle()
    }
}
