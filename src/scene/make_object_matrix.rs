//! `scene::make_object_matrix` (0x427812): an object's matrix from its angles, place
//! and size.
//!
//! In:  nothing
//! Out: eax = the place's x, ebx = the size, ecx = 0, edx = 0, esi = `flt_5EA10C`,
//!      edi = `flt_5EA04C`, ebp = the matrix made, `flt_5EA30C`
//!
//! `scene::set_light_direction`; the turning about y for `dword_635040` negated and
//! about x for `dword_63503C` negated; the place `dword_63501C` .. as floats
//! (`scene::set_place`); the y turning times the x turning to `flt_5EA10C`; all
//! sixteen of its floats times the size `dword_635038` over 16; and that with the
//! place added (`scene::copy_matrix_with_place`) to `flt_5EA30C`.
//! One caller, `sub_4907E7`.
//! Manual patch with a private copy: the function begins with a call.
//! Self-test: 60 settings by lot on the game's matrices, put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `make_object_matrix_plain`, through
//! `rc2k_core::scene::matrices::make_object_matrix` (plain floats; the x87 version stays
//! for the Rust callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::scene::matrix::{self, composed_inputs, composed_regions, set_composed, sum_places};
use crate::screens::harness::Left;
use crate::service::floats::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::make_object_matrix", rva(0x427812), thunk, &ORIGINAL)
        .private_copy(0xA5)
        .relocate_calls(&[0x00, 0x0D, 0x1A, 0x30, 0x47, 0x9F])
        .manual_patch()
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { matrix::make_object_matrix_plain() })
}

extern "C" fn dispatch(r: &mut Regs) {
    let places = sum_places(&composed_regions());
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &composed_regions(), &floats, &rust);
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
    let (regions, inputs) = (composed_regions(), composed_inputs());
    let saved = (capture(&regions), capture(&inputs));
    let mut lot = 0x00C0_FFEEu32;
    let mut next = |below: u32| {
        lot = lot.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (lot >> 8) % below
    };
    let tolerant_places = sum_places(&regions);
    let tolerant = Floats {
        memory: &tolerant_places,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for case in 0..60u32 {
            let angles = [next(0x100), next(0x100), next(0x2_0000), next(0x2_0000)];
            let place = [
                next(0x40_0000).wrapping_sub(0x20_0000),
                next(0x40_0000).wrapping_sub(0x20_0000),
                next(0x400_0000),
            ];
            let size = [16u32, 1, 0, 0x100, 0xFFFF_FFF0, 37][next(6) as usize];
            let seed = next(0x00FF_FFFF);
            check_state_floats(
                &mut t,
                "scene::make_object_matrix",
                &format!(
                    "control word {word:#x}, case {case}: angles {angles:x?}, place {place:x?}, size {size:#x}"
                ),
                call_original,
                Regs::SENTINEL,
                &|| unsafe { set_composed(&mut { seed }, angles, place, size) },
                &regions,
                held,
                &rust,
            );
        }
    });
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
