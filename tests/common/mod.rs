use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

pub struct CWDGuard {
    #[allow(unused)]
    guard: MutexGuard<'static, ()>,
    old_cwd: PathBuf,
}

impl Drop for CWDGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.old_cwd);
    }
}

static CWD_LOCK: Mutex<()> = Mutex::new(());

/// Take an exclusive lock on changing the current directory.
///
/// The returned guard will restore the current directory before
/// releasing the lock.
///
/// N.B.: This obviously only works if you remember call this before
/// changing the current directory.
#[allow(unused)] // because not all tests use this.
pub fn cwd_lock() -> CWDGuard {
    let guard = CWD_LOCK.lock().unwrap();
    let old_cwd = std::env::current_dir().unwrap();
    CWDGuard { guard, old_cwd }
}

/// For the wasi platforms, `std::env::temp_dir` will panic. For those targets, configure the /tmp
/// directory instead as the base directory for temp files.
pub fn configure_wasi_temp_dir() {
    if cfg!(target_os = "wasi") {
        let _ = tempfile::env::override_temp_dir(Path::new("/tmp"));
    }
}
