//! `scene::clip_polygon_to_screen` (0x44D15A): cuts the polygon to the screen's four
//! edges.
//!
//! In:  nothing
//! Out: the original leaves eax, ecx, edx, esi changed, and after a clipping ebx, edi
//!      and ebp too; its callers go on by the number of vertices alone; the hook gives a
//!      polygon wholly on the screen the original's eax, ecx, edx, esi (`unclipped_regs`)
//!      and keeps the registers after a clipping
//!
//! `dword_624120` = 1. Each vertex's place on the screen (its record's +0xC and +0x1C)
//! against the edges `flt_6241EC` (left), `flt_6241F0` (right), `flt_6241E4` (top) and
//! `flt_6241E8` (bottom) gives the bits 1, 2, 4, 8 of a mask: right is looked at
//! first and left only if the vertex is not past it, bottom first and top only if not
//! past that. A mask of 0: nothing more. Else the mask to `dword_624118`, and for
//! each of its bits in that order one pass of `scene/clip.rs`, ending as soon as
//! fewer than three vertices are left.
//! 14 callers. Refuses a polygon of no vertices, an entry without its record, and a
//! clipping that runs out of entries (what it had written is put back first).
//! Shadow mode: the Rust version, its writes put back, the original, compared.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::d3d::draw_fan_unclipped::COUNT_RVA;
use crate::errors::BadInput;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::scene::clip::{
    Edge, MASK_RVA, SCREEN_CLIPPED_RVA, below, clip_pass, listed_with_records, or_undone,
};
use crate::scene::polygon::dword;

// ---- game data ----

const LEFT_RVA: usize = rva(0x6241EC); // flt_6241EC
const RIGHT_RVA: usize = rva(0x6241F0); // flt_6241F0
const TOP_RVA: usize = rva(0x6241E4); // flt_6241E4
const BOTTOM_RVA: usize = rva(0x6241E8); // flt_6241E8

// ---- Rust version ----

/// The polygon's entries and the mask of the edges some vertex is past.
unsafe fn outside_mask() -> Result<u32, BadInput> {
    unsafe {
        let entries = listed_with_records()?;
        let edge = |at: usize| f32::from_bits(game_read(at)) as f64;
        let (left, right, top, bottom) = (
            edge(LEFT_RVA),
            edge(RIGHT_RVA),
            edge(TOP_RVA),
            edge(BOTTOM_RVA),
        );
        let mut mask = 0;
        for &entry in &entries {
            let record = dword(entry + 4) as usize;
            let x = f32::from_bits(dword(record + 0xC)) as f64;
            let y = f32::from_bits(dword(record + 0x1C)) as f64;
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
}

pub unsafe fn clip_polygon_to_screen() -> Result<(), BadInput> {
    unsafe {
        let mask = outside_mask()?;
        game_write(SCREEN_CLIPPED_RVA, 1u32);
        if mask == 0 {
            return Ok(());
        }
        game_write(MASK_RVA, mask);
        or_undone(|| {
            for (bit, edge, inside_past) in [
                (1, Edge::X(game_read(LEFT_RVA)), true),
                (2, Edge::X(game_read(RIGHT_RVA)), false),
                (4, Edge::Y(game_read(TOP_RVA)), true),
                (8, Edge::Y(game_read(BOTTOM_RVA)), false),
            ] {
                if mask & bit != 0 {
                    clip_pass(edge, inside_past, mask)?;
                    if game_read::<u32>(COUNT_RVA) < 3 {
                        break;
                    }
                }
            }
            Ok(())
        })
    }
}

/// The registers the original leaves for a polygon wholly on the screen (its mask loop's
/// end): eax the last listed entry's number x 0x44, ecx the count, edx 0, esi that entry's
/// vertex record. Callers' callees read none of them; set so that the registers match.
pub unsafe fn unclipped_regs(r: &mut Regs) {
    unsafe {
        use crate::scene::polygon::{ENTRIES_RVA, LIST_RVA};
        let n: u32 = game_read(COUNT_RVA);
        if n == 0 {
            return;
        }
        r.eax = game_read::<u32>(LIST_RVA + 4 * (n as usize - 1)).wrapping_mul(0x44);
        r.ecx = n;
        r.edx = 0;
        r.esi = ((crate::framework::game_addr(ENTRIES_RVA) as u32 + r.eax + 4) as *const u32)
            .read_unaligned();
    }
}

/// Whether no vertex is past an edge (the original's no-clip path).
pub unsafe fn wholly_on_screen() -> bool {
    matches!(unsafe { outside_mask() }, Ok(0))
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let unclipped = unsafe { wholly_on_screen() };
    unsafe { clip_polygon_to_screen() }?;
    if unclipped {
        unsafe { unclipped_regs(r) };
    }
    Ok(())
}
