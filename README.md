tempfile
========

![Build Status](https://travis-ci.org/Stebalien/tempfile.svg)

A secure cross-platform temporary file library for rust. In addition to creating
temporary files, this library also allows users to securely open multiple
independent references to the same temporary file (useful for consumer/producer
patterns and surprisingly difficult to implement securely).

Example
-------

```rust
use tempfile::TempFile;
use std::io::{Write, Read, Seek, SeekFrom};

// Write
let mut tmpfile = TempFile::new().unwrap();
write!(tmpfile, "Hello World!").unwrap();

// Seek to start
tmpfile.seek(SeekFrom::Start(0)).unwrap();

// Read
let mut buf = String::new();
tmpfile.read_to_string(&mut buf).unwrap();
assert_eq!("abcde", buf);
```

Documentation
-------------

https://stebalien.github.com/tempfile/tempfile/

Security
--------

This library should be race free, reliable, and secure in the presence of
pathological temporary file cleaners.
