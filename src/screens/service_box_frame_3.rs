//! `screens::service_box_frame_3` (0x49252A): the frame (orange) round box `dword_75ACA0` of the service area's second row.
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
    "screens::service_box_frame_3",
    rva(0x49252A),
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
    tested: &[
        (0x75AA9C, &[0x1]),
        (0x75ACA0, &[0x0, 0x1, 0x2, 0x3, 0x4, 0x5]),
    ],
    read: &[],
};

/// The Rust version (see the header): the registers it leaves.
#[allow(unused_assignments, clippy::needless_late_init)] // the translator's flags, one per `cmp`
pub unsafe fn service_box_frame_3(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut part = part;
    let mut g = *entry;
    let mut c: (u32, u32);
    c = (rd(0x75AA9C), 0x1);
    'l492534: {
        if c.0 != c.1 {
            break 'l492534;
        }
    }
    c = (rd(0x75ACA0), 0x0);
    'l492779: {
        if c.0 != c.1 {
            break 'l492779;
        }
        quad(&mut g, 0x193, 0x12B, 0x36, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x193, 0x12B, 0x1, 0x27, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x1C9, 0x12B, 0x1, 0x28, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x193, 0x152, 0x36, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x194, 0x12C, 0x34, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x194, 0x12C, 0x1, 0x25, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x1C8, 0x12C, 0x1, 0x26, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x194, 0x151, 0x34, 0x1, 0xFF8000, 0xFF, &mut part)?;
    }
    c = (rd(0x75ACA0), 0x1);
    'l4929be: {
        if c.0 != c.1 {
            break 'l4929be;
        }
        quad(&mut g, 0x1CA, 0x12B, 0x36, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x1CA, 0x12B, 0x1, 0x27, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x200, 0x12B, 0x1, 0x28, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x1CA, 0x152, 0x36, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x1CB, 0x12C, 0x34, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x1CB, 0x12C, 0x1, 0x25, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x1FF, 0x12C, 0x1, 0x26, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x1CB, 0x151, 0x34, 0x1, 0xFF8000, 0xFF, &mut part)?;
    }
    c = (rd(0x75ACA0), 0x2);
    'l492c03: {
        if c.0 != c.1 {
            break 'l492c03;
        }
        quad(&mut g, 0x201, 0x12B, 0x36, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x201, 0x12B, 0x1, 0x27, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x237, 0x12B, 0x1, 0x28, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x201, 0x152, 0x36, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x202, 0x12C, 0x34, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x202, 0x12C, 0x1, 0x25, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x236, 0x12C, 0x1, 0x26, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x202, 0x151, 0x34, 0x1, 0xFF8000, 0xFF, &mut part)?;
    }
    c = (rd(0x75ACA0), 0x3);
    'l492e48: {
        if c.0 != c.1 {
            break 'l492e48;
        }
        quad(&mut g, 0x238, 0x12B, 0x36, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x238, 0x12B, 0x1, 0x27, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x26E, 0x12B, 0x1, 0x28, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x238, 0x152, 0x36, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x239, 0x12C, 0x34, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x239, 0x12C, 0x1, 0x25, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x26D, 0x12C, 0x1, 0x26, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x239, 0x151, 0x34, 0x1, 0xFF8000, 0xFF, &mut part)?;
    }
    c = (rd(0x75ACA0), 0x4);
    'l49308d: {
        if c.0 != c.1 {
            break 'l49308d;
        }
        quad(&mut g, 0x26F, 0x12B, 0x36, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x26F, 0x12B, 0x1, 0x27, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x2A5, 0x12B, 0x1, 0x28, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x26F, 0x152, 0x36, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x270, 0x12C, 0x34, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x270, 0x12C, 0x1, 0x25, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x2A4, 0x12C, 0x1, 0x26, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x270, 0x151, 0x34, 0x1, 0xFF8000, 0xFF, &mut part)?;
    }
    c = (rd(0x75ACA0), 0x5);
    'l4932d2: {
        if c.0 != c.1 {
            break 'l4932d2;
        }
        quad(&mut g, 0x2A6, 0x12B, 0x36, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x2A6, 0x12B, 0x1, 0x27, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x2DC, 0x12B, 0x1, 0x28, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x2A6, 0x152, 0x36, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x2A7, 0x12C, 0x34, 0x1, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x2A7, 0x12C, 0x1, 0x25, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x2DB, 0x12C, 0x1, 0x26, 0xFF8000, 0xFF, &mut part)?;
        quad(&mut g, 0x2A7, 0x151, 0x34, 0x1, 0xFF8000, 0xFF, &mut part)?;
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
    if crate::screens::blanked("screens::service_box_frame_3") {
        return;
    }
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { service_box_frame_3(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    if crate::screens::blanked("screens::service_box_frame_3") {
        return Ok(());
    }
    let left = unsafe { service_box_frame_3(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

fn self_test() -> (usize, usize) {
    test_boxes(
        &HOOK,
        0x49252A,
        0xDAA,
        &[0x60, 0x83, 0x3D, 0x9C, 0xAA, 0x75, 0x00, 0x01],
        &[
            0x59, 0xA0, 0xE7, 0x12E, 0x175, 0x1BC, 0x203, 0x24A, 0x29E, 0x2E5, 0x32C, 0x373, 0x3BA,
            0x401, 0x448, 0x48F, 0x4E3, 0x52A, 0x571, 0x5B8, 0x5FF, 0x646, 0x68D, 0x6D4, 0x728,
            0x76F, 0x7B6, 0x7FD, 0x844, 0x88B, 0x8D2, 0x919, 0x96D, 0x9B4, 0x9FB, 0xA42, 0xA89,
            0xAD0, 0xB17, 0xB5E, 0xBB2, 0xBF9, 0xC40, 0xC87, 0xCCE, 0xD15, 0xD5C, 0xDA3,
        ],
        &[(0x44B538, 1)],
        &INPUTS,
        16,
        &|r, p| unsafe { service_box_frame_3(r, p) },
    )
}
