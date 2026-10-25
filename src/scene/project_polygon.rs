//! `scene::project_polygon` (0x44E1F7): finds or makes the record of each of the
//! polygon's points and counts those outside the depths that can be drawn.
//!
//! In:  nothing
//! Out: eax, ecx, edx, esi as the last point's work leaves them (eax and edx as
//!      `scene::project_point`, ecx = the number of vertices, esi = the last entry);
//!      everything else preserved
//!
//! For each entry of the polygon: its point (+0) through `scene::project_point`, the record's
//! address to its +4; `dword_6240F4`, 0 at the start, counts the points whose depth is under
//! 4096 (or is no number) or is `flt_6241C8` or more. The callers compare the count with the
//! number of vertices (all outside: nothing to draw) and with 0 (none: no clipping by depth).
//! Eight callers. A polygon of no vertices is refused (the original goes through 2^32).
//!
//! Converted 2026-10-21: the Rust version is `project_polygon_plain`, on
//! `scene::project_point_plain` and `rc2k_core::scene::projection::outside_depths` (the x87
//! version stays for the Rust callers in unconverted code).

use crate::errors::{BadInput, Refused};
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::course_store::COURSE;
use crate::scene::draw_store::DRAW;
use crate::scene::polygon::{dword, listed, put};
use crate::scene::project_point::{NEAR, project_point, record_offset};

// ---- game data ----

// ---- Rust version ----

pub unsafe fn project_polygon() -> Result<(), Refused> {
    unsafe {
        let entries = listed()?;
        if entries.is_empty() {
            return Err(BadInput("a polygon of no vertices").into());
        }
        // Nothing may be written before a refusal: every point is looked at first.
        // The scene's points are only needed for a scene's point: the menus have none,
        // and their objects' points (negative) only need the records.
        let scene_point = entries.iter().any(|&entry| dword(entry) as i32 >= 0);
        if COURSE_LISTS.point_records.bits() == 0
            || (scene_point && COURSE.scene_points.bits() == 0)
        {
            return Err(BadInput("no points or no records for them").into());
        }
        for &entry in &entries {
            record_offset(dword(entry))?;
        }
        let far = f32::from_bits(DRAW.farthest.bits());
        let mut outside = 0u32;
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
        }
        DRAW.polygon_outside_count.set_bits(outside);
        Ok(())
    }
}
