//! `hud::draw_signs_1b` (0x4B5451): the second player's signs in layout 1: as `hud::draw_signs` from the record's +0xB8 ..,
//! the places from +0xC8 plus 0xC8, at row 0x70 (0x20 higher with `dword_6D1502` 0).
//!
//! In:  ebp the HUD's record
//! Out: as `draw_signs`'
//!
//! Caller: layout 1's drawer. The sprites through `part` (1), in the game `add_sprite`'s
//! Rust version. Shadow mode runs the original alone (it draws). Self-test:
//! `draw_signs::test_signs`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva, shadow};
use crate::hud::draw_signs::{Part, game_part, signs_with, test_signs};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::draw_signs_1b", rva(0x4B5451), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_signs_1b(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe { signs_with(entry, 0xB8, 0xC8, 0xC8, 0x70, 0x20, part) }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_signs_1b(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let entry: &[u8] = &[0x8B, 0x9D, 0xC8, 0x00, 0x00, 0x00];
    test_signs(
        &HOOK,
        0x4B5451,
        0x15A,
        entry,
        &[0x57, 0xAB, 0xFF, 0x153],
        &|r, p| unsafe { draw_signs_1b(r, p) },
    )
}
