use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::{FromRawHandle, AsRawHandle, RawHandle};
use std::path::Path;
use std::io;
use std::ptr;
use std::fs::{self, File};
use winapi::{self, DWORD, HANDLE};
use kernel32::{CreateFileW, ReOpenFile, SetFileAttributesW};
use named::CustomNamedTempFile;

const ACCESS: DWORD     = winapi::FILE_GENERIC_READ
                        | winapi::FILE_GENERIC_WRITE;
const SHARE_MODE: DWORD = winapi::FILE_SHARE_DELETE
                        | winapi::FILE_SHARE_READ
                        | winapi::FILE_SHARE_WRITE;
const FLAGS: DWORD      = winapi::FILE_ATTRIBUTE_HIDDEN
                        | winapi::FILE_ATTRIBUTE_TEMPORARY;


fn to_utf16(s: &Path) -> Vec<u16> {
    s.as_os_str().encode_wide().chain(Some(0).into_iter()).collect()
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
    if handle == winapi::INVALID_HANDLE_VALUE {
        Err(io::Error::last_os_error())
    } else {
        Ok(unsafe { File::from_raw_handle(handle as RawHandle) })
    }
}

pub fn create_named(path: &Path) -> io::Result<File> {
    win_create(path, ACCESS, SHARE_MODE, winapi::CREATE_NEW, FLAGS)
}

pub fn create(dir: &Path) -> io::Result<File> {
    for _ in 0..::NUM_RETRIES {
        return match win_create(
            &dir.join(&CustomNamedTempFile::new().tmpname()),
            ACCESS,
            SHARE_MODE,
            winapi::CREATE_NEW,
            FLAGS | winapi::FILE_FLAG_DELETE_ON_CLOSE)
        {
            Ok(f) => Ok(f),
            Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => Err(e),
        };
    }
    Err(io::Error::new(io::ErrorKind::AlreadyExists,
                       "too many temporary directories already exist"))
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
        if h == winapi::INVALID_HANDLE_VALUE {
            Err(io::Error::last_os_error())
        } else {
            Ok(FromRawHandle::from_raw_handle(h as RawHandle))
        }
    }
}

pub fn persist(old_path: &Path, new_path: &Path) -> io::Result<()> {
    // TODO: We should probably do this in one-shot using SetFileInformationByHandle but the API is
    // really painful.

    let old_path_w = to_utf16(old_path);

    // Don't succeed if this fails. We don't want to claim to have successfully persisted a file
    // still marked as temporary because this file won't have the same consistency guarantees.
    if unsafe { SetFileAttributesW(old_path_w.as_ptr(), winapi::FILE_ATTRIBUTE_NORMAL) == 0 } {
        return Err(io::Error::last_os_error());
    }
    return match fs::rename(old_path, new_path) {
        Ok(()) => Ok(()),
        Err(e) => {
            // If this fails, the temporary file is now un-hidden and no longer marked temporary
            // (slightly less efficient) but it will still work.
            let _ = unsafe { SetFileAttributesW(old_path_w.as_ptr(), FLAGS) };
            Err(e)
        }
    }
}
