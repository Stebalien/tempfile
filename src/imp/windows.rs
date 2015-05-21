use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::{FromRawHandle, AsRawHandle, RawHandle};
use std::path::Path;
use std::io;
use std::ptr;
use std::fs::File;
use ::libc::{self, DWORD, HANDLE, CreateFileW};
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


fn to_utf16(s: &Path) -> Vec<u16> {
    s.as_os_str().encode_wide().chain(Some(0).into_iter()).collect()
}

extern "system" {
    // TODO: move to external crate.
    fn ReOpenFile(hOriginalFile: HANDLE,
                  dwDesiredAccess: DWORD,
                  dwShareMode: DWORD,
                  dwFlags: DWORD) -> HANDLE;
}


fn win_create(path: &Path,
                     access: DWORD,
                     share_mode: DWORD,
                     disp: DWORD,
                     flags: DWORD) -> io::Result<File> {

    let path = to_utf16(path);
    let handle = unsafe {
        CreateFileW(
            path.as_ptr(),
            access,
            share_mode,
            0 as *mut _,
            disp,
            flags,
            ptr::null_mut())
    };
    if handle == libc::INVALID_HANDLE_VALUE {
        Err(io::Error::last_os_error())
    } else {
        Ok(unsafe { File::from_raw_handle(handle as RawHandle) })
    }
}

pub fn create_named(path: &Path) -> io::Result<File> {
    win_create(path, ACCESS, SHARE_MODE, libc::CREATE_NEW, FLAGS)
}

pub fn create(dir: &Path) -> io::Result<File> {
    for _ in 0..::NUM_RETRIES {
        return match win_create(
            &dir.join(&tmpname()),
            ACCESS,
            SHARE_MODE,
            libc::CREATE_NEW,
            FLAGS_DEL)
        {
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

fn reopen(f: &File) -> io::Result<File> {
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
