//! `hud::needle_1a` (0x4B6E67): the first needle of layout 1: its set-up (`hud::dials`), then `hud::draw_needle`.
//!
//! In:  ebp the HUD's record
//! Out: as `draw_needle` leaves them
//!
//! The shape through `part` (1), in the game its Rust version. Shadow mode runs the
//! original alone (it draws). Self-test: `hud::dials::test`.

use crate::framework::{RawHook, Regs, hook_asm, rva, shadow};
use crate::hud::dials::{From, Setup, set_up, test};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::needle_1a", rva(0x4B6E67), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub static SETUP: Setup = Setup {
    copies: &[
        (From::Field(0x10), rva(0x75C2E0), false),
        (From::Var(rva(0x75C30C)), rva(0x75C2A0), false),
        (From::Var(rva(0x75C310)), rva(0x75C2A4), false),
        (From::Var(rva(0x75C314)), rva(0x75C2B8), false),
    ],
    len: 0x33,
    entry: [0xFF, 0x75, 0x10, 0x8F, 0x05, 0xE0, 0xC2, 0x75, 0x00],
    call: 0x2D,
    callee: &[(0x4B6ECD, 1)],
};

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { set_up(r, &SETUP, &mut crate::hud::draw_needle::game_part_needle) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    test(&HOOK, 0x4B6E67, &SETUP)
}
