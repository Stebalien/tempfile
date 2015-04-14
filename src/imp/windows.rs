use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::{FromRawHandle, AsRawHandle};
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
const FLAGS: DWORD      = libc::FILE_ATTRIBUTE_HIDDEN
                        | libc::FILE_ATTRIBUTE_TEMPORARY
                        | libc::FILE_FLAG_DELETE_ON_CLOSE; 

extern "system" {
    // TODO: move to external crate.
    fn ReOpenFile(hOriginalFile: HANDLE,
                  dwDesiredAccess: DWORD,
                  dwShareMode: DWORD,
                  dwFlags: DWORD) -> HANDLE;
}

pub fn create(dir: &Path) -> io::Result<File> {
    let opts = OpenOptions::new()
        .desired_access(ACCESS)
        .share_mode(SHARE_MODE)
        .creation_disposition(libc::CREATE_NEW)
        .flags_and_attributes(FLAGS);

    loop {
        return match opts.open(dir.join(tmpname())) {
            Ok(f) => Ok(f),
            Err(e) if e.kind() == io::Error::AlreadyExists => continue,
            Err(e) => Err(e),
        };
    }
}

pub fn reopen(f: &File) -> io::Result<File> {
    let h = f.as_raw_handle();
    match ReOpenFile(h, ACCESS, SHARE_MODE, FLAGS) {
        libc::INVALID_HANDLE_VALUE => Err(io::Error::last_os_error()),
        h => Ok(FromRawHandle::from_raw_handle(h))
    }
}
