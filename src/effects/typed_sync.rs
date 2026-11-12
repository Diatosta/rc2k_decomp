//! Test support for the marks' x87 blocks that work on typed statics: the differential tests
//! (`asm_test`) run the original assembly over a test buffer standing for the marks' block at
//! the test buffer `mk` and the plain Rust version over the typed statics; around the Rust
//! version the statics hold the buffer's bytes, and the bytes it leaves go back to the buffer
//! (`test_support::test_lock` is held by `differential`).

use crate::effects::asm_layout::{Laid, Run};
use crate::effects::asm_test::Operand;
use crate::effects::store::START;

/// A typed static and where its first byte is in the marks' block (IDA).
pub struct Overlay {
    home: *mut u8,
    len: usize,
    start: usize,
    /// The runs of the static's fields that were in the 1999 layout (`Laid`); none for a
    /// static whose bytes are the buffer's, one for one.
    runs: &'static [Run],
}

/// The overlay of `st`, a typed static whose first byte is IDA's address `start`.
pub fn overlay<T>(st: &'static T, start: usize) -> Overlay {
    Overlay {
        home: std::ptr::from_ref::<T>(st) as *mut u8,
        len: size_of::<T>(),
        start,
        runs: &[],
    }
}

/// The overlay of `st`, a typed static with a 1999 layout (`Laid`): its fields hold the
/// bytes the buffer has at their 1999 places.
pub fn overlay_laid<T: Laid>(st: &'static T) -> Overlay {
    Overlay {
        runs: T::RUNS,
        ..overlay(st, T::START)
    }
}

/// Runs `f` with each overlay's static holding the bytes the buffer `mk` has there; what the
/// statics hold afterwards goes into the buffer, and the statics are put back as they were.
pub fn with_overlays<R>(mk: Operand, overlays: &[Overlay], f: impl FnOnce() -> R) -> R {
    // The places in the buffer and in the static of every piece the buffer and the static
    // share: the whole static, or each run of a static with a 1999 layout.
    let pieces = |o: &Overlay| -> Vec<(*mut u8, *mut u8, usize)> {
        if o.runs.is_empty() {
            vec![(mk.at(o.start.wrapping_sub(START), o.len), o.home, o.len)]
        } else {
            o.runs
                .iter()
                .map(|r| {
                    let from = mk.at((o.start + r.at).wrapping_sub(START), r.len);
                    (from, o.home.wrapping_add(r.to), r.len)
                })
                .collect()
        }
    };
    // Every piece must fit the buffer before any is touched.
    let all: Vec<_> = overlays.iter().map(pieces).collect();
    let saved: Vec<Vec<u8>> = overlays
        .iter()
        .map(|o| unsafe { std::slice::from_raw_parts(o.home, o.len) }.to_vec())
        .collect();
    for (buffer, home, len) in all.iter().flatten() {
        unsafe { std::ptr::copy_nonoverlapping(*buffer, *home, *len) };
    }
    let out = f();
    for (o, (pieces, bytes)) in overlays.iter().zip(all.iter().zip(&saved)) {
        for (buffer, home, len) in pieces {
            unsafe { std::ptr::copy_nonoverlapping(*home, *buffer, *len) };
        }
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), o.home, o.len) };
    }
    out
}
