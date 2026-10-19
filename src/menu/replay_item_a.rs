//! `menu::replay_item_a` (0x47053E): a replay item of the review menus (see `menu/replay_items.rs`).
//!
//! In:  nothing
//! Out: the registers as its calls leave them; no flags read
//!
//! An item action in six menu tables. Ungated (the viewer runs for long and gates its own frames).
//! Shadow mode runs the original alone (the viewer plays the replay). Self-test: `replay_items::test_item`.

use crate::framework::{RawHook, Regs, hook_asm, rva, shadow};
use crate::menu::replay_items::{game_part, replay_item_a, test_item};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::replay_item_a", rva(0x47053E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { replay_item_a(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    test_item(&HOOK, 0x47053E, 0x2D, &[0x12, 0x1D, 0x27], &|r| unsafe {
        replay_item_a(r, &mut crate::race::mud::test_part)
    })
}
