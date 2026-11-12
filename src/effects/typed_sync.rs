//! Test support for the marks' x87 blocks that work on typed statics: the differential tests
//! (`asm_test`) run the original assembly over a test buffer standing for the marks' block at
//! `mk` (IDA 0x764F40) and the plain Rust version over the typed statics; around the Rust
//! version the statics hold the buffer's bytes, and the bytes it leaves go back to the buffer
//! (`store::test_lock` is held by `differential`).

// reason: the blocks over the race block are still compared on i686 only, against its
// assembly; on other targets only the recorded blocks use this harness.
#![cfg_attr(not(target_arch = "x86"), allow(dead_code))]
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
pub fn with_overlays<R>(mk: usize, overlays: &[Overlay], f: impl FnOnce() -> R) -> R {
    for o in overlays {
        fits_buffer(mk.wrapping_add(o.start).wrapping_sub(START), o.len);
    }
    let at = |o: &Overlay| crate::pointers::host_at(mk.wrapping_add(o.start).wrapping_sub(START));
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

/// Stops the test (naming it) unless `len` bytes from the game address `at` lie in one test
/// buffer: a block copied into a buffer too small for it overwrote the heap past it.
#[track_caller]
pub fn fits_buffer(at: usize, len: usize) {
    let room =
        crate::pointers::region_of(at as u32).map(|(_, end)| (end as usize).wrapping_sub(at));
    assert!(
        room.is_some_and(|room| room >= len),
        "a {len:#x}-byte block at {at:#x} does not fit its test buffer ({room:#x?} bytes there)"
    );
}
