extern crate tempfile;
use tempfile::TempFile;
use std::io::{Write, Read};

#[test]
fn test() {
    let mut tmpfile = TempFile::new_in("/tmp").unwrap();
    write!(tmpfile, "abcde").unwrap();
    let mut shared = tmpfile.share().unwrap();
    let mut buf = String::new();
    shared.read_to_string(&mut buf).unwrap();
    assert_eq!("abcde", buf);
}
