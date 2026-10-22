//! `screens::draw_fade_overlay` (0x49FC7F): a see-through quad over the whole 800 x 600
//! screen, colour 0x11166A, its see-through `dword_75AB50` (the fades of the inspection's
//! screens), by `scene::draw_tinted_quad`; the quad's inputs `dword_623E10` ..
//! `dword_623E24` (`screens::service_boxes::quad`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A1E60` (the fades' frame). The quad through `part` (1), in the game its
//! Rust version. Shadow mode runs the original alone (it draws). Self-test:
//! `service_boxes::test_boxes`, the see-through by lot.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva, shadow};
use crate::screens::service_boxes::{Inputs, Part, quad, quad_part, rd, test_boxes};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_fade_overlay",
    rva(0x49FC7F),
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

const SEE_THROUGH: usize = 0x75AB50; // dword_75AB50

pub static INPUTS: Inputs = Inputs {
    tested: &[],
    read: &[SEE_THROUGH],
};

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_fade_overlay(entry: &Regs, mut part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::screens::panels::fade_overlay`.
    let mut g = *entry;
    let a = rc2k_core::screens::panels::fade_overlay(rd(SEE_THROUGH));
    quad(&mut g, a.x, a.y, a.w, a.h, a.colour, a.see, &mut part)?;
    Ok(*entry)
}

pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    quad_part(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if crate::screens::blanked("screens::draw_fade_overlay") {
        return;
    }
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_fade_overlay(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    if crate::screens::blanked("screens::draw_fade_overlay") {
        return Ok(());
    }
    let left = unsafe { draw_fade_overlay(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

fn self_test() -> (usize, usize) {
    test_boxes(
        &HOOK,
        0x49FC7F,
        0x50,
        // pusha; mov dword_623E10, 0
        &[
            0x60, 0xC7, 0x05, 0x10, 0x3E, 0x62, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        &[0x48],
        &[(0x44B538, 1)],
        &INPUTS,
        4,
        &|r, p| unsafe { draw_fade_overlay(r, p) },
    )
}
