//! `screens::service_box_frame_1` (0x490E22): the frame (orange 0xFF8000, solid) round box `dword_75AC94` (0 .. 4) of the service area's first row.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `screens::draw_service_boxes`. The body is the
//! original's code written out by `service_boxes`' translator; the quads through `part` (1,
//! in the game `draw_tinted_quad`'s Rust version). Shadow mode runs the original alone (it
//! draws). Self-test: `service_boxes::test_boxes` (16 cases).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva, shadow};
use crate::screens::service_boxes::{Inputs, Part, quad, quad_part, rd, test_boxes};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::service_box_frame_1",
    rva(0x490E22),
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

/// What the code tests and reads.
pub static INPUTS: Inputs = Inputs {
    tested: &[(0x75AA9C, &[0x1]), (0x75AC94, &[0x0, 0x1, 0x2, 0x3, 0x4])],
    read: &[],
};

/// The Rust version (see the header): the registers it leaves.
#[allow(unused_assignments, clippy::needless_late_init)] // the translator's flags, one per `cmp`
pub unsafe fn service_box_frame_1(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut part = part;
    let mut g = *entry;
    let mut c: (u32, u32);
    c = (rd(0x75AA9C), 0x1);
    'l490e2c: {
        if c.0 != c.1 {
            break 'l490e2c;
        }
    }
    c = (rd(0x75AC94), 0x0);
    'l491071: {
        if c.0 != c.1 {
            break 'l491071;
        }
        quad(&mut g, 0x44, 0x12B, 0x41, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x44, 0x12B, 0x1, 0x27, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x85, 0x12B, 0x1, 0x28, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x44, 0x152, 0x41, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x45, 0x12C, 0x3F, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x45, 0x12C, 0x1, 0x25, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x84, 0x12C, 0x1, 0x26, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x45, 0x151, 0x3F, 0x1, 0xFF8000, 0xFF, &mut part)?;
    }
    c = (rd(0x75AC94), 0x1);
    'l4912b6: {
        if c.0 != c.1 {
            break 'l4912b6;
        }
        quad(&mut g, 0x86, 0x12B, 0x41, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x86, 0x12B, 0x1, 0x27, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0xC7, 0x12B, 0x1, 0x28, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x86, 0x152, 0x41, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x87, 0x12C, 0x3F, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x87, 0x12C, 0x1, 0x25, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0xC6, 0x12C, 0x1, 0x26, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x87, 0x151, 0x3F, 0x1, 0xFF8000, 0xFF, &mut part)?;
    }
    c = (rd(0x75AC94), 0x2);
    'l4914fb: {
        if c.0 != c.1 {
            break 'l4914fb;
        }
        quad(&mut g, 0xC8, 0x12B, 0x41, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0xC8, 0x12B, 0x1, 0x27, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x109, 0x12B, 0x1, 0x28, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0xC8, 0x152, 0x41, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0xC9, 0x12C, 0x3F, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0xC9, 0x12C, 0x1, 0x25, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x108, 0x12C, 0x1, 0x26, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0xC9, 0x151, 0x3F, 0x1, 0xFF8000, 0xFF, &mut part)?;
    }
    c = (rd(0x75AC94), 0x3);
    'l491740: {
        if c.0 != c.1 {
            break 'l491740;
        }
        quad(&mut g, 0x10A, 0x12B, 0x41, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x10A, 0x12B, 0x1, 0x27, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x14B, 0x12B, 0x1, 0x28, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x10A, 0x152, 0x41, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x10B, 0x12C, 0x3F, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x10B, 0x12C, 0x1, 0x25, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x14A, 0x12C, 0x1, 0x26, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x10B, 0x151, 0x3F, 0x1, 0xFF8000, 0xFF, &mut part)?;
    }
    c = (rd(0x75AC94), 0x4);
    'l491985: {
        if c.0 != c.1 {
            break 'l491985;
        }
        quad(&mut g, 0x14C, 0x12B, 0x41, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x14C, 0x12B, 0x1, 0x27, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x18D, 0x12B, 0x1, 0x28, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x14C, 0x152, 0x41, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x14D, 0x12C, 0x3F, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x14D, 0x12C, 0x1, 0x25, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x18C, 0x12C, 0x1, 0x26, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x14D, 0x151, 0x3F, 0x1, 0xFF8000, 0xFF, &mut part)?;
    }
    let _ = (&mut g, &mut part);
    Ok(*entry)
}

/// In the game: the quads' Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    quad_part(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if crate::screens::blanked("screens::service_box_frame_1") {
        return;
    }
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { service_box_frame_1(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    test_boxes(
        &HOOK,
        0x490E22,
        0xB65,
        &[0x60, 0x83, 0x3D, 0x9C, 0xAA, 0x75, 0x00, 0x01],
        &[
            0x59, 0xA0, 0xE7, 0x12E, 0x175, 0x1BC, 0x203, 0x24A, 0x29E, 0x2E5, 0x32C, 0x373, 0x3BA,
            0x401, 0x448, 0x48F, 0x4E3, 0x52A, 0x571, 0x5B8, 0x5FF, 0x646, 0x68D, 0x6D4, 0x728,
            0x76F, 0x7B6, 0x7FD, 0x844, 0x88B, 0x8D2, 0x919, 0x96D, 0x9B4, 0x9FB, 0xA42, 0xA89,
            0xAD0, 0xB17, 0xB5E,
        ],
        &[(0x44B538, 1)],
        &INPUTS,
        16,
        &|r, p| unsafe { service_box_frame_1(r, p) },
    )
}
