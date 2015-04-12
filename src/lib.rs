#![feature(convert, from_raw_os)]
//! Securely create and manage temporary files. Temporary files created by this create are
//! automatically deleted on exit (actually, they're usually deleted immediately after they are
//! created).
//!
extern crate libc;
extern crate rand;

use std::io::{self, Read, Write, Seek, SeekFrom};
use std::fs::File;
use std::path::Path;
use std::env;

mod imp;
mod util;

pub struct TempFile(File);

impl TempFile {
    /// Create a new temporary file.
    #[inline(always)]
    pub fn new() -> io::Result<TempFile> {
        Self::new_in(&env::temp_dir())
    }

    /// Create a new temporary file in the specified directory.
    #[inline(always)]
    pub fn new_in<P: AsRef<Path>>(dir: P) -> io::Result<TempFile> {
        imp::create(dir.as_ref()).map(|f| TempFile(f))
    }

    /*
    /// Create a new temporary file and return a vector of open files pointing to this file
    /// descriptor.
    ///
    /// Note: This function exists because `TempFile::reopen` will not work on FreeBSD without
    /// fdescfs mounted.
    ///
    /// This function may go away if I stop caring.
    #[inline(always)]
    pub fn shared(count: usize) -> io::Result<Vec<TempFile>> {
        Self::shared_in(&env::temp_dir(), count)
    }
    /// Same as above but lets you specify the directory.
    #[inline(always)]
    pub fn shared_in(count: usize) -> io::Result<Vec<TempFile>> {
        imp::create_shared(dir.as_ref(), count).map(|v| v.map(|f| TempFile(f)))
    }
    */
    /// Number of bytes in the file.
    #[inline(always)]
    pub fn len(&self) -> io::Result<u64> {
        self.0.metadata().map(|m| m.len())
    }
    /// Truncate the file to `size` bytes.
    #[inline(always)]
    pub fn set_len(&self, size: u64) -> io::Result<()> {
        self.0.set_len(size)
    }

    /// Create a new temporary file that references 
    ///
    /// On unix systems, this requires access to `/dev/fd/`. On freebsd/macosx, this requires that
    /// the fdescfs filesystem be mounted.
    #[inline(always)]
    pub fn share(&self) -> io::Result<TempFile> {
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
        self.0.as_raw_fd()
    }
}
