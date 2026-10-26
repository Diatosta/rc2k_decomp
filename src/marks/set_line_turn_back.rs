//! `marks::set_line_turn_back` (0x4BF466): the 2 x 2 turn of the line at esi (dx, dy) into
//! `flt_82E396` .. `flt_82E3A6`.
//!
//! In:  esi the line (+0, +4 to +0xC, +0x10)
//! Out: every register kept
//!
//! See `marks/turns.rs`. Callers: `sub_4BEFD2` (twice). Self-test: the original and the
//! Rust version on lines by lot, put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `turns::set_line_turn_plain`, through
//! `rc2k_core::marks::turns::line_turn` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::turns::*;
use crate::race::stage_prefs::left_of;
use crate::scene::matrix::float_places;
use crate::screens::harness::Left;
use crate::service::floats::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::set_line_turn_back", rva(0x4BF466), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { set_line_turn_plain(r.esi, true) };
    Ok(left_of(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    let places = float_places(&turn_regions());
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &turn_regions(), &floats, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = turn_regions();
    let saved = capture(&regions);
    let mut line = [0u32; 5];
    let p = line.as_mut_ptr() as u32;
    let tolerant_places = float_places(&regions);
    let tolerant = Floats {
        memory: &tolerant_places,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for seed in 1..=10u32 {
            check_state_floats(
                &mut t,
                "marks::set_line_turn_back",
                &format!("control word {word:#x}, seed {seed}"),
                call_original,
                Regs {
                    esi: p,
                    ..Regs::SENTINEL
                },
                &|| {
                    let mut v = [0u32; 5];
                    crate::scene::matrix3::test_floats(seed, &mut v);
                    unsafe { (p as *mut [u32; 5]).write(v) };
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
