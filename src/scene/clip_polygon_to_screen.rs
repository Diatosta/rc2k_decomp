//! `scene::clip_polygon_to_screen` (0x44D15A): cuts the polygon to the screen's four
//! edges.
//!
//! In:  nothing
//! Out: the original leaves eax, ecx, edx, esi changed, and after a clipping ebx, edi
//!      and ebp too; its callers go on by the number of vertices alone (none reads them)
//!
//! `dword_624120` = 1. Each vertex's place on the screen (its record's +0xC and +0x1C) against
//! the edges `flt_6241EC` (left), `flt_6241F0` (right), `flt_6241E4` (top) and `flt_6241E8`
//! (bottom) gives the bits 1, 2, 4, 8 of a mask: right is looked at first and left only if the
//! vertex is not past it, bottom first and top only if not past that. A mask of 0: nothing
//! more. Else the mask to `dword_624118`, and for each of its bits in that order one pass of
//! `scene/clip.rs`, ending as soon as fewer than three vertices are left. 14 callers. Refuses a
//! polygon of no vertices, an entry without its record, and a clipping that runs out of entries
//! (what it had written is put back first).

use crate::errors::BadInput;
use crate::scene::clip::{Edge, below, clip_pass, listed_with_records, or_undone};
use crate::scene::draw_store::DRAW;
use crate::scene::polygon::dword;
use crate::session::Float;

// ---- game data ----

// ---- Rust version ----

/// The polygon's entries and the mask of the edges some vertex is past.
fn outside_mask() -> Result<u32, BadInput> {
    let entries = listed_with_records()?;
    let edge = |cell: &Float| cell.get() as f64;
    let (left, right, top, bottom) = (
        edge(&DRAW.clip_screen[2]),
        edge(&DRAW.clip_screen[3]),
        edge(&DRAW.clip_screen[0]),
        edge(&DRAW.clip_screen[1]),
    );
    let mut mask = 0;
    for &entry in &entries {
        let record = dword(entry + 4);
        let x = f32::from_bits(crate::scene::point_records::vertex(record, 0xC)) as f64;
        let y = f32::from_bits(crate::scene::point_records::vertex(record, 0x1C)) as f64;
        if !below(x, right) {
            mask |= 2;
        } else if below(x, left) {
            mask |= 1;
        }
        if !below(y, bottom) {
            mask |= 8;
        } else if below(y, top) {
            mask |= 4;
        }
    }
    Ok(mask)
}

pub unsafe fn clip_polygon_to_screen() -> Result<(), BadInput> {
    unsafe {
        let mask = outside_mask()?;
        DRAW.screen_clipped.set_bits(1u32);
        if mask == 0 {
            return Ok(());
        }
        DRAW.polygon_clip_mask.set_bits(mask);
        or_undone(|| {
            for (bit, edge, inside_past) in [
                (1, Edge::X(DRAW.clip_screen[2].bits()), true),
                (2, Edge::X(DRAW.clip_screen[3].bits()), false),
                (4, Edge::Y(DRAW.clip_screen[0].bits()), true),
                (8, Edge::Y(DRAW.clip_screen[1].bits()), false),
            ] {
                if mask & bit != 0 {
                    clip_pass(edge, inside_past, mask)?;
                    if DRAW.polygon_vertex_count.bits() < 3 {
                        break;
                    }
                }
            }
            Ok(())
        })
    }
}
