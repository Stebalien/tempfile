extern crate tempfile;
use std::io::{Read, Seek, SeekFrom, Write};
use std::fs;

#[test]
fn test_basic() {
    let mut tmpfile = tempfile::tempfile().unwrap();
    write!(tmpfile, "abcde").unwrap();
    tmpfile.seek(SeekFrom::Start(0)).unwrap();
    let mut buf = String::new();
    tmpfile.read_to_string(&mut buf).unwrap();
    assert_eq!("abcde", buf);
}

#[test]
fn test_cleanup() {
    let tmpdir = tempfile::tempdir().unwrap();
    {
        let mut tmpfile = tempfile::tempfile_in(&tmpdir).unwrap();
        write!(tmpfile, "abcde").unwrap();
    }
    let num_files = fs::read_dir(&tmpdir).unwrap().count();
    assert!(num_files == 0);
}
