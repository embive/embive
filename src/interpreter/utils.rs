//! Utility functions for the interpreter

#[inline]
#[cold]
fn cold() {}

/// A hint that the branch is likely to be taken.
#[inline]
pub fn likely(b: bool) -> bool {
    if !b {
        cold()
    }
    b
}

/// A hint that the branch is unlikely to be taken.
#[inline]
pub fn unlikely(b: bool) -> bool {
    if b {
        cold()
    }
    b
}
