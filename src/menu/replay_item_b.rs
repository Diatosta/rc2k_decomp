//! `menu::replay_item_b` (0x47052F): a replay item of the review menus (see `menu/replay_items.rs`).
//!
//! In:  nothing
//! Out: the registers as its calls leave them; no flags read
//!
//! An item action in six menu tables. Shadow mode runs the original alone (the viewer
//! plays the replay). Self-test: `replay_items::test_item`.

use crate::framework::{RawHook, Regs, hook_asm, rva, shadow};
use crate::menu::replay_items::{game_part, replay_item_b, test_item};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::replay_item_b", rva(0x47052F), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { replay_item_b(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    test_item(&HOOK, 0x47052F, 0xF, &[0x9], &|r| unsafe {
        replay_item_b(r, &mut crate::race::mud::test_part)
    })
}
