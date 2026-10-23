//! `screens::service_box_frame_4` (0x4932D4): the frame (yellow) round box `dword_75AC9C` of the second row, shown as `service_box_frame_2`'s.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `screens::draw_service_boxes`. The body is the
//! original's code written out by `service_boxes`' translator; the quads through `part` (1,
//! in the game `draw_tinted_quad`'s Rust version). Shadow mode runs the original alone (it
//! draws). Self-test: `service_boxes::test_boxes` (16 cases).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::service_boxes::{Inputs, Part, addr, at_esi, quad, quad_part, rd};

/// What the code tests and reads.
pub static INPUTS: Inputs = Inputs {
    tested: &[
        (0x7559A4, &[0x1]),
        (0x75AA9C, &[0x1]),
        (0x75AC9C, &[0x0, 0x1, 0x2, 0x3, 0x4, 0x5]),
    ],
    read: &[0x755525, 0x75ACA0],
};

/// The Rust version (see the header): the registers it leaves.
#[allow(unused_assignments, clippy::needless_late_init)] // the translator's flags, one per `cmp`
pub unsafe fn service_box_frame_4(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut part = part;
    let mut g = *entry;
    let mut c: (u32, u32);
    c = (rd(0x75AA9C), 0x1);
    'l4940ba: {
        'l49331c: {
            'l4932fa: {
                if c.0 != c.1 {
                    break 'l4932fa;
                }
                g.eax = 0xF;
                c = (rd(0x755525), g.eax);
                if c.0 != c.1 {
                    break 'l4932fa;
                }
                c = (rd(0x7559A4), 0x1);
                if c.0 != c.1 {
                    break 'l4940ba;
                }
                break 'l49331c;
            }
            g.eax = rd(0x75AC9C);
            c = (rd(0x75ACA0), g.eax);
            if c.0 != c.1 {
                break 'l49331c;
            }
            g.eax = 0xF;
            g.esi = addr(0x759B6E).wrapping_add(g.eax.wrapping_mul(4));
            c = (at_esi(g.esi), 0x1);
            if c.0 != c.1 {
                break 'l4940ba;
            }
        }
        c = (rd(0x75AC9C), 0x0);
        'l493561: {
            if c.0 != c.1 {
                break 'l493561;
            }
            quad(&mut g, 0x193, 0x12B, 0x36, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x193, 0x12B, 0x1, 0x27, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x1C9, 0x12B, 0x1, 0x28, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x193, 0x152, 0x36, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x194, 0x12C, 0x34, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x194, 0x12C, 0x1, 0x25, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x1C8, 0x12C, 0x1, 0x26, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x194, 0x151, 0x34, 0x1, 0xFFFF00, 0xFF, &mut part)?;
        }
        c = (rd(0x75AC9C), 0x1);
        'l4937a6: {
            if c.0 != c.1 {
                break 'l4937a6;
            }
            quad(&mut g, 0x1CA, 0x12B, 0x36, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x1CA, 0x12B, 0x1, 0x27, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x200, 0x12B, 0x1, 0x28, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x1CA, 0x152, 0x36, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x1CB, 0x12C, 0x34, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x1CB, 0x12C, 0x1, 0x25, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x1FF, 0x12C, 0x1, 0x26, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x1CB, 0x151, 0x34, 0x1, 0xFFFF00, 0xFF, &mut part)?;
        }
        c = (rd(0x75AC9C), 0x2);
        'l4939eb: {
            if c.0 != c.1 {
                break 'l4939eb;
            }
            quad(&mut g, 0x201, 0x12B, 0x36, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x201, 0x12B, 0x1, 0x27, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x237, 0x12B, 0x1, 0x28, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x201, 0x152, 0x36, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x202, 0x12C, 0x34, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x202, 0x12C, 0x1, 0x25, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x236, 0x12C, 0x1, 0x26, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x202, 0x151, 0x34, 0x1, 0xFFFF00, 0xFF, &mut part)?;
        }
        c = (rd(0x75AC9C), 0x3);
        'l493c30: {
            if c.0 != c.1 {
                break 'l493c30;
            }
            quad(&mut g, 0x238, 0x12B, 0x36, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x238, 0x12B, 0x1, 0x27, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x26E, 0x12B, 0x1, 0x28, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x238, 0x152, 0x36, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x239, 0x12C, 0x34, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x239, 0x12C, 0x1, 0x25, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x26D, 0x12C, 0x1, 0x26, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x239, 0x151, 0x34, 0x1, 0xFFFF00, 0xFF, &mut part)?;
        }
        c = (rd(0x75AC9C), 0x4);
        'l493e75: {
            if c.0 != c.1 {
                break 'l493e75;
            }
            quad(&mut g, 0x26F, 0x12B, 0x36, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x26F, 0x12B, 0x1, 0x27, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x2A5, 0x12B, 0x1, 0x28, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x26F, 0x152, 0x36, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x270, 0x12C, 0x34, 0x1, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x270, 0x12C, 0x1, 0x25, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x2A4, 0x12C, 0x1, 0x26, 0xFFFF00, 0xFF, &mut part)?;
            quad(&mut g, 0x270, 0x151, 0x34, 0x1, 0xFFFF00, 0xFF, &mut part)?;
        }
        c = (rd(0x75AC9C), 0x5);
        if c.0 != c.1 {
            break 'l4940ba;
        }
        quad(&mut g, 0x2A6, 0x12B, 0x36, 0x1, 0xFFFF00, 0xFF, &mut part)?;
        quad(&mut g, 0x2A6, 0x12B, 0x1, 0x27, 0xFFFF00, 0xFF, &mut part)?;
        quad(&mut g, 0x2DC, 0x12B, 0x1, 0x28, 0xFFFF00, 0xFF, &mut part)?;
        quad(&mut g, 0x2A6, 0x152, 0x36, 0x1, 0xFFFF00, 0xFF, &mut part)?;
        quad(&mut g, 0x2A7, 0x12C, 0x34, 0x1, 0xFFFF00, 0xFF, &mut part)?;
        quad(&mut g, 0x2A7, 0x12C, 0x1, 0x25, 0xFFFF00, 0xFF, &mut part)?;
        quad(&mut g, 0x2DB, 0x12C, 0x1, 0x26, 0xFFFF00, 0xFF, &mut part)?;
        quad(&mut g, 0x2A7, 0x151, 0x34, 0x1, 0xFFFF00, 0xFF, &mut part)?;
    }
    let _ = (&mut g, &mut part);
    Ok(*entry)
}

/// In the game: the quads' Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    quad_part(r)
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    if crate::screens::blanked("screens::service_box_frame_4") {
        return Ok(());
    }
    let left = unsafe { service_box_frame_4(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
