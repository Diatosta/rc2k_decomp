//! `race::set_race_screen_mode` (0x404830): `gfx::set_screen_mode` (0x404B63, the
//! platform's: called in the game) with eax `g_raceScreenMode`; `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/race_shell.rs`. Callers: `sub_4170E0`, `sub_41CDC2`, `sub_4843EC` (each calls
//! on). Shadow mode runs the original alone (it sets the screen's mode). Self-test: a copy
//! whose call goes to a stand-in that notes it, against the Rust version on the same.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::race::race_shell::*;
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::set_race_screen_mode",
    rva(0x404830),
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
    unsafe { set_race_screen_mode(r, &mut game_screen_mode) };
}

fn self_test() -> (usize, usize) {
    use crate::race::level_init::{note, take_called};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(note);
    let copy = unsafe {
        crate::redirect::RedirectedCopy::new(
            game_addr(rva(0x404830)),
            0xD,
            &[0x60, 0xA1, 0x48, 0x49, 0x4F, 0x00],
            &[0x6],
            &|c| (c == SET_SCREEN_MODE_RVA).then(|| crate::install::stand_ins::stand_in(1)),
        )
    };
    let copy = match copy {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let kept = [at(SCREEN_MODE_RVA)];
    let saved = capture(&kept);
    for mode in [0u32, 3, 0x1234] {
        unsafe { game_write(SCREEN_MODE_RVA, mode) };
        take_called();
        let before = Regs::SENTINEL;
        let mut ro = before;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let theirs = (take_called(), ro.preserved_except(&before, 0));
        unsafe { set_race_screen_mode(&before, &mut |r| note(1, r)) };
        let mine = (take_called(), true);
        t.check(theirs == mine, || {
            format!("mode {mode:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&kept, &saved);
    t.finish()
}
