use ::rand;
use ::rand::Rng;

pub fn tmpname() -> [u8; 7] {
    let mut bytes = ['.' as u8; 7];
    rand::thread_rng().fill_bytes(&mut bytes[1..]);

    for byte in bytes.iter_mut() {
        *byte = match *byte % 62 {
            v @ 0...9 => (v + '0' as u8),
            v @ 10...35 => (v + 'a' as u8),
            v @ 36...61 => (v + 'A' as u8),
            _ => unreachable!(),
        }
    }
    bytes
}

