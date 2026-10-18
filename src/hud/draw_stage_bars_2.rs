//! `hud::draw_stage_bars_2` (0x4B63C2): `hud::draw_stage_bars_1` 0x11E lower.
//!
//! In:  ebp the HUD's record
//! Out: as `draw_stage_bars_1`'s
//!
//! Caller: layout 2's drawer (0x4B3C0B). The quads through `part` as
//! `draw_stage_bars_1`'s. Shadow mode runs the original alone (it draws). Self-test:
//! `draw_stage_bars_1::test_bars`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva, shadow};
use crate::hud::draw_stage_bar::{Part, game_part};
use crate::hud::draw_stage_bars_1::{bars_with, test_bars};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::draw_stage_bars_2", rva(0x4B63C2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_stage_bars_2(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe { bars_with(entry, 0x11E, part) }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_stage_bars_2(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    test_bars(&HOOK, 0x4B63C2, &|r, p| unsafe { draw_stage_bars_2(r, p) })
}
