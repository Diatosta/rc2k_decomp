//! `scene::project_polygon`: finds or makes the record of each of the polygon's points and
//! counts those outside the depths that can be drawn.
//!
//! For each entry of the polygon: its point (+0) through `scene::project_point`, the record (its
//! number in `scene::point_records`) to its +4; `SceneDraw.polygon_outside_count`, 0 at the start,
//! counts the points whose depth is under 4096 (or is no number) or is `SceneDraw.farthest` or
//! more. The callers compare the count with the number of vertices (all outside: nothing to draw)
//! and with 0 (none: no clipping by depth). Refuses a polygon of no vertices.
//!
//! It runs on `scene::project_point` and
//! `rc2k_core::scene::projection::outside_depths`.

use crate::errors::{BadInput, Refused};
use crate::scene::polygon::{Listed, listed};
use crate::scene::project_point::{NEAR, project_point};

// ---- game data ----

pub fn project_polygon(
    part: &crate::scene::workers::PartScratch,
    poly: &crate::scene::polygon_work::PolygonWork,
    reads: crate::scene::workers::DrawReads<'_>,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), Refused> {
    let polygon_work = &part.polygon;
    project_entries(part, poly, &listed(polygon_work, poly)?, reads, view)
}

/// `project_polygon` on the polygon's entries as listed.
pub fn project_entries(
    part: &crate::scene::workers::PartScratch,
    poly: &crate::scene::polygon_work::PolygonWork,
    entries: &Listed,
    reads: crate::scene::workers::DrawReads<'_>,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), Refused> {
    if entries.is_empty() {
        return Err(BadInput("a polygon of no vertices").into());
    }
    // The scene's points are only needed for a scene's point: the menus have none,
    // and their objects' points (negative) only need the records.
    let scene_point = entries.iter().any(|entry| entry.point.get() as i32 >= 0);
    if !crate::scene::point_records::held(reads.course_lists)
        || (scene_point && !crate::race::scene_points::held(&reads.stage.course))
    {
        return Err(BadInput("no points or no records for them").into());
    }
    let far = view.polygon.window.farthest.get();
    let mut outside = 0u32;
    for entry in entries {
        let point = project_point(part, entry.point.get(), reads, view)?;
        entry.record.set(point.record);
        let depth = f32::from_bits(crate::scene::point_records::field(
            part,
            point.record,
            0x18,
        )?);
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
    poly.polygon_outside_count.set(outside);
    Ok(())
}
