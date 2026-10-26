//! `scene::set_fine_turn_x` (0x4270BA): the turning matrix about the x axis for an angle of
//! 32 bits a turn.
//!
//! In:  edx = the angle, a whole turn being 2^32
//! Out: eax = the sine, ebx = the cosine, ecx = the sine negated, edi = the matrix;
//!      everything else preserved
//!
//! The angle as a whole number (signed) over `flt_5C5D98` (65536) twice and over
//! `flt_5C5DD4` (1 / 2 pi), then `fsincos`: the instruction itself, as its result is
//! the processor's. The pair is kept in `dword_624148 and dword_62414C`, the angle in
//! `dword_62400C`, and goes to `g_turnXMatrix`: +0x14 the cosine, +0x18 the sine, +0x24 the sine negated, +0x28 the cosine.
//! The matrix's other floats stay as they are.
//! Self-test: 48 angles on the game's matrix, put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `view_matrix::set_fine_turn_plain`,
//! through `rc2k_core::scene::matrices::set_fine_turn` (plain floats; the x87 version stays
//! for the Rust callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ECX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::scene::matrix::{float_places, test_floats};
use crate::scene::view_matrix::{self, Axis};
use crate::screens::harness::Left;
use crate::service::floats::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::set_fine_turn_x", rva(0x4270BA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- hook ----

const AXIS: Axis = Axis::X;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { view_matrix::set_fine_turn_plain(AXIS, r.edx) })
}

extern "C" fn dispatch(r: &mut Regs) {
    let places = float_places(&AXIS.regions());
    let floats = Floats {
        memory: &places,
        regs: &[(EAX, FLOAT), (EBX, FLOAT), (ECX, FLOAT)],
    };
    dispatch_state_floats(&HOOK, r, call_original, &AXIS.regions(), &floats, &rust);
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
    let regions = AXIS.regions();
    let saved = capture(&regions);
    let mut seed = 0x2468_ACE1u32;
    let tolerant_places = float_places(&regions);
    let tolerant = Floats {
        memory: &tolerant_places,
        regs: &[(EAX, FLOAT), (EBX, FLOAT), (ECX, FLOAT)],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for n in 0..48u32 {
            let angle = match n {
                0 => 0,
                1 => 1,
                2 => 0xFFFF_FFFF,
                3 => 0x4000_0000,
                4 => 0x8000_0000,
                5 => 0x7FFF_FFFF,
                6 => 0xC000_0000,
                7 => 0x2000_0000,
                n => n.wrapping_mul(0x0612_3457),
            };
            let floats = test_floats(&mut seed);
            check_state_floats(
                &mut t,
                "scene::set_fine_turn_x",
                &format!("control word {word:#x}, angle {angle:#x}"),
                call_original,
                Regs {
                    edx: angle,
                    ..Regs::SENTINEL
                },
                &|| unsafe {
                    restore(&regions, &vec![0xAA; saved.len()]);
                    (regions[0].0 as *mut [u32; 16]).write_unaligned(floats);
                },
                &regions,
                held,
                &rust,
            );
        }
    });
    restore(&regions, &saved);
    t.finish()
}
