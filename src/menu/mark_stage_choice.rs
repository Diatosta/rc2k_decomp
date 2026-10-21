//! `menu::mark_stage_choice` (0x486C21): `dword_755700` 1 (the stage screens' map then takes
//! the choice's stage; 2 a championship's, 3 a record's).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! One caller, `menu::select_stage_loop` at its set-up. Self-test: the game's variable,
//! put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::menu::stage_choice::MODE_RVA;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::mark_stage_choice", rva(0x486C21), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn mark_stage_choice() {
    // Converted: `rc2k_core::menu::event::MapStage`.
    unsafe { game_write(MODE_RVA, rc2k_core::menu::event::MapStage::Choice as u32) };
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    mark_stage_choice();
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[at(MODE_RVA)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [at(MODE_RVA)];
    let saved = capture(&regions);
    check_state(
        &mut t,
        "from 3",
        call_original,
        Regs::SENTINEL,
        &|| unsafe { game_write(MODE_RVA, 3u32) },
        &regions,
        &rust,
    );
    restore(&regions, &saved);
    t.finish()
}
