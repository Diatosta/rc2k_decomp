//! `race::clear_prefs_qword` (0x465381): `qword_609344` 0 when `dword_71C800` is set.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/stage_prefs.rs`. Self-test: the original and the Rust version on the game's
//! variables with values of the test's, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::stage_prefs::*;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::clear_prefs_qword", rva(0x465381), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    vec![(game_addr(rva(0x609344)) as u32, 8), at(rva(0x71C800))]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { clear_prefs_qword() };
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
    let _ = (at as fn(usize) -> Region, game_addr as fn(usize) -> usize);
    for flag in [0u32, 1, 0x8000_0000] {
        check_state(
            &mut t,
            &format!("flag {flag:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(rva(0x609344), [0x1234_5678u32, 0x9ABC_DEF0]);
                game_write(rva(0x71C800), flag);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
