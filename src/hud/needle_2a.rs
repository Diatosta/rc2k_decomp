//! `hud::needle_2a` (0x4B6DEC): the first needle of layout 2: its set-up (`hud::dials`), then `hud::draw_needle`.
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
    RawHook::new("hud::needle_2a", rva(0x4B6DEC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub static SETUP: Setup = Setup {
    copies: &[
        (From::Field(0x10), rva(0x75C2E0), false),
        (From::Var(rva(0x75C2F0)), rva(0x75C2A0), false),
        (From::Var(rva(0x75C2F4)), rva(0x75C2A4), false),
        (From::Var(rva(0x75C2F8)), rva(0x75C2A4), true),
        (From::Var(rva(0x75C2FC)), rva(0x75C2B8), false),
    ],
    len: 0x48,
    entry: [0xFF, 0x75, 0x10, 0x8F, 0x05, 0xE0, 0xC2, 0x75, 0x00],
    call: 0x42,
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

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { set_up(r, &SETUP, &mut crate::hud::draw_needle::game_part_needle) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

fn self_test() -> (usize, usize) {
    test(&HOOK, 0x4B6DEC, &SETUP)
}
