//! `menu::show_in_game_options` (0x46FBE3): the in-game options' items 3 .. 6 shown.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `menu/in_game_options_items.rs`: strings 0x171 .. 0x174, choosable (+0x48
//! 0xFFFFFC1B), +0x1C 0xFFFFFFA7. Callers: the event loops at their end (`sub_482382`) and
//! seven more. Self-test: the game's menu filled with 0xAA, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::in_game_options_items::{items_region, show_in_game_options};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::show_in_game_options",
    rva(0x46FBE3),
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

fn rust(_: &Regs) -> Result<Left, Refused> {
    show_in_game_options();
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[items_region()], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [items_region()];
    let saved = capture(&regions);
    check_state(
        &mut t,
        "items 3 .. 6",
        call_original,
        Regs::SENTINEL,
        &|| unsafe { std::ptr::write_bytes(regions[0].0 as *mut u8, 0xAA, regions[0].1) },
        &regions,
        &rust,
    );
    restore(&regions, &saved);
    t.finish()
}
