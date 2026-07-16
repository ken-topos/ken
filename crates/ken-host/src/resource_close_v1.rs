//! The sole errno-reporting resource-close boundary.
//!
//! The safe facade consumes the unique owner. `IntoRawFd` therefore leaves no
//! `OwnedFd` that could close again, and rustix specifies the raw fd as invalid
//! after `try_close` returns on either success or failure. Callers must first
//! remove the owner from its live table slot and commit the tombstone.

#![allow(unsafe_code)]

use std::io;
use std::os::fd::IntoRawFd;

pub(super) fn close(handle: crate::ResourceHandleV1) -> io::Result<()> {
    let raw_fd = handle.inner.0.into_raw_fd();
    // SAFETY: consuming the unique, non-cloneable owner produced this raw fd;
    // no alias or borrower survives, and this is its sole close attempt.
    unsafe { rustix::io::try_close(raw_fd) }.map_err(io::Error::from)
}
