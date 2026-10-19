//! `menu::previous_stage_number` (0x48201B): the stage choice's stage back (below 2, signed, to 6), then the map drawn again.
//!
//! In:  nothing
//! Out: every register preserved (as the map leaves them)
//!
//! See `menu/stage_choice.rs`. In three item lists (the stage choice screens). Self-test: a
//! copy whose map call is counted, the game's variable with values of the test's.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::menu::stage_choice::{
    Setting, check_setting, draw_stage_map_in_game, previous_stage_number,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::previous_stage_number",
    rva(0x48201B),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

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
    if let Err(e) = unsafe { previous_stage_number(&mut || draw_stage_map_in_game()) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let s = Setting {
        address: 0x48201B,
        len: 0x21,
        entry: &[0x83, 0x3D, 0xCC, 0x58, 0x75, 0x00, 0x02],
        call: 0x1B,
        variable: 0x7558CC,
    };
    if let Err(why) = check_setting(
        &mut t,
        &s,
        &[0, 1, 2, 5, 6, 7, 0x8000_0000, 0xFFFF_FFFF],
        previous_stage_number,
    ) {
        return t.skip(why);
    }
    t.finish()
}
