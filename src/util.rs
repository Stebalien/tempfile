use std::ffi::{OsString, OsStr};
use ::rand;
use ::rand::Rng;

pub fn tmpname() -> OsString {
    let mut bytes = [b'.'; ::NUM_RAND_CHARS+1];
    rand::thread_rng().fill_bytes(&mut bytes[1..]);

    for byte in bytes[1..].iter_mut() {
        *byte = match *byte % 62 {
            v @ 0...9 => (v + '0' as u8),
            v @ 10...35 => (v - 10 + 'a' as u8),
            v @ 36...61 => (v - 36 + 'A' as u8),
            _ => unreachable!(),
        }
    }
    // TODO: Use OsStr::to_cstring (convert)
    OsStr::new(unsafe { ::std::str::from_utf8_unchecked(&bytes) }).to_os_string()
}

