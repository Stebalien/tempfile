extern crate tempdir;
extern crate tempfile;
use tempfile::{NamedTempFile, NamedTempFileOptions};
use std::env;
use std::io::{Write, Read, Seek, SeekFrom};
use std::fs;
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

#[test]
fn test_customnamed() {
    let tmpfile = NamedTempFileOptions::new()
        .prefix("tmp")
        .suffix(&".rs".to_string())
        .rand_bytes(12)
        .create()
        .unwrap();
    let name = tmpfile.path().file_name().unwrap().to_str().unwrap();
    assert!(name.starts_with("tmp"));
    assert!(name.ends_with(".rs"));
    assert_eq!(name.len(), 18);
}


#[test]
fn test_reopen() {
    let source = NamedTempFile::new().unwrap();
    let mut first = source.reopen().unwrap();
    let mut second = source.reopen().unwrap();
    drop(source);

    write!(first, "abcde").expect("write failed");
    let mut buf = String::new();
    second.read_to_string(&mut buf).unwrap();
    assert_eq!("abcde", buf);
}

#[test]
fn test_to_file() {
    let mut file = NamedTempFile::new().unwrap();
    let path = file.path().to_owned();
    write!(file, "abcde").expect("write failed");

    assert!(path.exists());
    let mut file: File = file.into();
    assert!(!path.exists());

    file.seek(SeekFrom::Start(0)).unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    assert_eq!("abcde", buf);
}


#[test]
fn test_immut() {
    let tmpfile = NamedTempFile::new().unwrap();
    (&tmpfile).write_all(b"abcde").unwrap();
    (&tmpfile).seek(SeekFrom::Start(0)).unwrap();
    let mut buf = String::new();
    (&tmpfile).read_to_string(&mut buf).unwrap();
    assert_eq!("abcde", buf);
}

#[test]
fn deleted_noclobber() {
    let temp_dir = tempdir::TempDir::new("tempfile-deleted").unwrap();
    let tmp = NamedTempFile::new_deleted(&temp_dir).unwrap();

    // Will only actually be deleted on (modern) linux:
    #[cfg(target_os = "linux")]
    assert_eq!(0, fs::read_dir(&temp_dir).unwrap().count());

    let dest = temp_dir.path().to_path_buf().join("foo");

    tmp.persist_noclobber(&dest).unwrap();
    assert!(dest.exists());
}

#[test]
fn deleted() {
    let temp_dir = tempdir::TempDir::new("tempfile-deleted").unwrap();
    let tmp = NamedTempFile::new_deleted(&temp_dir).unwrap();

    let dest = temp_dir.path().join("foo");

    tmp.persist(&dest).unwrap();
    assert!(dest.exists());
}
