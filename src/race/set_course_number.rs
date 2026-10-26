//! `race::set_course_number` (0x43944F): the course's number put into the stage's paths.
//!
//! In:  al = the number
//! Out: every register preserved
//!
//! See `race/level_init.rs`: the tens and the units digit into the nine path strings that
//! hold it. Two callers (`sub_44426F` and one more). Self-test: the game's strings, numbers
//! 0 .. 255 by steps, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::level_init::{NUMBER_PLACES, set_course_number};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::set_course_number", rva(0x43944F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    NUMBER_PLACES
        .iter()
        .map(|&p| (game_addr(rva(p)) as u32, 2))
        .collect()
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { set_course_number(r.eax) };
    Ok(Left::nothing())
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
    for n in [0u32, 7, 10, 42, 99, 100, 255, 0x1234_5603] {
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
