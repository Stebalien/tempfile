use std::fs::File;
use std::io::{self, Read, Write, Seek, SeekFrom, Cursor};
use file::tempfile;

/// An object that behaves like a regular temporary file, but keeps data in
/// memory until it reaches a configured size, at which point the data is
/// written to a temporary file on disk, and further operations use the file
/// on disk.
#[derive(Debug)]
pub struct SpooledTempFile {
    max_size: usize,
    cursor: Option<Cursor<Vec<u8>>>,
    file: Option<File>,
}

/// Create a new spooled temporary file.
///
/// # Security
///
/// This variant is secure/reliable in the presence of a pathological temporary
/// file cleaner.
///
/// # Resource Leaking
///
/// The temporary file will be automatically removed by the OS when the last
/// handle to it is closed. This doesn't rely on Rust destructors being run, so
/// will (almost) never fail to clean up the temporary file.
///
/// # Examples
///
/// ```
/// # extern crate tempfile;
/// use tempfile::spooled_tempfile;
/// use std::io::{self, Write};
///
/// # fn main() {
/// #     if let Err(_) = run() {
/// #         ::std::process::exit(1);
/// #     }
/// # }
/// # fn run() -> Result<(), io::Error> {
/// let mut file = spooled_tempfile(15);
///
/// writeln!(file, "short line")?;
/// assert!(!file.rolled_over());
///
/// // as a result of this write call, the size of the data will exceed
/// // `max_size` (15), so it will be written to a temporary file on disk,
/// // and the in-memory buffer will be dropped
/// writeln!(file, "marvin gardens")?;
/// assert!(file.rolled_over());
///
/// # Ok(())
/// # }
/// ```
pub fn spooled_tempfile(max_size: usize) -> SpooledTempFile {
    SpooledTempFile {
        max_size: max_size,
        cursor: Some(Cursor::new(Vec::new())),
        file: None,
    }
}

impl SpooledTempFile {
    /// Returns true if the file has been rolled over to disk.
    pub fn rolled_over(&self) -> bool {
        if let Some(ref _file) = self.file {
            true
        } else {
            false
        }
    }
}

impl Read for SpooledTempFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if let Some(ref mut cursor) = self.cursor {
            cursor.read(buf)
        } else if let Some(ref mut file) = self.file {
            file.read(buf)
        } else {
            panic!(); // bug
        }
    }
}

impl Write for SpooledTempFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // roll over to file if necessary
        let mut rolling = false;
        if let Some(ref mut cursor) = self.cursor {
            rolling = cursor.position() as usize + buf.len() > self.max_size;
            if rolling {
                let mut file = tempfile()?;
                file.write(cursor.get_ref())?;
                file.seek(SeekFrom::Start(cursor.position()))?;
                self.file = Some(file);
            }
        }
        if rolling {
            self.cursor.take();
        }

        // write the bytes
        if let Some(ref mut cursor) = self.cursor {
            cursor.write(buf)
        } else if let Some(ref mut file) = self.file {
            file.write(buf)
        } else {
            panic!(); // bug
        }
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        if let Some(ref mut cursor) = self.cursor {
            cursor.flush()
        } else if let Some(ref mut file) = self.file {
            file.flush()
        } else {
            panic!(); // bug
        }
    }
}

impl Seek for SpooledTempFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        if let Some(ref mut cursor) = self.cursor {
            cursor.seek(pos)
        } else if let Some(ref mut file) = self.file {
            file.seek(pos)
        } else {
            panic!(); // bug
        }
    }
}
