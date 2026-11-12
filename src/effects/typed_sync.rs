//! Test support for the marks' x87 blocks that work on typed statics: the differential tests
//! (`asm_test`) run the original assembly over a test buffer standing for the marks' block at
//! the test buffer `mk` and the plain Rust version over the typed statics; around the Rust
//! version the statics hold the buffer's bytes, and the bytes it leaves go back to the buffer
//! (`store::test_lock` is held by `differential`).

use crate::effects::asm_test::Operand;
use crate::effects::store::START;

/// A typed static and where its first byte is in the marks' block (IDA).
pub struct Overlay {
    home: *mut u8,
    len: usize,
    start: usize,
}

/// The overlay of `st`, a typed static whose first byte is IDA's address `start`.
pub fn overlay<T>(st: &'static T, start: usize) -> Overlay {
    Overlay {
        home: std::ptr::from_ref::<T>(st) as *mut u8,
        len: size_of::<T>(),
        start,
    }
}

/// Runs `f` with each overlay's static holding the bytes the buffer `mk` has there; what the
/// statics hold afterwards goes into the buffer, and the statics are put back as they were.
pub fn with_overlays<R>(mk: Operand, overlays: &[Overlay], f: impl FnOnce() -> R) -> R {
    let at = |o: &Overlay| mk.at(o.start.wrapping_sub(START), o.len);
    // Every overlay must fit the buffer before any is touched.
    for o in overlays {
        at(o);
    }
    let saved: Vec<Vec<u8>> = overlays
        .iter()
        .map(|o| unsafe { std::slice::from_raw_parts(o.home, o.len) }.to_vec())
        .collect();
    for o in overlays {
        unsafe { std::ptr::copy_nonoverlapping(at(o), o.home, o.len) };
    }
    let out = f();
    for (o, bytes) in overlays.iter().zip(&saved) {
        unsafe {
            std::ptr::copy_nonoverlapping(o.home, at(o), o.len);
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), o.home, o.len);
        }
    }
    out
}
