//! Test support for the marks' x87 blocks that work on typed statics: the differential tests
//! (`asm_test`) run the recorded x87 assembly over a test buffer standing for the marks' block at
//! the test buffer `mk` and the plain Rust version over the typed statics; around the Rust version
//! the statics hold the buffer's bytes, and the bytes it leaves go back to the buffer
//! (`test_support::test_lock` is held by `differential`).

use crate::effects::asm_fields::{image, put_image};
use crate::effects::asm_layout::{Laid, Run};
use crate::effects::asm_test::Operand;
use crate::effects::asm_test::START;

/// Sets a static's fields from all its bytes.
type Setter = Box<dyn Fn(&[u8])>;

/// A typed static with a recorded layout (`Laid`) and where its first byte is in the marks'
/// block. Its fields are read and written as a byte image (`asm_fields`), whatever the static's
/// own layout.
pub struct Overlay {
    start: usize,
    /// The runs of the static's fields that were in the recorded layout.
    runs: &'static [Run],
    /// Where each run begins in the static's image.
    run_starts: Vec<usize>,
    /// The static's bytes, all of them.
    get: Box<dyn Fn() -> Vec<u8>>,
    /// The static's fields set from all of its bytes.
    set: Setter,
}

/// The overlay of `st`, a typed static with a recorded layout (`Laid`): its fields hold the
/// bytes the buffer has at their places.
pub(super) fn overlay_laid<T: Laid>(st: &'static T) -> Overlay {
    Overlay {
        start: T::START,
        runs: T::RUNS,
        run_starts: T::RUNS
            .iter()
            .map(|r| {
                crate::effects::asm_fields::field_start(st, r.field)
                    .unwrap_or_else(|| panic!("{}", r.field))
            })
            .collect(),
        get: Box::new(move || image(st)),
        set: Box::new(move |bytes| put_image(st, bytes)),
    }
}

/// Runs `f` with each overlay's static holding the bytes the buffer `mk` has there; what the
/// statics hold afterwards goes into the buffer, and the statics are put back as they were.
pub(super) fn with_overlays<R>(mk: Operand, overlays: &[Overlay], f: impl FnOnce() -> R) -> R {
    let place = |o: &Overlay, r: &Run| (o.start + r.at).wrapping_sub(START);
    // Every piece must fit the buffer before any is touched.
    for o in overlays {
        for r in o.runs {
            mk.at(place(o, r), r.len);
        }
    }
    let saved: Vec<Vec<u8>> = overlays.iter().map(|o| (o.get)()).collect();
    for (o, before) in overlays.iter().zip(&saved) {
        let mut bytes = before.clone();
        for (r, &at) in o.runs.iter().zip(&o.run_starts) {
            bytes[at..at + r.len].copy_from_slice(&mk.read(place(o, r), r.len));
        }
        (o.set)(&bytes);
    }
    let out = f();
    for (o, before) in overlays.iter().zip(&saved) {
        let after = (o.get)();
        for (r, &at) in o.runs.iter().zip(&o.run_starts) {
            mk.put(place(o, r), &after[at..at + r.len]);
        }
        (o.set)(before);
    }
    out
}
