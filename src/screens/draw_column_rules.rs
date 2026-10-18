//! `screens::draw_column_rules` (0x49FFC5): the rules between a two-column page's sixteen
//! rows: in each column (x 0x43 and 0x192) from y 0x5F every 0x1D a line 0x14C wide and 3
//! high, see-through blue 0x11166A (`dword_75AA94`; `scene::draw_tinted_quad`).
//!
//! In:  nothing
//! Out: ecx -1, edx past the last row; the rest as the last quad leaves them
//!
//! Caller: `screens::draw_two_column_panel`. Shadow mode runs the original alone (it draws).
//! Self-test: `service_boxes::test_boxes` (4 cases).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva, shadow};
use crate::screens::service_boxes::{Inputs, Part, quad, quad_part, rd, test_boxes};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_column_rules",
    rva(0x49FFC5),
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

/// The Rust version: the registers it leaves.
pub unsafe fn draw_column_rules(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut part = part;
    let mut g = *entry;
    for x in [0x43u32, 0x192] {
        let mut y = 0x5Fu32;
        for n in (0..0x10u32).rev() {
            // `push ecx; push edx` .. `pop edx; pop ecx` round the quad.
            (g.ecx, g.edx) = (n, y);
            quad(&mut g, x, y, 0x14C, 3, 0x11166A, rd(0x75AA94), &mut part)?;
            y += 0x1D;
        }
        g.ecx = u32::MAX;
        g.edx = y;
    }
    Ok(g)
}

pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    quad_part(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if crate::screens::blanked("screens::draw_column_rules") {
        return;
    }
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_column_rules(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    test_boxes(
        &HOOK,
        0x49FFC5,
        0xB3,
        &[0xB9, 0x0F, 0x00, 0x00, 0x00, 0xBA, 0x5F, 0x00],
        &[0x4C, 0xA5],
        &[(0x44B538, 1)],
        &INPUTS,
        4,
        &|r, p| unsafe { draw_column_rules(r, p) },
    )
}
