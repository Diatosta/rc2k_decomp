//! `hud::draw_markers_2` (0x4B55EE): `hud::draw_markers_1`'s twin at rows 0x11F and
//! 0x12C (the lower half's).
//!
//! In:  ebp the HUD's record
//! Out: as `draw_markers_1`'s
//!
//! Caller: layout 2's drawer (0x4B403B). The sprites through `part` as
//! `draw_markers_1`'s. Shadow mode runs the original alone (it draws). Self-test:
//! `draw_markers_1::test_markers`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva, shadow};
use crate::hud::draw_markers_1::{Part, game_part, markers_with, test_markers};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::draw_markers_2", rva(0x4B55EE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_markers_2(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe { markers_with(entry, [0x11F, 0x12C], part) }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_markers_2(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    test_markers(&HOOK, 0x4B55EE, &|r, p| unsafe { draw_markers_2(r, p) })
}
