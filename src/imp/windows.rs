use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::{FromRawHandle, AsRawHandle, RawHandle};
use std::path::Path;
use std::io;
use std::fs::{File, OpenOptions};
use ::libc::{self, DWORD, HANDLE};
use ::util::tmpname;

const ACCESS: DWORD     = libc::FILE_GENERIC_READ
                        | libc::FILE_GENERIC_WRITE;
const SHARE_MODE: DWORD = libc::FILE_SHARE_DELETE
                        | libc::FILE_SHARE_READ
                        | libc::FILE_SHARE_WRITE;
const FLAGS_DEL: DWORD  = libc::FILE_ATTRIBUTE_HIDDEN
                        | libc::FILE_ATTRIBUTE_TEMPORARY
                        | libc::FILE_FLAG_DELETE_ON_CLOSE; 
const FLAGS: DWORD      = libc::FILE_ATTRIBUTE_HIDDEN
                        | libc::FILE_ATTRIBUTE_TEMPORARY;

extern "system" {
    // TODO: move to external crate.
    fn ReOpenFile(hOriginalFile: HANDLE,
                  dwDesiredAccess: DWORD,
                  dwShareMode: DWORD,
                  dwFlags: DWORD) -> HANDLE;
}


pub fn create_named(path: &Path) -> io::Result<File> {
    OpenOptions::new().desired_access(ACCESS as i32)
        .share_mode(SHARE_MODE as i32)
        .creation_disposition(libc::CREATE_NEW as i32)
        .flags_and_attributes(FLAGS as i32).open(path)
}

pub fn create(dir: &Path) -> io::Result<File> {
    let mut opts = OpenOptions::new();
    opts.desired_access(ACCESS as i32)
        .share_mode(SHARE_MODE as i32)
        .creation_disposition(libc::CREATE_NEW as i32)
        .flags_and_attributes(FLAGS_DEL as i32);
    for _ in 0..::NUM_RETRIES {
        return match opts.open(&dir.join(&tmpname())) {
            Ok(f) => Ok(f),
            Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => Err(e),
        };
    }
    Err(io::Error::new(io::ErrorKind::AlreadyExists, "too many temporary directories already exist"))
}

pub fn create_shared(dir: &Path, count: usize) -> io::Result<Vec<File>> {
    if count == 0 {
        return Ok(vec![]);
    }
    let first = try!(create(dir));
    let mut files: Vec<File> = try!((1..count).map(|_| reopen(&first)).collect());
    files.push(first);
    Ok(files)
}

pub fn reopen(f: &File) -> io::Result<File> {
    let h = f.as_raw_handle();
    unsafe {
        let h = ReOpenFile(h as HANDLE, ACCESS, SHARE_MODE, 0);
        if h == libc::INVALID_HANDLE_VALUE {
            Err(io::Error::last_os_error())
        } else {
            Ok(FromRawHandle::from_raw_handle(h as RawHandle))
        }
    }
}
