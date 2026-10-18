//! `hud::disc_1a` (0x4B6B5B): the first dial of layout 1 (`sub_4B2743`): its set-up (`hud::dials`), then `hud::draw_disc`.
//!
//! In:  ebp the HUD's record
//! Out: as `draw_disc` leaves them
//!
//! The shape through `part` (1), in the game its Rust version. Shadow mode runs the
//! original alone (it draws). Self-test: `hud::dials::test`.

use crate::framework::{RawHook, Regs, hook_asm, rva, shadow};
use crate::hud::dials::{From, Setup, set_up, test};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::disc_1a", rva(0x4B6B5B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub static SETUP: Setup = Setup {
    copies: &[
        (From::Field(0x34), rva(0x75C054), false),
        (From::Field(0x38), rva(0x75C058), false),
        (From::Field(0x3C), rva(0x75C05C), false),
        (From::Var(rva(0x75C158)), rva(0x75C050), false),
        (From::Var(rva(0x75C15C)), rva(0x75C04C), false),
        (From::Var(rva(0x75C160)), rva(0x75C044), false),
        (From::Var(rva(0x75C164)), rva(0x75C048), false),
    ],
    len: 0x51,
    entry: [0xFF, 0x75, 0x34, 0x8F, 0x05, 0x54, 0xC0, 0x75, 0x00],
    call: 0x4B,
    callee: &[(0x4B6C4E, 1)],
};

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { set_up(r, &SETUP, &mut crate::hud::add_triangle::game_part_disc) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    test(&HOOK, 0x4B6B5B, &SETUP)
}
