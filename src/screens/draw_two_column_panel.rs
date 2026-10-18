//! `screens::draw_two_column_panel` (0x49FE4B): a two-column page's backdrop: the top bar,
//! the two side bars, the bottom bar and the middle rule (see-through blue 0x11166A,
//! `dword_75AA94`; `scene::draw_tinted_quad`), then the rows' rules
//! (`screens::draw_column_rules`) and boxes (`screens::draw_column_boxes`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! In two menu loops' draw lists (0x758F12, 0x758F1A). Shadow mode runs the original alone
//! (it draws). Self-test: `service_boxes::test_boxes` (4 cases), the two calls to stand-ins.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva, shadow};
use crate::screens::service_boxes::{Inputs, Part, quad, quad_part, rd, test_boxes};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_two_column_panel",
    rva(0x49FE4B),
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

/// The Rust version: every register kept. Parts: 1 the quads, 2 the rules, 3 the boxes.
pub unsafe fn draw_two_column_panel(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut part = part;
    let mut g = *entry;
    for (x, y, w, h) in [
        (0u32, 0u32, 0x320u32, 0x45u32),
        (0, 0x45, 0x43, 0x258),
        (0x2DE, 0x45, 0x42, 0x213),
        (0x43, 0x215, 0x29B, 0x43),
        (0x18F, 0x45, 3, 0x1D0),
    ] {
        quad(&mut g, x, y, w, h, 0x11166A, rd(0x75AA94), &mut part)?;
    }
    part(2, &mut g)?;
    part(3, &mut g)?;
    Ok(*entry)
}

pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::screens::{draw_column_boxes, draw_column_rules};
    unsafe {
        match place {
            1 => quad_part(r),
            2 => {
                *r = draw_column_rules::draw_column_rules(r, &mut draw_column_rules::game_part)?;
                Ok(())
            }
            _ => {
                *r = draw_column_boxes::draw_column_boxes(r, &mut draw_column_boxes::game_part)?;
                Ok(())
            }
        }
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if crate::screens::blanked("screens::draw_two_column_panel") {
        return;
    }
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_two_column_panel(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    test_boxes(
        &HOOK,
        0x49FE4B,
        0x17A,
        &[0x60, 0xC7, 0x05, 0x10, 0x3E, 0x62, 0x00, 0x00],
        &[0x45, 0x8E, 0xD7, 0x120, 0x169, 0x16E, 0x173],
        &[(0x44B538, 1), (0x49FFC5, 2), (0x4A0078, 3)],
        &INPUTS,
        4,
        &|r, p| unsafe { draw_two_column_panel(r, p) },
    )
}
