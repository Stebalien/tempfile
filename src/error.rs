use std::{error, io, fmt};
use std::path::PathBuf;

#[derive(Debug)]
struct PathError {
    path: PathBuf,
    err: io::Error,
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} at path {:?}", self.err, self.path)
    }
}

impl error::Error for PathError {
    fn description(&self) -> &str {
        self.err.description()
    }
    
    fn cause(&self) -> Option<&error::Error> {
        self.err.cause()
    }
}

pub(crate) trait IoErrorExt {
    fn with_path<P>(self, path: P) -> Self where P: Into<PathBuf>;
}

impl IoErrorExt for io::Error {
    fn with_path<P>(self, path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        io::Error::new(self.kind(), PathError {
            path: path.into(),
            err: self,
        })
    }
}
