use std::os::windows::fs::OpenOptionsExt;
use libc::DWORD;
use ::util::tmpname;

const ACCESS: DWORD     = libc::FILE_GENERIC_READ
                        | libc::FILE_GENERIC_WRITE;
const SHARE_MODE: DWORD = libc::FILE_SHARE_DELETE
                        | libc::FILE_SHARE_READ
                        | libc::FILE_SHARE_WRITE;
const FLAGS: DWORD      = libc::FILE_ATTRIBUTE_HIDDEN
                        | libc::FILE_ATTRIBUTE_TEMPORARY
                        | libc::FILE_FLAG_DELETE_ON_CLOSE; 

pub fn create(dir: &Path) -> io::Result<File> {
    let opts = OpenOptions::new()
        .desired_access(ACCESS)
        .share_mode(SHARE)
        .creation_disposition(libc::CREATE_NEW)
        .flags_and_attributes(ATTRS);

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
