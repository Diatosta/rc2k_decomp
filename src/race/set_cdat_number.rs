//! `race::set_cdat_number` (0x4456DD): al's two digits into the course data's two paths.
//!
//! In:  al = the number
//! Out: eax the tens' digit, edx the units', ebx 10; the rest preserved
//!
//! See `race/course_data.rs`. Callers `sub_4455FC` (with 0) and `race::find_present_courses`. Self-test:
//! the original and the Rust version on the game's paths, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::course_data::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::set_cdat_number", rva(0x4456DD), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    CDAT_DIGITS
        .iter()
        .map(|&a| (game_addr(a) as u32, 2))
        .collect()
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(crate::race::stage_prefs::left_of(&unsafe {
        set_cdat_number(r)
    }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let saved = capture(&regions);
    for n in [0u32, 1, 9, 10, 12, 0x63, 0x64, 0xFF, 0xDEAD_BE07] {
        check_state(
            &mut t,
            &format!("eax {n:#x}"),
            call_original,
            Regs {
                eax: n,
                ..Regs::SENTINEL
            },
            &|| restore(&regions, &saved),
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
