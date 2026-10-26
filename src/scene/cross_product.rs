//! `scene::cross_product` (0x420F8D): the cross product of two vectors of three floats.
//!
//! In:  ebx, ecx = the two vectors, edx = where the result goes
//! Out: every register and the flags preserved
//!
//! out[0] = c[2] b[1] - b[2] c[1], out[1] = c[0] b[2] - b[0] c[2],
//! out[2] = c[1] b[0] - b[1] c[0] (b = ebx's, c = ecx's), each stored before the next
//! is worked out. (The x87 version: the original's instructions.)
//! Ten callers.
//! Refuses a null pointer.
//! Self-test: vectors by lot, the result apart and in the place of either.
//!
//! Converted 2026-10-21: the hook's Rust version is `cross_product_plain`, through
//! `rc2k_core::scene::matrices::cross` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::scene::matrix::{MATRIX_LEN, float_places, test_floats};
use crate::scene::view_matrix;
use crate::screens::harness::Left;
use crate::service::floats::*;
use crate::service::harness::Region;
use std::cell::Cell;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::cross_product", rva(0x420F8D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.ebx == 0 || r.ecx == 0 || r.edx == 0 {
        return Err(BadInput("a null vector").into());
    }
    unsafe { view_matrix::cross_product_plain(r.ebx as usize, r.ecx as usize, r.edx as usize) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions: &[Region] = if r.ebx == 0 || r.ecx == 0 || r.edx == 0 {
        &[]
    } else {
        &[(r.edx, 12)]
    };
    let places = float_places(regions);
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, regions, &floats, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let work: [Cell<[u32; 16]>; 3] = Default::default();
    let address = |n: usize| work[n].as_ptr() as u32;
    let mut seed = 0x1C2D_3E4Fu32;
    let tolerant_places = float_places(&[0, 1, 2].map(|n| (address(n), MATRIX_LEN)));
    let tolerant = Floats {
        memory: &tolerant_places,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for case in 0..60 {
            let floats = [
                test_floats(&mut seed),
                test_floats(&mut seed),
                test_floats(&mut seed),
            ];
            let out = [2usize, 0, 1][case % 3];
            let regions: [Region; 3] = [0, 1, 2].map(|n| (address(n), MATRIX_LEN));
            check_state_floats(
                &mut t,
                "scene::cross_product",
                &format!("control word {word:#x}, case {case}, the result in vector {out}"),
                call_original,
                Regs {
                    ebx: address(0),
                    ecx: address(1),
                    edx: address(out),
                    ..Regs::SENTINEL
                },
                &|| {
                    for (cell, floats) in work.iter().zip(floats) {
                        cell.set(floats);
                    }
                },
                &regions,
                held,
                &rust,
            );
        }
    });
    t.finish()
}
