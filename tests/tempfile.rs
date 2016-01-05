extern crate tempfile;
use std::io::{Write, Read, Seek, SeekFrom};

#[test]
fn test_basic() {
    let mut tmpfile = tempfile::tempfile().unwrap();
    write!(tmpfile, "abcde").unwrap();
    tmpfile.seek(SeekFrom::Start(0)).unwrap();
    let mut buf = String::new();
    tmpfile.read_to_string(&mut buf).unwrap();
    assert_eq!("abcde", buf);
}
