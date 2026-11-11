//! `scene::clip_polygon_by_depth` (0x44DB4F): cuts the polygon to the depths that can
//! be drawn.
//!
//! `dword_624124` = 1; `dword_62411C` = 0x1000 and one pass of `scene/clip.rs` that
//! keeps what is deeper than that (0x44DB93); then, with `dword_621510` at 0,
//! `dword_621900` not 0 and three vertices or more left, `dword_62411C` =
//! `dword_624230` and a pass that keeps what is not deeper (0x44DEAA). Not with `far_clip`
//! false: the own car over the view (the original cleared `dword_621900` around it).
//! Seven callers. Refuses as `scene::clip_polygon_to_screen`.

use crate::errors::BadInput;
use crate::race::owner::Race;
use crate::scene::clip::{Edge, clip_pass, listed_with_records, or_undone};

// ---- game data ----

const NEAREST: i32 = 0x1000;

// ---- Rust version ----

pub fn clip_polygon_by_depth(
    poly: &crate::scene::polygon_work::PolygonWork,
    race: &Race,
    scene: &crate::scene::owner::Scene,
    far_clip: bool,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), BadInput> {
    let course = &race.course;
    let entries = listed_with_records(poly)?;
    or_undone(|| {
        poly.depth_clipped.set(1u32);
        poly.clip_plane.set(NEAREST as u32);
        let listed = clip_pass(poly, course, Some(entries), Edge::Depth(NEAREST), true, 0)?;
        let far =
            far_clip && view.face.no_far_clip.get() == 0 && scene.draw_switches.far_on.get() != 0;
        if far && (poly.polygon_vertex_count.get() as i32).wrapping_sub(3) >= 0 {
            let farthest = view.polygon.window.clip_farthest.get() as i32;
            poly.clip_plane.set(farthest as u32);
            clip_pass(poly, course, listed, Edge::Depth(farthest), false, 0)?;
        }
        Ok(())
    })
}
