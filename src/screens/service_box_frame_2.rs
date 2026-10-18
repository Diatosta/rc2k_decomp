//! `screens::service_box_frame_2` (0x491987): the frame (yellow 0xFFFF00) round box `dword_75AC90` (0 .. 4) of the first row, shown while it is the chosen one or (with `dword_75AA9C`) the cursor is on item 0xE.
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
use crate::screens::service_boxes::{Inputs, Part, addr, at_esi, quad, quad_part, rd, test_boxes};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::service_box_frame_2",
    rva(0x491987),
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
        (0x7559A4, &[0x1]),
        (0x75AA9C, &[0x1]),
        (0x75AC90, &[0x0, 0x1, 0x2, 0x3, 0x4]),
    ],
    read: &[0x755525, 0x75AC94],
};

/// The Rust version (see the header): the registers it leaves.
#[allow(unused_assignments, clippy::needless_late_init)] // the translator's flags, one per `cmp`
pub unsafe fn service_box_frame_2(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut part = part;
    let mut g = *entry;
    let mut c: (u32, u32);
    c = (rd(0x75AA9C), 0x1);
    'l492528: {
        'l4919cf: {
            'l4919ad: {
                if c.0 != c.1 {
                    break 'l4919ad;
                }
                g.eax = 0xE;
                c = (rd(0x755525), g.eax);
                if c.0 != c.1 {
                    break 'l4919ad;
                }
                c = (rd(0x7559A4), 0x1);
                if c.0 != c.1 {
                    break 'l492528;
                }
                break 'l4919cf;
            }
            g.eax = rd(0x75AC90);
            c = (rd(0x75AC94), g.eax);
            if c.0 != c.1 {
                break 'l4919cf;
            }
            g.eax = 0xE;
            g.esi = addr(0x759B6E).wrapping_add(g.eax.wrapping_mul(4));
            c = (at_esi(g.esi), 0x1);
            if c.0 != c.1 {
                break 'l492528;
            }
        }
        c = (rd(0x75AC90), 0x0);
        'l491c14: {
            if c.0 != c.1 {
                break 'l491c14;
            }
            quad(&mut g, 0x44, 0x12B, 0x41, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x44, 0x12B, 0x1, 0x27, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x85, 0x12B, 0x1, 0x28, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x44, 0x152, 0x41, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x45, 0x12C, 0x3F, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x45, 0x12C, 0x1, 0x25, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x84, 0x12C, 0x1, 0x26, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x45, 0x151, 0x3F, 0x1, 0xFFFF00, 0xFF, &mut part)?;
        }
        c = (rd(0x75AC90), 0x1);
        'l491e59: {
            if c.0 != c.1 {
                break 'l491e59;
            }
            quad(&mut g, 0x86, 0x12B, 0x41, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x86, 0x12B, 0x1, 0x27, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0xC7, 0x12B, 0x1, 0x28, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x86, 0x152, 0x41, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x87, 0x12C, 0x3F, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x87, 0x12C, 0x1, 0x25, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0xC6, 0x12C, 0x1, 0x26, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x87, 0x151, 0x3F, 0x1, 0xFFFF00, 0xFF, &mut part)?;
        }
        c = (rd(0x75AC90), 0x2);
        'l49209e: {
            if c.0 != c.1 {
                break 'l49209e;
            }
            quad(&mut g, 0xC8, 0x12B, 0x41, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0xC8, 0x12B, 0x1, 0x27, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x109, 0x12B, 0x1, 0x28, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0xC8, 0x152, 0x41, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0xC9, 0x12C, 0x3F, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0xC9, 0x12C, 0x1, 0x25, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x108, 0x12C, 0x1, 0x26, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0xC9, 0x151, 0x3F, 0x1, 0xFFFF00, 0xFF, &mut part)?;
        }
        c = (rd(0x75AC90), 0x3);
        'l4922e3: {
            if c.0 != c.1 {
                break 'l4922e3;
            }
            quad(&mut g, 0x10A, 0x12B, 0x41, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x10A, 0x12B, 0x1, 0x27, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x14B, 0x12B, 0x1, 0x28, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x10A, 0x152, 0x41, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x10B, 0x12C, 0x3F, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x10B, 0x12C, 0x1, 0x25, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x14A, 0x12C, 0x1, 0x26, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x10B, 0x151, 0x3F, 0x1, 0xFFFF00, 0xFF, &mut part)?;
        }
        c = (rd(0x75AC90), 0x4);
        if c.0 != c.1 {
            break 'l492528;
        }
        quad(&mut g, 0x14C, 0x12B, 0x41, 0x1, 0xFFFF00, 0xFF, &mut part)?;
        quad(&mut g, 0x14C, 0x12B, 0x1, 0x27, 0xFFFF00, 0xFF, &mut part)?;
        quad(&mut g, 0x18D, 0x12B, 0x1, 0x28, 0xFFFF00, 0xFF, &mut part)?;
        quad(&mut g, 0x14C, 0x152, 0x41, 0x1, 0xFFFF00, 0xFF, &mut part)?;
        quad(&mut g, 0x14D, 0x12C, 0x3F, 0x1, 0xFFFF00, 0xFF, &mut part)?;
        quad(&mut g, 0x14D, 0x12C, 0x1, 0x25, 0xFFFF00, 0xFF, &mut part)?;
        quad(&mut g, 0x18C, 0x12C, 0x1, 0x26, 0xFFFF00, 0xFF, &mut part)?;
        quad(&mut g, 0x14D, 0x151, 0x3F, 0x1, 0xFFFF00, 0xFF, &mut part)?;
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
    if crate::screens::blanked("screens::service_box_frame_2") {
        return;
    }
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { service_box_frame_2(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    test_boxes(
        &HOOK,
        0x491987,
        0xBA3,
        &[0x60, 0x83, 0x3D, 0x9C, 0xAA, 0x75, 0x00, 0x01],
        &[
            0x97, 0xDE, 0x125, 0x16C, 0x1B3, 0x1FA, 0x241, 0x288, 0x2DC, 0x323, 0x36A, 0x3B1,
            0x3F8, 0x43F, 0x486, 0x4CD, 0x521, 0x568, 0x5AF, 0x5F6, 0x63D, 0x684, 0x6CB, 0x712,
            0x766, 0x7AD, 0x7F4, 0x83B, 0x882, 0x8C9, 0x910, 0x957, 0x9AB, 0x9F2, 0xA39, 0xA80,
            0xAC7, 0xB0E, 0xB55, 0xB9C,
        ],
        &[(0x44B538, 1)],
        &INPUTS,
        16,
        &|r, p| unsafe { service_box_frame_2(r, p) },
    )
}
