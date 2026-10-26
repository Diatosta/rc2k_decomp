//! `scene::unit_vector` (0x420EF8): three floats made one long.
//!
//! In:  ecx = the three floats, edx = where the result goes (may be the same place)
//! Out: every register preserved (the flags are the last compare's; no caller reads them)
//!
//! The length (squares added, the last two first, `fsqrt`) is left in `dword_62400C`.
//! A length that compares equal to `flt_5C5D90` (0; also a length that is no number)
//! gives three zeros; else each float over the length, a minus zero made zero.
//! Four callers.
//! Refuses a null pointer.
//! Self-test: vectors by lot, apart and in place.
//!
//! Converted 2026-10-21: the hook's Rust version is `unit_vector_plain`, through
//! `rc2k_core::scene::matrices::unit_vector` (plain floats; the x87 version stays for the
//! Rust callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::scene::matrix::{float_places, test_floats};
use crate::scene::view_matrix;
use crate::screens::harness::Left;
use crate::service::floats::*;
use crate::service::harness::{Region, at, capture, restore};
use std::cell::Cell;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::unit_vector", rva(0x420EF8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.ecx == 0 || r.edx == 0 {
        return Err(BadInput("a null vector").into());
    }
    unsafe { view_matrix::unit_vector_plain(r.ecx as usize, r.edx as usize) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions: &[Region] = if r.ecx == 0 || r.edx == 0 {
        &[]
    } else {
        &[(r.edx, 12), at(CHECKED_RVA)]
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
    let work: [Cell<[u32; 16]>; 2] = Default::default();
    let address = |n: usize| work[n].as_ptr() as u32;
    let checked = [at(CHECKED_RVA)];
    let saved = capture(&checked);
    let mut seed = 0x0F1E_2D3Cu32;
    let tolerant_places = float_places(&[(address(0), 0x40), (address(1), 0x40), checked[0]]);
    let tolerant = Floats {
        memory: &tolerant_places,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for case in 0..80 {
            let mut floats = [test_floats(&mut seed), test_floats(&mut seed)];
            if case % 9 == 0 {
                // No length at all, with zeros of both signs.
                floats[0][..3].copy_from_slice(&[0, 0x8000_0000, 0]);
            }
            let out = case % 2;
            let regions: [Region; 3] = [(address(0), 0x40), (address(1), 0x40), checked[0]];
            check_state_floats(
                &mut t,
                "scene::unit_vector",
                &format!(
                    "control word {word:#x}, case {case}: {:x?}, the result in {out}",
                    &floats[0][..3]
                ),
                call_original,
                Regs {
                    ecx: address(0),
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
    restore(&checked, &saved);
    t.finish()
}
