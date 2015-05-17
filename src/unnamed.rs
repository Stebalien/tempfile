use std::io::{self, Read, Write, Seek, SeekFrom};
use std::fs::File;
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

impl TempFile {
    /// Create a new temporary file.
    #[inline]
    pub fn new() -> io::Result<TempFile> {
        Self::new_in(&env::temp_dir())
    }

    /// Create a new temporary file in the specified directory.
    #[inline]
    pub fn new_in<P: AsRef<Path>>(dir: P) -> io::Result<TempFile> {
        imp::create(dir.as_ref()).map(|f| TempFile(f))
    }

    /// Create a new temporary file and open it `count` times returning `count` independent
    /// references to the same file.
    ///
    /// This can be useful if you want multiple seek positions in the same temporary file.
    /// Additionally, this function guarantees that all of the returned temporary file objects
    /// refer to the same underlying temporary file even in the presence of a pathological
    /// temporary file cleaner.
    #[inline]
    pub fn shared(count: usize) -> io::Result<Vec<TempFile>> {
        Self::shared_in(&env::temp_dir(), count)
    }

    /// Same as `shared` but creates the file in the specified directory.
    #[inline]
    pub fn shared_in<P: AsRef<Path>>(dir: P, count: usize) -> io::Result<Vec<TempFile>> {
        imp::create_shared(dir.as_ref(), count).map(|files| {
            files.into_iter().map(|f|TempFile(f)).collect()
        })
    }


    /// Number of bytes in the file.
    #[inline]
    pub fn len(&self) -> io::Result<u64> {
        self.0.metadata().map(|m| m.len())
    }

    /// Truncate the file to `size` bytes.
    #[inline(always)]
    pub fn set_len(&self, size: u64) -> io::Result<()> {
        self.0.set_len(size)
    }

    /// Re-open the temporary file. The returned TempFile will refer to the same underlying
    /// temporary file but will have an independent offset.
    ///
    /// This method is only available on windows and Linux, not FreeBSD/MacOS. Unfortunately, it is
    /// impossible to reliably implement this method on those operating systems.
    ///
    /// If you need your code to be cross-platform, please use `shared`/`shared_in` defined above.
    ///
    /// **Unstable**: This is platform specific and may go away in the future.
    #[cfg(any(windows, target_os = "linux"))]
    #[inline]
    pub fn reopen(&self) -> io::Result<TempFile> {
        imp::reopen(&self.0).map(|f|TempFile(f))
    }
}

impl Read for TempFile {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl Write for TempFile {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }
    #[inline(always)]
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl Seek for TempFile {
    #[inline(always)]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.0.seek(pos)
    }
}

#[cfg(unix)]
impl std::os::unix::io::AsRawFd for TempFile {
    #[inline(always)]
    fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        self.0.as_raw_fd()
    }
}

#[cfg(windows)]
impl std::os::windows::io::AsRawHandle for TempFile {
    #[inline(always)]
    fn as_raw_handle(&self) -> std::os::windows::io::RawHandle {
        self.0.as_raw_handle()
    }
}

