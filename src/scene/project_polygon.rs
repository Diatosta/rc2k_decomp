//! `scene::project_polygon` (0x44E1F7): finds or makes the record of each of the
//! polygon's points and counts those outside the depths that can be drawn.
//!
//! In:  nothing
//! Out: eax, ecx, edx, esi as the last point's work leaves them (eax and edx as
//!      `scene::project_point`, ecx = the number of vertices, esi = the last entry);
//!      everything else preserved
//!
//! For each entry of the polygon: its point (+0) through `scene::project_point`, the record
//! (`scene::point_records`' number since phase 13; the original's address) to its +4; `dword_6240F4`, 0 at the start, counts the points whose depth is under
//! 4096 (or is no number) or is `flt_6241C8` or more. The callers compare the count with the
//! number of vertices (all outside: nothing to draw) and with 0 (none: no clipping by depth).
//! Eight callers. A polygon of no vertices is refused (the original goes through 2^32).
//!
//! Converted 2026-10-21: the Rust version is `project_polygon_plain`, on
//! `scene::project_point_plain` and `rc2k_core::scene::projection::outside_depths` (the x87
//! version stays for the Rust callers in unconverted code).

use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::scene::polygon::{Listed, listed};
use crate::scene::project_point::{NEAR, project_point};

// ---- game data ----

// ---- Rust version ----

pub fn project_polygon(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    project_entries(race, &listed()?, scene)
}

/// `project_polygon` on the polygon's entries as listed.
pub fn project_entries(
    race: &Race,
    entries: &Listed,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let course = &race.course;
    let drawing = &scene.drawing;
    if entries.is_empty() {
        return Err(BadInput("a polygon of no vertices").into());
    }
    // The scene's points are only needed for a scene's point: the menus have none,
    // and their objects' points (negative) only need the records.
    let scene_point = entries.iter().any(|entry| entry.point.get() as i32 >= 0);
    if !crate::scene::point_records::held()
        || (scene_point && course.course.scene_points.bits() == 0)
    {
        return Err(BadInput("no points or no records for them").into());
    }
    let far = f32::from_bits(drawing.scene.farthest.bits());
    let mut outside = 0u32;
    for entry in entries {
        let point = project_point(race, entry.point.get(), scene)?;
        entry.record.set(point.record);
        let depth = f32::from_bits(crate::scene::point_records::field(point.record, 0x18)?);
        // Two compares, each taking "no number" for "below".
        let near = depth
            .partial_cmp(&NEAR)
            .is_none_or(std::cmp::Ordering::is_lt);
        let in_reach = depth
            .partial_cmp(&far)
            .is_none_or(std::cmp::Ordering::is_lt);
        if near || !in_reach {
            outside += 1;
        }
    }
    drawing.scene.polygon_outside_count.set_bits(outside);
    Ok(())
}
