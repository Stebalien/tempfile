use super::imp;
use super::util;

struct TempDir {
    path: PathBuf,
    inner: TempDirImp
}

impl TempDir {
    pub fn new() -> TempDir {
        // TODO
    }
    pub fn new_in<P: AsRef<Path>>(dir: P) -> io::Result<TempDir> {
        let path = dir.as_ref().join(&util::tmpname());
        path.push(""); // Make it a directory.
        // TODO
    }

    // TODO: Real Options
    /// Securely create a file in this directory.
    ///
    /// TODO: SECURITY WARNING
    pub fn open_file<P: AsRef<Path>>(&self, path: P, create: bool) -> io::Result<File> {
        self.inner.open_file(path.as_ref(), create)
    }

    pub fn remove_file<P: AsRef<Path>>(path: P) -> io::Result<()> {
        self.inner.remove_file(path.as_ref());
    }

    /// Remove the specified directory.
    ///
    /// If recurse is true, this function recursivly deletes all files and directories under the
    /// specified directory. When false, this function will refuse to delete non-empty directories.
    ///
    /// Note: If recurse is true, this function is not atomic. If it failes to delete a
    /// subdirectory/file, this function will return an error without proceeding.
    pub fn remove_dir<P: AsRef<Path>>(&self, path: P, recurse: bool) -> io::Result<()> {
        self.inner.remove_dir(path.as_ref(), recurse);
    }

    pub fn rename<P1: AsRef<Path>, P2: AsRef<Path>>(&self, from: P1, to: P2) -> io::Result<()> {
        self.inner.rename(from.as_ref(), to.as_ref());
    }

    /// Persist the temporary directory at the target path.
    ///
    /// If an empty directory exists at the target path, persist will atomically replace it. If
    /// this method fails, it will return `self` in the resulting PersistError.
    ///
    /// Note: Temporary directories cannot be persisted across filesystems.
    ///
    /// *SECURITY WARNING:* Only use this method if you're positive that a temporary file cleaner
    /// won't have deleted your directory. Otherwise, you might end up persisting an attacker
    /// controlled directory.
    #[inline]
    pub fn persist<P: AsRef<Path>>(mut self, new_path: P) -> Result<File, PersistError> {
        match fs::rename(&self.inner().path, new_path) {
            Ok(_) => Ok(self.0.take().unwrap().file),
            Err(e) => Err(PersistError { file: self, error: e }),
        }
    }
}
