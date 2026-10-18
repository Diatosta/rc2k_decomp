//! `menu::fade_in` (0x4A1CFA): the inspection's fade in: the see-through from 0 up to `dword_75AA94` with the time gone (`menu::fades::fade`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_48E093`, `sub_4A15E3`, `sub_4A8556`. The clock is `system::tick_count`'s Rust version; the frames
//! (`menu::fade_frame` through `screens::run_frame`) through the game (platform).
//! Ungated (it runs for the fade's length). A private copy (a call at +1; its three jumps
//! to the fatal stub relocated). Fatal error 0x80701010 as the original's checks;
//! refuses a length of 1 .. 99 ms (the original divides by 0), before anything. Shadow
//! mode runs the original alone (it shows frames). Self-test: `fades::test_fade`.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::menu::fades::{fade, game_part, test_fade};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("menu::fade_in", rva(0x4A1CFA), thunk, &ORIGINAL)
    .private_copy(0xAB)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const CALLS: [usize; 7] = [0x1, 0x1B, 0x20, 0x41, 0x5A, 0x79, 0x93];

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { fade(r, false, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_fade(
        t,
        0x4A1CFA,
        0xAB,
        &[0x60, 0xE8, 0x24, 0xF3, 0xF6, 0xFF],
        &CALLS,
        false,
    )
}
