use std::ffi::{OsString, OsStr};
use std::str::FromStr;
use ::rand;
use ::rand::Rng;

pub struct Template {
    random_len: usize,
    prefix: String,
    suffix: String
}

impl FromStr for Template {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let start = s.find('X').unwrap_or(0);
        let end =  match s.rfind('X') {
            Some(i) => i + 1,
            None => start
        };
        if s[start..end].contains(|c| c != 'X') {
            return Err("The number of the sequence of 'X' must be 1.".to_string())
        }
        Ok(Template{
            random_len: end - start,
            prefix: s[..start].to_string(),
            suffix: s[end..].to_string()
        })
    }
}

impl Template {
    pub fn new<S: Into<String>>(random_len: usize, prefix: S, suffix: S) -> Template {
        Template{random_len: random_len, prefix: prefix.into(), suffix: suffix.into()}
    }
}


pub fn tmpname(template: &Template) -> OsString {
    let mut bytes = Vec::new();
    for _ in 0..template.random_len {
        bytes.push(b'.');
    }
    rand::thread_rng().fill_bytes(&mut bytes[..]);

    for byte in bytes.iter_mut() {
        *byte = match *byte % 62 {
            v @ 0...9 => (v + '0' as u8),
            v @ 10...35 => (v - 10 + 'a' as u8),
            v @ 36...61 => (v - 36 + 'A' as u8),
            _ => unreachable!(),
        }
    }
    let s = unsafe { ::std::str::from_utf8_unchecked(&bytes) };

    let res = format!("{}{}{}", template.prefix, s, template.suffix);
    // TODO: Use OsStr::to_cstring (convert)
    OsStr::new(&res[..]).to_os_string()
}

#[test]
fn test_tmpname() {
    assert!(tmpname(&"foobar".parse().unwrap()).to_str() == Some("foobar"));
    // to satisfy life time restriction, temporary binding is needed.
    let tmp = tmpname(&"fooXXXbar".parse().unwrap());
    let tmpstropt = tmp.to_str();
    assert!(tmpstropt.is_some());
    let tmpstr = tmpstropt.unwrap();
    assert!(tmpstr.len() == 9);
    assert!(tmpstr.starts_with("foo"));
    assert!(tmpstr.ends_with("bar"));
}
