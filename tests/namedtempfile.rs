extern crate tempfile;
use tempfile::NamedTempFile;
use std::env;
use std::io::{Write, Read, Seek, SeekFrom};
use std::fs::File;
use std::path::Path;

fn exists<P: AsRef<Path>>(path: P) -> bool {
    std::fs::metadata(path.as_ref()).is_ok()
}


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
    assert!(exists(&path));
    drop(tmpfile);
    assert!(!exists(&path));
}

#[test]
fn test_persist() {
    let mut tmpfile = NamedTempFile::new().unwrap();
    let old_path = tmpfile.path().to_path_buf();
    let persist_path = env::temp_dir().join("persisted_temporary_file");
    write!(tmpfile, "abcde").unwrap();
    {
        assert!(exists(&old_path));
        let mut f = tmpfile.persist(&persist_path).unwrap();
        assert!(!exists(&old_path));

        // Check original file
        f.seek(SeekFrom::Start(0)).unwrap();
        let mut buf = String::new();
        f.read_to_string(&mut buf).unwrap();
        assert_eq!("abcde", buf);
    }

    {
        // Try opening it at the new path.
        let mut f = File::open(&persist_path).unwrap();
        f.seek(SeekFrom::Start(0)).unwrap();
        let mut buf = String::new();
        f.read_to_string(&mut buf).unwrap();
        assert_eq!("abcde", buf);
    }
    std::fs::remove_file(&persist_path).unwrap();
}

#[test]
fn test_persist_noclobber() {
    let mut tmpfile = NamedTempFile::new().unwrap();
    let old_path = tmpfile.path().to_path_buf();
    let persist_target = NamedTempFile::new().unwrap();
    let persist_path = persist_target.path().to_path_buf();
    write!(tmpfile, "abcde").unwrap();
    assert!(exists(&old_path));
    {
        tmpfile = tmpfile.persist_noclobber(&persist_path).unwrap_err().into();
        assert!(exists(&old_path));
        std::fs::remove_file(&persist_path).unwrap();
        drop(persist_target);
    }
    tmpfile.persist_noclobber(&persist_path).unwrap();
    // Try opening it at the new path.
    let mut f = File::open(&persist_path).unwrap();
    f.seek(SeekFrom::Start(0)).unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();
    assert_eq!("abcde", buf);
    std::fs::remove_file(&persist_path).unwrap();
}
