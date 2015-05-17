extern crate tempfile;
use tempfile::TempFile;
use std::io::{Write, Read, Seek, SeekFrom};

#[test]
fn test_basic() {
    let mut tmpfile = TempFile::new().unwrap();
    write!(tmpfile, "abcde").unwrap();
    tmpfile.seek(SeekFrom::Start(0)).unwrap();
    let mut buf = String::new();
    tmpfile.read_to_string(&mut buf).unwrap();
    assert_eq!("abcde", buf);
}

#[test]
fn test_shared() {
    let mut tmpfiles = TempFile::shared(2).unwrap();
    write!(tmpfiles[0], "abcde").unwrap();
    let mut buf = String::new();
    tmpfiles[1].read_to_string(&mut buf).unwrap();
    assert_eq!("abcde", buf);
}
