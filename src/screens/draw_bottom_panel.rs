//! `screens::draw_bottom_panel` (0x49FCCF): a page's backdrop: the whole page above 0x1E3
//! and the bottom bar round its middle, see-through blue 0x11166A (`dword_75AA94`), the
//! middle's strip dark grey 0x1F1F1F; each a `scene::draw_tinted_quad`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x7590BA). Shadow mode runs the original alone (it draws).
//! Self-test: `service_boxes::test_boxes` (4 cases).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva, shadow};
use crate::screens::service_boxes::{Inputs, Part, quad, quad_part, rd, test_boxes};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_bottom_panel",
    rva(0x49FCCF),
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

pub static INPUTS: Inputs = Inputs {
    tested: &[],
    read: &[0x75AA94],
};

/// The Rust version: every register kept.
pub unsafe fn draw_bottom_panel(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut part = part;
    let see = rd(0x75AA94);
    for (x, y, w, h, colour) in [
        (0u32, 0u32, 0x320u32, 0x1E3u32, 0x11166Au32),
        (0, 0x1E3, 0xED, 0x75, 0x11166A),
        (0xED, 0x1F6, 0x233, 0x62, 0x11166A),
        (0x233, 0x1E3, 0xED, 0x13, 0x11166A),
        (0x128, 0x1E2, 0xD2, 0x15, 0x1F1F1F),
    ] {
        // Each inside `pusha` .. `popa`.
        let mut g = *entry;
        quad(&mut g, x, y, w, h, colour, see, &mut part)?;
    }
    Ok(*entry)
}

pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    quad_part(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if crate::screens::blanked("screens::draw_bottom_panel") {
        return;
    }
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_bottom_panel(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    test_boxes(
        &HOOK,
        0x49FCCF,
        0x17C,
        &[0x60, 0xC7, 0x05, 0x10, 0x3E, 0x62, 0x00, 0x00],
        &[0x48, 0x93, 0xDE, 0x129, 0x174],
        &[(0x44B538, 1)],
        &INPUTS,
        4,
        &|r, p| unsafe { draw_bottom_panel(r, p) },
    )
}
