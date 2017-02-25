tempfile
========

Minimum Rust Version: 1.9.0

Linux: [![Build Status](https://travis-ci.org/Stebalien/tempfile.svg?branch=master)](https://travis-ci.org/Stebalien/tempfile)
Windows: [![Build status](https://ci.appveyor.com/api/projects/status/5q00b8rvvg46i5tf/branch/master?svg=true)](https://ci.appveyor.com/project/Stebalien/tempfile/branch/master)

A secure cross-platform temporary file library for rust. In addition to creating
temporary files, this library also allows users to securely open multiple
independent references to the same temporary file (useful for consumer/producer
patterns and surprisingly difficult to implement securely).

Example
-------

```rust
extern crate tempfile;
use std::fs::File;
use std::io::{Write, Read, Seek, SeekFrom};

fn main() {
    // Write
    let mut tmpfile: File = tempfile::tempfile().unwrap();
    write!(tmpfile, "Hello World!").unwrap();

    // Seek to start
    tmpfile.seek(SeekFrom::Start(0)).unwrap();

    // Read
    let mut buf = String::new();
    tmpfile.read_to_string(&mut buf).unwrap();
    assert_eq!("Hello World!", buf);
}
```

Documentation
-------------

https://stebalien.github.com/tempfile/tempfile/
