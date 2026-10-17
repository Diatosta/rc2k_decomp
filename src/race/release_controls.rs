//! `race::release_controls` (0x409425): `dword_4F7DE4` 2 (the controls' state;
//! `race::create_controls` sets 3).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/race_shell.rs`. One caller, `sub_444AC7` (the stage's end; `pushf` follows).
//! Shadow mode: the Rust version, the variable put back, the original's, compared.
//! Self-test: the game's variable from 0 and 3, put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::race::race_shell::*;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::release_controls", rva(0x409425), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { release_controls() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[at(CONTROLS_RVA)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [at(CONTROLS_RVA)];
    let saved = capture(&regions);
    for from in [0u32, 3] {
        check_state(
            &mut t,
            &format!("from {from}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(CONTROLS_RVA, from) },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
