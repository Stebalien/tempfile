extern crate tempfile;
use tempfile::TempFile;
use std::io::{Write, Read, Seek, SeekFrom};

#[test]
fn test_basic() {
    let mut tmpfile = TempFile::new_in("/tmp").unwrap();
    write!(tmpfile, "abcde").unwrap();
    tmpfile.seek(SeekFrom::Start(0)).unwrap();
    let mut buf = String::new();
    tmpfile.read_to_string(&mut buf).unwrap();
    assert_eq!("abcde", buf);
}

#[test]
fn test_share() {
    let mut tmpfile = TempFile::new_in("/tmp").unwrap();
    write!(tmpfile, "abcde").unwrap();
    let mut shared = tmpfile.share().unwrap();
    let mut buf = String::new();
    shared.read_to_string(&mut buf).unwrap();
    assert_eq!("abcde", buf);
}
