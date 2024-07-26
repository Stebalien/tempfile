# Changelog

## 3.11.0

- Add the ability to override the default temporary directory. This API shouldn't be used in general, but there are some cases where it's unavoidable.

## 3.10.1

- Handle potential integer overflows in 32-bit systems when seeking/truncating "spooled" temporary files past 4GiB (2³²).
- Handle a theoretical 32-bit overflow when generating a temporary file name larger than 4GiB. Now it'll panic (on allocation failure) rather than silently succeeding due to wraparound.

Thanks to @stoeckmann for finding and fixing both of these issues.

## 3.10.0

- Drop `redox_syscall` dependency, we now use `rustix` for Redox.
- Add `Builder::permissions` for setting the permissions on temporary files and directories (thanks to @Byron).
- Update rustix to 0.38.31.
- Update fastrand to 2.0.1.

## 3.9.0

- Updates windows-sys to 0.52
- Updates minimum rustix version to 0.38.25

## 3.8.1

- Update rustix to fix a potential panic on `persist_noclobber` on android.
- Update redox_syscall to 0.4 (on redox).
- Fix some docs typos.

## 3.8.0

- Added `with_prefix` and `with_prefix_in` to `TempDir` and `NamedTempFile` to make it easier to create temporary files/directories with nice prefixes.
- Misc cleanups.

## 3.7.1

- Tempfile builds on haiku again.
- Under the hood, we've switched from the unlinkat/linkat syscalls to the regular unlink/link syscalls where possible.

## 3.7.0

BREAKING: This release updates the MSRV to 1.63. This isn't an API-breaking change (so no major
release) but it's still a breaking change for some users.

- Update fastrand from 1.6 to 2.0
- Update rustix to 0.38
- Updates the MSRV to 1.63.
- Provide AsFd/AsRawFd on wasi.

## 3.6.0

- Update windows-sys to 0.48.
- Update rustix min version to 0.37.11
- Forward some `NamedTempFile` and `SpooledTempFile` methods to the underlying `File` object for
  better performance (especially vectorized writes, etc.).
- Implement `AsFd` and `AsHandle`.
- Misc documentation fixes and code cleanups.

## 3.5.0

- Update rustix from 0.36 to 0.37.1. This makes wasi work on rust stable
- Update `windows-sys`, `redox_syscall`
- BREAKING: Remove the implementation of `Write for &NamedTempFile<F> where &F: Write`. Unfortunately, this can cause compile issues in unrelated code (https://github.com/Stebalien/tempfile/issues/224).

## 3.4.0

SECURITY: Prior `tempfile` releases depended on `remove_dir_all` version 0.5.0 which was vulnerable to a [TOCTOU race](https://github.com/XAMPPRocky/remove_dir_all/security/advisories/GHSA-mc8h-8q98-g5hr). This same race is present in rust versions prior to 1.58.1.

Features:

- Generalized temporary files: `NamedTempFile` can now abstract over different kinds of files (e.g.,
  unix domain sockets, pipes, etc.):
    - Add `Builder::make` and `Builder::make_in` for generalized temp file
    creation.
    - Add `NamedTempFile::from_parts` to complement `NamedTempFile::into_parts`.
    - Add generic parameter to `NamedTempFile` to support wrapping non-File types.

Bug Fixes/Improvements:

- Don't try to create a temporary file multiple times if the file path has been fully specified by
  the user (no random characters).
- `NamedTempFile::persist_noclobber` is now always atomic on linux when `renameat_with` is
  supported. Previously, it would first link the new path, then unlink the previous path.
- Fix compiler warnings on windows.

Trivia:

- Switch from `libc` to `rustix` on wasi/unix. This now makes direct syscalls instead of calling
  through libc.
- Remove `remove_dir_all` dependency. The rust standard library has optimized their internal version
  significantly.
 - Switch to official windows-sys windows bindings.

Breaking:

 - The minimum rust version is now `1.48.0`.
 - Mark most functions as `must_use`.
 - Uses direct syscalls on linux by default, instead of libc.
 - The new type parameter in `NamedTempFile` may lead to type inference issues in some cases.

## 3.3.0

Features:

- Replace rand with fastrand for a significantly smaller dependency tree. Cryptographic randomness
  isn't necessary for temporary file names, and isn't all that helpful either.
- Add limited WASI support.
- Add a function to extract the inner data from a `SpooledTempFile`.

Bug Fixes:

- Make it possible to persist unnamed temporary files on linux by removing the `O_EXCL` flag.
- Fix redox minimum crate version.

## 3.2.0

Features:

- Bump rand dependency to `0.8`.
- Bump cfg-if dependency to `1.0`

Other than that, this release mostly includes small cleanups and simplifications.

Breaking: The minimum rust version is now `1.40.0`.

## 3.1.0

Features:

- Bump rand dependency to `0.7`.

Breaking: The minimum rust version is now `1.32.0`.

## 3.0.9

Documentation:

- Add an example for reopening a named temporary file.
- Flesh out the security documentation.

Features:

- Introduce an `append` option to the builder.
- Errors:
  - No longer implement the soft-deprecated `description`.
  - Implement `source` instead of `cause`.

Breaking: The minimum rust version is now 1.30.

## 3.0.8

This is a bugfix release.

Fixes:

- Export `PathPersistError`.
- Fix a bug where flushing a `SpooledTempFile` to disk could fail to write part
  of the file in some rare, yet-to-reproduced cases.

## 3.0.7

Breaking:

- `Builder::prefix` and `Builder::suffix` now accept a generic `&AsRef<OsStr>`.
  This could affect type inference.
- Temporary files (except unnamed temporary files on Windows and Linux >= 3.11)
  now use absolute path names. This will break programs that create temporary
  files relative to their current working directory when they don't have the
  search permission (x) on some ancestor directory. This is only likely to
  affect programs with strange chroot-less filesystem sandboxes. If you believe
  you're affected by this issue, please comment on #40.

Features:

- Accept anything implementing `&AsRef<OsStr>` in the builder: &OsStr, &OsString, &Path, etc.

Fixes:

- Fix LFS support.
- Use absolute paths for named temporary files to guard against changes in the
  current directory.
- Use absolute paths when creating unnamed temporary files on platforms that
  can't create unlinked or auto-deleted temporary files. This fixes a very
  unlikely race where the current directory could change while the temporary
  file is being created.

Misc:

- Use modern stdlib features to avoid custom unsafe code. This reduces the
  number of unsafe blocks from 12 to 4.

## 3.0.6

- Don't hide temporary files on windows, fixing #66 and #69.

## 3.0.5

Features:

- Added a spooled temporary file implementation. This temporary file variant
  starts out as an in-memory temporary file but "rolls-over" onto disk when it
  grows over a specified size (#68).
- Errors are now annotated with paths to make debugging easier (#73).

Misc:

- The rand version has been bumped to 0.6 (#74).

Bugs:

- Tempfile compiles again on Redox (#75).

## 3.0.4

- Now compiles on unsupported platforms.

## 3.0.3

- update rand to 0.5

## 3.0.2

- Actually *delete* temporary files on non-Linux unix systems (thanks to
@oliverhenshaw for the fix and a test case).

## 3.0.1

- Restore NamedTempFile::new_in

## 3.0.0

- Adds temporary directory support (@KodrAus)
- Allow closing named temporary files without deleting them (@jasonwhite)

## 2.2.0

- Redox Support

## 2.1.6

- Remove build script and bump minimum rustc version to 1.9.0

## 2.1.5

- Don't build platform-specific dependencies on all platforms.
- Cleanup some documentation.

## 2.1.4

- Fix crates.io tags. No interesting changes.

## 2.1.3

Export `PersistError`.

## 2.1.2

Add `Read`/`Write`/`Seek` impls on `&NamedTempFile`. This mirrors the
implementations on `&File`. One can currently just deref to a `&File` but these
implementations are more discoverable.

## 2.1.1

Add LFS Support.

## 2.1.0

- Implement `AsRef<File>` for `NamedTempFile` allowing named temporary files to
  be borrowed as `File`s.
- Add a method to convert a `NamedTempFile` to an unnamed temporary `File`.

## 2.0.1

- Arm bugfix

## 2.0.0

This release replaces `TempFile` with a `tempfile()` function that returns
`std::fs::File` objects. These are significantly more useful because most rust
libraries expect normal `File` objects.

To continue supporting shared temporary files, this new version adds a
`reopen()` method to `NamedTempFile`.
