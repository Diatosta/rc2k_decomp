//! `screens::draw_column_boxes` (0x4A0078): a two-column page's 32 boxes: in each column (x
//! 0x43, then 0x192; rows from y 0x45 every 0x1D, numbered by `dword_7559EC` 1 .. 0x20, the
//! place in `g_markerX` / `g_markerY`) a box 0x14D x 0x1B (0x4FCF, see-through
//! `dword_75AA90`) and its black border; the chosen row (`dword_755525`, while
//! `dword_7559A4` is 1) also a yellow (0xFFFF00) border and one a pixel outside it. Each a
//! `scene::draw_tinted_quad`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `screens::draw_two_column_panel`. Shadow mode runs the original alone (it draws).
//! Self-test: `service_boxes::test_boxes` (16 cases).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva, shadow};
use crate::screens::service_boxes::{Inputs, Part, quad, quad_part, rd, set, test_boxes};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_column_boxes",
    rva(0x4A0078),
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

const ROWS: [u32; 32] = {
    let mut v = [0u32; 32];
    let mut k = 0;
    while k < 32 {
        v[k] = k as u32 + 1;
        k += 1;
    }
    v
};

pub static INPUTS: Inputs = Inputs {
    tested: &[(0x755525, &ROWS), (0x7559A4, &[1])],
    read: &[0x75AA90],
};

const ROW: usize = 0x7559EC; // dword_7559EC
const MARKER_X: usize = 0x755A18; // g_markerX
const MARKER_Y: usize = 0x755A1C; // g_markerY

/// The Rust version: every register kept.
pub unsafe fn draw_column_boxes(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::screens::panels::row_box`; the row and marker variables
    // are left as the original leaves them.
    use rc2k_core::screens::panels::{BOXES_Y, COLUMNS, ROW_STEP, ROWS, row_box};
    let mut part = part;
    let mut g = *entry;
    set(ROW, 1);
    for x in COLUMNS {
        set(MARKER_X, x);
        set(MARKER_Y, BOXES_Y);
        for _ in 0..ROWS {
            let chosen = rd(0x755525) == rd(ROW) && rd(0x7559A4) == 1;
            for a in row_box(rd(MARKER_X), rd(MARKER_Y), rd(0x75AA90), chosen) {
                quad(&mut g, a.x, a.y, a.w, a.h, a.colour, a.see, &mut part)?;
            }
            set(MARKER_Y, rd(MARKER_Y).wrapping_add(ROW_STEP));
            set(ROW, rd(ROW).wrapping_add(1));
        }
    }
    Ok(*entry)
}

pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    quad_part(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if crate::screens::blanked("screens::draw_column_boxes") {
        return;
    }
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_column_boxes(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    if crate::screens::blanked("screens::draw_column_boxes") {
        return Ok(());
    }
    let left = unsafe { draw_column_boxes(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

fn self_test() -> (usize, usize) {
    test_boxes(
        &HOOK,
        0x4A0078,
        0x833,
        &[0x60, 0xC7, 0x05, 0xEC, 0x59, 0x75, 0x00, 0x01],
        &[
            0x63, 0xAC, 0xF3, 0x13F, 0x18B, 0x1F2, 0x239, 0x285, 0x2D1, 0x31C, 0x365, 0x3B2, 0x3FF,
            0x476, 0x4BF, 0x506, 0x552, 0x59E, 0x605, 0x64C, 0x698, 0x6E4, 0x72F, 0x778, 0x7C5,
            0x812,
        ],
        &[(0x44B538, 1)],
        &INPUTS,
        16,
        &|r, p| unsafe { draw_column_boxes(r, p) },
    )
}
