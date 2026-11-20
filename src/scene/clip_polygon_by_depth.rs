//! `scene::clip_polygon_by_depth`: cuts the polygon to the depths that can be drawn.
//!
//! A clip-flag cell = 1; the clip depth = 0x1000 and one pass of `scene/clip.rs` that keeps what is
//! deeper than that; then, with a second switch at 0, the far-clip switch not 0 and three vertices
//! or more left, the clip depth = the far depth and a pass that keeps what is not deeper. Not with
//! `far_clip` false: the own car over the view. Refuses as `scene::clip_polygon_to_screen`.

use crate::errors::BadInput;
use crate::scene::clip::{Edge, clip_pass, listed_with_records, or_undone};

// ---- game data ----

const NEAREST: i32 = 0x1000;

pub fn clip_polygon_by_depth(
    part: &crate::scene::scratch_store::PartScratch,
    poly: &crate::scene::polygon_work_store::PolygonWork,
    course: crate::race::owner_course::CourseReads<'_>,
    switches: &crate::scene::draw_switches_store::DrawSwitches,
    far_clip: bool,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), BadInput> {
    let polygon_work = &part.polygon;
    let entries = listed_with_records(polygon_work, poly)?;
    or_undone(|| {
        poly.depth_clipped.set(1u32);
        poly.clip_plane.set(NEAREST as u32);
        let listed = clip_pass(
            part,
            poly,
            course,
            Some(entries),
            Edge::Depth(NEAREST),
            true,
        )?;
        let far = far_clip && view.face.no_far_clip.get() == 0 && switches.far_on != 0;
        if far && (poly.polygon_vertex_count.get() as i32).wrapping_sub(3) >= 0 {
            let farthest = view.polygon.window.clip_farthest.get() as i32;
            poly.clip_plane.set(farthest as u32);
            clip_pass(part, poly, course, listed, Edge::Depth(farthest), false)?;
        }
        Ok(())
    })
}
