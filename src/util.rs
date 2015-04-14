use std::ffi::OsString;
use ::rand;
use ::rand::Rng;

pub fn tmpname() -> OsString {
    let mut bytes = vec!['.' as u8; 7];
    rand::thread_rng().fill_bytes(&mut bytes[1..]);

    for byte in bytes[1..].iter_mut() {
        *byte = match *byte % 62 {
            v @ 0...9 => (v + '0' as u8),
            v @ 10...35 => (v - 10 + 'a' as u8),
            v @ 36...61 => (v - 36 + 'A' as u8),
            _ => unreachable!(),
        }
    }
    OsString::from_bytes(bytes).unwrap()
}

