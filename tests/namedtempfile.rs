extern crate tempfile;
use std::env;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use tempfile::{Builder, NamedTempFile};

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
    let tmpfile = Builder::new()
        .prefix("tmp")
        .suffix(&".rs".to_string())
        .rand_bytes(12)
        .tempfile()
        .unwrap();
    let name = tmpfile.path().file_name().unwrap().to_str().unwrap();
    assert!(name.starts_with("tmp"));
    assert!(name.ends_with(".rs"));
    assert_eq!(name.len(), 18);
}

#[test]
fn test_world_accessible() {
    #[cfg(unix)]
    fn assert_filemode(file: &File, world_accessible: bool) {
        use std::os::unix::fs::PermissionsExt;

        const MASK: u32 = 0o644;
        let value = if world_accessible { 0o644 } else { 0o600 };
        let mode = file.metadata().unwrap().permissions().mode();
        assert!(mode & MASK == value,
            "mode & MASK != value: 0o{:o} & 0o{:o} != 0o{:o}",
            mode, MASK, value);
    }
    #[cfg(not(unix))]
    fn assert_filemode(file: &File, world_accessible: bool) {
        let _ = (file, world_accessible);
    }
    for &world_accessible in &[None, Some(false), Some(true)] {
        let mut builder = Builder::new();
        if let Some(wa) = world_accessible {
            builder.world_accessible(wa);
        }
        let tempfile = builder.tempfile().unwrap();
        assert_filemode(tempfile.as_file(), world_accessible.unwrap_or(false));
    }
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
fn test_into_file() {
    let mut file = NamedTempFile::new().unwrap();
    let path = file.path().to_owned();
    write!(file, "abcde").expect("write failed");

    assert!(path.exists());
    let mut file = file.into_file();
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
fn test_temppath() {
    let mut tmpfile = NamedTempFile::new().unwrap();
    write!(tmpfile, "abcde").unwrap();

    let path = tmpfile.into_temp_path();
    assert!(path.is_file());
}

#[test]
fn test_temppath_persist() {
    let mut tmpfile = NamedTempFile::new().unwrap();
    write!(tmpfile, "abcde").unwrap();

    let tmppath = tmpfile.into_temp_path();

    let old_path = tmppath.to_path_buf();
    let persist_path = env::temp_dir().join("persisted_temppath_file");

    {
        assert!(exists(&old_path));
        tmppath.persist(&persist_path).unwrap();
        assert!(!exists(&old_path));
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
fn test_temppath_persist_noclobber() {
    let mut tmpfile = NamedTempFile::new().unwrap();
    write!(tmpfile, "abcde").unwrap();

    let mut tmppath = tmpfile.into_temp_path();

    let old_path = tmppath.to_path_buf();
    let persist_target = NamedTempFile::new().unwrap();
    let persist_path = persist_target.path().to_path_buf();

    assert!(exists(&old_path));

    {
        tmppath = tmppath.persist_noclobber(&persist_path).unwrap_err().into();
        assert!(exists(&old_path));
        std::fs::remove_file(&persist_path).unwrap();
        drop(persist_target);
    }

    tmppath.persist_noclobber(&persist_path).unwrap();

    // Try opening it at the new path.
    let mut f = File::open(&persist_path).unwrap();
    f.seek(SeekFrom::Start(0)).unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();
    assert_eq!("abcde", buf);
    std::fs::remove_file(&persist_path).unwrap();
}
