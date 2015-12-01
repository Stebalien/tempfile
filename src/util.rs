use std::ffi::OsString;
use std::iter;
use rand;
use rand::Rng;

pub fn tmpname(prefix: &str, suffix: &str, rand_len: usize) -> OsString {
    let mut buf = String::with_capacity(prefix.len() + suffix.len() + rand_len);
    buf.push_str(prefix);
    buf.extend(iter::repeat('X').take(rand_len));
    buf.push_str(suffix);

    // Randomize.
    unsafe {
        // We guarantee utf8.
        let bytes = &mut buf.as_mut_vec()[prefix.len()..prefix.len() + rand_len];
        rand::thread_rng().fill_bytes(bytes);
        for byte in bytes.iter_mut() {
            *byte = match *byte % 62 {
                v @ 0...9 => (v + '0' as u8),
                v @ 10...35 => (v - 10 + 'a' as u8),
                v @ 36...61 => (v - 36 + 'A' as u8),
                _ => unreachable!(),
            }
        }
    }
    OsString::from(buf)
}
