
use std::ffi::{OsString, OsStr};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::ops::{Deref, DerefMut};
use std::error;
use std::fmt;
use std::env;
use std;
use ::rand;
use ::rand::Rng;

use super::imp;

/// A named temporary file.
///
/// This variant is *NOT* secure/reliable in the presence of a pathological temporary file cleaner.
///
/// NamedTempFiles are deleted on drop. As rust doesn't guarantee that a struct will ever be
/// dropped, these temporary files will not be deleted on abort, resource leak, early exit, etc.
///
/// Please use TempFile unless you absolutely need a named file.
///
pub struct NamedTempFile(Option<NamedTempFileInner>);

struct NamedTempFileInner {
    file: File,
    path: PathBuf,
}

impl fmt::Debug for NamedTempFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NamedTempFile({:?})", self.0.as_ref().unwrap().path)
    }
}

impl Deref for NamedTempFile {
    type Target = File;
    #[inline]
    fn deref(&self) -> &File {
        &self.inner().file
    }
}

impl DerefMut for NamedTempFile {
    #[inline]
    fn deref_mut(&mut self) -> &mut File {
        &mut self.inner_mut().file
    }
}

/// Error returned when persisting a temporary file fails
#[derive(Debug)]
pub struct PersistError {
    /// The underlying IO error.
    pub error: io::Error,
    /// The temporary file that couldn't be persisted.
    pub file: NamedTempFile,
}

impl From<PersistError> for io::Error {
    #[inline]
    fn from(error: PersistError) -> io::Error {
        error.error
    }
}

impl From<PersistError> for NamedTempFile {
    #[inline]
    fn from(error: PersistError) -> NamedTempFile {
        error.file
    }
}

impl fmt::Display for PersistError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to persist temporary file: {}", self.error)
    }
}

impl error::Error for PersistError {
    fn description(&self) -> &str {
        "failed to persist temporary file"
    }
    fn cause(&self) -> Option<&error::Error> {
        Some(&self.error)
    }
}

impl NamedTempFile {
    #[inline]
    fn inner(&self) -> &NamedTempFileInner {
        self.0.as_ref().unwrap()
    }

    #[inline]
    fn inner_mut(&mut self) -> &mut NamedTempFileInner {
        self.0.as_mut().unwrap()
    }

    /// Create a new temporary file.
    ///
    /// *SECURITY WARNING:* This will create a temporary file in the default temporary file
    /// directory (platform dependent). These directories are often patrolled by temporary file
    /// cleaners so only use this method if you're *positive* that the temporary file cleaner won't
    /// delete your file.
    ///
    /// Reasons to use this method:
    ///   1. The file has a short lifetime and your temporary file cleaner is sane (doesn't delete
    ///      recently accessed files).
    ///   2. You trust every user on your system (i.e. you are the only user).
    ///   3. You have disabled your system's temporary file cleaner or verified that your system
    ///      doesn't have a temporary file cleaner.
    ///
    /// Reasons not to use this method:
    ///   1. You'll fix it later. No you won't.
    ///   2. You don't care about the security of the temporary file. If none of the "reasons to
    ///      use this method" apply, referring to a temporary file by name may allow an attacker
    ///      to create/overwrite your non-temporary files. There are exceptions but if you don't
    ///      already know them, don't use this method.
    pub fn new() -> io::Result<NamedTempFile> {
        Self::new_in(&env::temp_dir())
    }

    /// Create a new temporary file in the specified directory.
    pub fn new_in<P: AsRef<Path>>(dir: P) -> io::Result<NamedTempFile> {
        CustomNamedTempFile::start().new_in(dir)
    }


    /// Get the temporary file's path.
    ///
    /// *SECURITY WARNING:* Only use this method if you're positive that a temporary file cleaner
    /// won't have deleted your file. Otherwise, the path returned by this method may refer to an
    /// attacker controlled file.
    #[inline]
    pub fn path(&self) -> &Path {
        &self.inner().path
    }

    /// Close and remove the temporary file.
    ///
    /// Use this if you want to detect errors in deleting the file.
    pub fn close(mut self) -> io::Result<()> {
        let NamedTempFileInner { path, file } = self.0.take().unwrap();
        drop(file);
        fs::remove_file(path)
    }

    /// Persist the temporary file at the target path.
    ///
    /// If a file exists at the target path, persist will atomically replace it. If this method
    /// fails, it will return `self` in the resulting PersistError.
    ///
    /// Note: Temporary files cannot be persisted across filesystems.
    ///
    /// *SECURITY WARNING:* Only use this method if you're positive that a temporary file cleaner
    /// won't have deleted your file. Otherwise, you might end up persisting an attacker controlled
    /// file.
    pub fn persist<P: AsRef<Path>>(mut self, new_path: P) -> Result<File, PersistError> {
        match imp::persist(&self.inner().path, new_path.as_ref(), true) {
            Ok(_) => Ok(self.0.take().unwrap().file),
            Err(e) => Err(PersistError { file: self, error: e }),
        }
    }

    /// Persist the temporary file at the target path iff no file exists there.
    ///
    /// If a file exists at the target path, fail. If this method fails, it will return `self` in
    /// the resulting PersistError.
    ///
    /// Note: Temporary files cannot be persisted across filesystems.
    /// Also Note: This method is not atomic. It can leave the original link to the temporary file
    /// behind.
    ///
    /// *SECURITY WARNING:* Only use this method if you're positive that a temporary file cleaner
    /// won't have deleted your file. Otherwise, you might end up persisting an attacker controlled
    /// file.
    pub fn persist_noclobber<P: AsRef<Path>>(mut self, new_path: P) -> Result<File, PersistError> {
        match imp::persist(&self.inner().path, new_path.as_ref(), false) {
            Ok(_) => Ok(self.0.take().unwrap().file),
            Err(e) => Err(PersistError { file: self, error: e }),
        }
    }
}

impl Drop for NamedTempFile {
    fn drop(&mut self) {
        if let Some(NamedTempFileInner { file, path }) = self.0.take() {
            drop(file);
            let _ = fs::remove_file(path);
        }
    }
}

impl Read for NamedTempFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }
}

impl Write for NamedTempFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (**self).write(buf)
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        (**self).flush()
    }
}

impl Seek for NamedTempFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        (**self).seek(pos)
    }
}

#[cfg(unix)]
impl std::os::unix::io::AsRawFd for NamedTempFile {
    #[inline]
    fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        (**self).as_raw_fd()
    }
}

#[cfg(windows)]
impl std::os::windows::io::AsRawHandle for NamedTempFile {
    #[inline]
    fn as_raw_handle(&self) -> std::os::windows::io::RawHandle {
        (**self).as_raw_handle()
    }
}



#[derive(Debug, Clone)]
pub struct CustomNamedTempFile<'a , 'b> {
    random_len: usize,
    prefix: &'a str,
    postfix: &'b str
}

impl <'a , 'b>CustomNamedTempFile<'a , 'b> {

    /// Start building a CustomNamedTempFile. See `new` for more information.
    pub fn start() -> Self {
        CustomNamedTempFile {
            random_len: ::NUM_RAND_CHARS,
            prefix: ".",
            postfix: ""
        }
    }

    /// Set prefix to the CustomNamedTempFile builder. The prefix MUST NOT contain any '/'s.
    /// The default value is ".".
    /// See `new` for more information.
    pub fn prefix(&mut self, prefix: &'a str) -> &mut Self {
        // TODO check '/'
        self.prefix = prefix;
        self
    }

    /// Set postfix to the CustomNamedTempFile builder. The post MUST NOT contain any '/'s.
    /// The default value is ""
    /// See `new` for more information.
    pub fn postfix(&mut self, postfix: &'b str) -> &mut Self {
        // TODO check '/'
        self.postfix = postfix;
        self
    }

    /// Set the length of random generated part of file name to the CustomNamedTempFile builder.
    /// The default value is
    /// It is recommended to set it larger than 5.
    /// See `new` for more information.
    pub fn rand(&mut self, rand: usize) -> &mut Self {
        self.random_len = rand;
        self
    }

    /// New a new temporary file with Custom format.
    ///
    /// # Examples
    /// ```no_run
    /// use tempfile::CustomNamedTempFile;
    ///
    /// let named_temp_file = CustomNamedTempFile::start()
    ///                         .prefix("hogehoge")
    ///                         .postfix(".rs")
    ///                         .rand(5)
    ///                         .new_in("/tmp")
    ///                         .unwrap();
    /// println!("{:?}", named_temp_file);        //Something like "NamedTempFile(\"/tmp/hogehoge65R8Y.rs\")"
    /// ```
    pub fn new(&self) -> io::Result<NamedTempFile> {
        self.new_in(&env::temp_dir())
    }

    /// New a new temporary file from the template in the specified directory.
    pub fn new_in<P: AsRef<Path>>(&self, dir: P) -> io::Result<NamedTempFile> {
        for _ in 0..::NUM_RETRIES {
            let path = dir.as_ref().join(&self.tmpname());
            return match imp::create_named(&path) {
                Ok(file) => Ok(NamedTempFile(Some(NamedTempFileInner { path: path, file: file, }))),
                Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(e) => Err(e),
            }
        }
        Err(io::Error::new(io::ErrorKind::AlreadyExists,
                           "too many temporary directories already exist"))

    }

    // for crate internal usage.
    pub fn tmpname(&self) -> OsString {
        let mut bytes = Vec::new();
        for _ in 0..self.random_len {
            bytes.push(b'.');
        }
        rand::thread_rng().fill_bytes(&mut bytes[..]);

        for byte in bytes.iter_mut() {
            *byte = match *byte % 62 {
                v @ 0...9 => (v + '0' as u8),
                v @ 10...35 => (v - 10 + 'a' as u8),
                v @ 36...61 => (v - 36 + 'A' as u8),
                _ => unreachable!(),
            }
        }
        let s = unsafe { ::std::str::from_utf8_unchecked(&bytes) };

        let res = format!("{}{}{}", self.prefix, s, self.postfix);
        // TODO: Use OsStr::to_cstring (convert)
        OsStr::new(&res[..]).to_os_string()
    }

}
