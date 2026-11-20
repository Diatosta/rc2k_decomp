//! Test support for the marks' x87 blocks that work on typed statics: the differential tests
//! (`asm_test`) run the recorded x87 assembly over a test buffer standing for the marks' block at
//! the test buffer `mk` and the plain Rust version over the typed statics; around the Rust version
//! the statics hold the buffer's bytes, and the bytes it leaves go back to the buffer
//! (`test_support::test_lock` is held by `differential`).

use crate::effects::asm_layout::Laid;
use crate::effects::asm_test::Operand;
use crate::effects::asm_test::START;
use crate::test_support::paths::{field_bytes, field_range, set_field_range};

type Reader = Box<dyn Fn(&str, usize) -> Vec<u8>>;
type Writer = Box<dyn Fn(&str, &[u8]) -> bool>;

/// A typed static with a recorded layout (`Laid`) and where its first byte is in the marks'
/// block. Its cells are read and written by field path, whatever the static's own layout.
pub struct Overlay {
    start: usize,
    /// The runs of the static's fields that were in the recorded layout.
    runs: &'static [crate::effects::asm_layout::Run],
    /// The bytes of the static's cells from a field on, a given length.
    get: Reader,
    /// The cells from a field on set from bytes.
    set: Writer,
}

/// The overlay of `st`, a typed static with a recorded layout (`Laid`): its fields hold the
/// bytes the buffer has at their places.
pub(super) fn overlay_laid<T: Laid>(st: &'static T) -> Overlay {
    Overlay {
        start: T::START,
        runs: T::RUNS,
        get: Box::new(move |field, len| {
            if field.is_empty() && len == usize::MAX {
                field_bytes(st, "")
            } else {
                field_range(st, field, len)
            }
        }),
        set: Box::new(move |field, bytes| set_field_range(st, field, bytes)),
    }
}

/// Runs `f` with each overlay's static holding the bytes the buffer `mk` has there; what the
/// statics hold afterwards goes into the buffer, and the statics are put back as they were.
pub(super) fn with_overlays<R>(mk: Operand, overlays: &[Overlay], f: impl FnOnce() -> R) -> R {
    let place =
        |o: &Overlay, r: &crate::effects::asm_layout::Run| (o.start + r.at).wrapping_sub(START);
    // Every piece must fit the buffer before any is touched.
    for o in overlays {
        for r in o.runs {
            mk.at(place(o, r), r.len);
        }
    }
    let saved: Vec<Vec<u8>> = overlays.iter().map(|o| (o.get)("", usize::MAX)).collect();
    for o in overlays {
        for r in o.runs {
            assert!(
                (o.set)(r.field, &mk.read(place(o, r), r.len)),
                "{}",
                r.field
            );
        }
    }
    let out = f();
    for (o, bytes) in overlays.iter().zip(&saved) {
        for r in o.runs {
            mk.put(place(o, r), &(o.get)(r.field, r.len));
        }
        assert!((o.set)("", bytes));
    }
    out
}
