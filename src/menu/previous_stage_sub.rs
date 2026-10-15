//! `menu::previous_stage_sub` (0x481FD9): the stage choice's rally back (below 1 to 5), then the map drawn again.
//!
//! In:  nothing
//! Out: every register preserved (as the map leaves them)
//!
//! See `menu/stage_choice.rs`. In three item lists (the stage choice screens). Self-test: a
//! copy whose map call is counted, the game's variable with values of the test's.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::menu::stage_choice::{
    Setting, check_setting, draw_stage_map_in_game, previous_stage_sub,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::previous_stage_sub", rva(0x481FD9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { previous_stage_sub(&mut || draw_stage_map_in_game()) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let s = Setting {
        address: 0x481FD9,
        len: 0x21,
        entry: &[0x83, 0x3D, 0xD0, 0x58, 0x75, 0x00, 0x01],
        call: 0x1B,
        variable: 0x7558D0,
    };
    if let Err(why) = check_setting(
        &mut t,
        &s,
        &[0, 1, 3, 5, 6, 0xFFFF_FFFF],
        previous_stage_sub,
    ) {
        return t.skip(why);
    }
    t.finish()
}
