#![feature(path_ext)]
extern crate tempfile;
use tempfile::NamedTempFile;
use std::io::{Write, Read, Seek, SeekFrom};
use std::fs::PathExt;

#[test]
fn test_basic() {
    let mut tmpfile = NamedTempFile::new().unwrap();
    write!(tmpfile, "abcde").unwrap();
    tmpfile.seek(SeekFrom::Start(0)).unwrap();
    let mut buf = String::new();
    tmpfile.read_to_string(&mut buf).unwrap();
    assert_eq!("abcde", buf);
}

#[test]
fn test_deleted() {
    let tmpfile = NamedTempFile::new().unwrap();
    let path = tmpfile.path().to_path_buf();
    assert!(path.exists());
    drop(tmpfile);
    assert!(!path.exists());
}

#[test]
fn test_into_path() {
    let tmpfile = NamedTempFile::new().unwrap();
    assert!(tmpfile.path().exists());
    let pathbuf = tmpfile.into_path();
    assert!(pathbuf.exists());
    std::fs::remove_file(pathbuf).unwrap();
}
