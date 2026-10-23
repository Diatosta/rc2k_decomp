//! `scene::project_polygon` (0x44E1F7): finds or makes the record of each of the
//! polygon's points and counts those outside the depths that can be drawn.
//!
//! In:  nothing
//! Out: eax, ecx, edx, esi as the last point's work leaves them (eax and edx as
//!      `scene::project_point`, ecx = the number of vertices, esi = the last entry);
//!      everything else preserved
//!
//! For each entry of the polygon: its point (+0) through `scene::project_point`, the
//! record's address to its +4; `dword_6240F4`, 0 at the start, counts the points whose
//! depth is under 4096 (or is no number) or is `flt_6241C8` or more. The callers
//! compare the count with the number of vertices (all outside: nothing to draw) and
//! with 0 (none: no clipping by depth).
//! Eight callers. A polygon of no vertices is refused (the original goes through 2^32).
//! Self-test: polygons of the test's on points, records and views of the test's.
//!
//! Converted 2026-10-21: the hook's Rust version is `project_polygon_plain`, on
//! `scene::project_point_plain` and `rc2k_core::scene::projection::outside_depths` (the x87
//! version stays for the Rust callers in unconverted code). Shadow mode and the self-test
//! compare the records' floats and `dword_62400C` within `service::floats`' tolerances
//! (the self-test at both test precisions, exact at double); the count decides on the
//! stored depths, as the original.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, ECX, EDX, ESI, Regs, game_read, game_write, rva};
use crate::scene::polygon::{dword, listed, put};
use crate::scene::project_point::{
    NEAR, POINTS_RVA, RECORDS_RVA, project_point, project_point_plain, record_offset,
};
use crate::screens::harness::Left;

// ---- game data ----

pub const OUTSIDE_RVA: usize = rva(0x6240F4); // dword_6240F4
const FAR_RVA: usize = rva(0x6241C8); // flt_6241C8

// ---- Rust version ----

pub unsafe fn project_polygon() -> Result<Left, Refused> {
    unsafe {
        let entries = listed()?;
        let Some(&last) = entries.last() else {
            return Err(BadInput("a polygon of no vertices").into());
        };
        // Nothing may be written before a refusal: every point is looked at first.
        // The scene's points are only needed for a scene's point: the menus have none,
        // and their objects' points (negative) only need the records.
        let scene_point = entries.iter().any(|&entry| dword(entry) as i32 >= 0);
        if game_read::<u32>(RECORDS_RVA) == 0 || (scene_point && game_read::<u32>(POINTS_RVA) == 0)
        {
            return Err(BadInput("no points or no records for them").into());
        }
        for &entry in &entries {
            record_offset(dword(entry))?;
        }
        let far = f32::from_bits(game_read(FAR_RVA));
        let mut outside = 0u32;
        let mut done = None;
        for &entry in &entries {
            let point = project_point(dword(entry))?;
            put(entry + 4, point.record);
            let depth = f32::from_bits(dword(point.record as usize + 0x18));
            // Two compares, each taking "no number" for "below".
            let near = depth.partial_cmp(&NEAR).is_none_or(|order| order.is_lt());
            let in_reach = depth.partial_cmp(&far).is_none_or(|order| order.is_lt());
            if near || !in_reach {
                outside += 1;
            }
            done = Some(point);
        }
        game_write(OUTSIDE_RVA, outside);
        let done = done.expect("the polygon has a vertex");
        Ok(Left::nothing()
            .with(EAX, done.eax)
            .with(ECX, entries.len() as u32)
            .with(EDX, done.record)
            .with(ESI, last as u32))
    }
}

/// The converted `project_polygon` (plain floats): the hook's. The x87 version above stays
/// for the Rust callers in unconverted code.
// Converted: `rc2k_core::scene::projection::outside_depths`
pub unsafe fn project_polygon_plain() -> Result<Left, Refused> {
    unsafe {
        let entries = listed()?;
        let Some(&last) = entries.last() else {
            return Err(BadInput("a polygon of no vertices").into());
        };
        let scene_point = entries.iter().any(|&entry| dword(entry) as i32 >= 0);
        if game_read::<u32>(RECORDS_RVA) == 0 || (scene_point && game_read::<u32>(POINTS_RVA) == 0)
        {
            return Err(BadInput("no points or no records for them").into());
        }
        for &entry in &entries {
            record_offset(dword(entry))?;
        }
        let far = f32::from_bits(game_read(FAR_RVA));
        let mut outside = 0u32;
        let mut done = None;
        for &entry in &entries {
            let point = project_point_plain(dword(entry))?;
            put(entry + 4, point.record);
            let depth = f32::from_bits(dword(point.record as usize + 0x18));
            if rc2k_core::scene::projection::outside_depths(depth, far) {
                outside += 1;
            }
            done = Some(point);
        }
        game_write(OUTSIDE_RVA, outside);
        let done = done.expect("the polygon has a vertex");
        Ok(Left::nothing()
            .with(EAX, done.eax)
            .with(ECX, entries.len() as u32)
            .with(EDX, done.record)
            .with(ESI, last as u32))
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { project_polygon_plain() }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::service::floats::floats_entry(r, &rust)
}
