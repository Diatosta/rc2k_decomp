//! `scene::clip_polygon_to_screen`: cuts the polygon to the screen's four edges.
//!
//! `SceneDraw.screen_clipped` = 1. Each vertex's place on the screen (its record's screen x and
//! y) against the edges `SceneDraw.clip_screen[2]` (left), `[3]` (right), `[0]` (top) and
//! `[1]` (bottom) gives the bits 1, 2, 4, 8 of a mask: right is looked at first and left only
//! if the vertex is not past it, bottom first and top only if not past that. A mask of 0:
//! nothing more. Else the mask goes to `SceneDraw.polygon_clip_mask`, and for each of its bits
//! in that order one pass of `scene/clip.rs`, ending as soon as fewer than three vertices are
//! left. 14 callers. Refuses a polygon of no vertices, an entry without its record, and a
//! clipping that runs out of entries (what it had written is put back first).

use crate::errors::BadInput;
use crate::memory::session::PlainFloat;
use crate::scene::clip::{Edge, below, clip_pass, listed_with_records, or_undone};

// ---- game data ----

/// The polygon's entries and the mask of the edges some vertex is past.
fn outside_mask<'p>(
    part: &'p crate::scene::scratch_store::PartScratch,
    poly: &crate::scene::polygon_work_store::PolygonWork,
) -> Result<(crate::scene::polygon::Listed<'p>, u32), BadInput> {
    let polygon_work = &part.polygon;
    let entries = listed_with_records(polygon_work, poly)?;
    let edge = |cell: &PlainFloat| cell.get() as f64;
    let (left, right, top, bottom) = (
        edge(&poly.window.clip_screen[2]),
        edge(&poly.window.clip_screen[3]),
        edge(&poly.window.clip_screen[0]),
        edge(&poly.window.clip_screen[1]),
    );
    let mut mask = 0;
    for entry in &entries {
        let record = entry.record.get();
        let x = f32::from_bits(crate::scene::point_records::vertex(
            part,
            &poly.quad_vertices,
            record,
            0xC,
        )) as f64;
        let y = f32::from_bits(crate::scene::point_records::vertex(
            part,
            &poly.quad_vertices,
            record,
            0x1C,
        )) as f64;
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
    Ok((entries, mask))
}

pub fn clip_polygon_to_screen(
    part: &crate::scene::scratch_store::PartScratch,
    poly: &crate::scene::polygon_work_store::PolygonWork,
    course: crate::race::owner_course::CourseReads<'_>,
) -> Result<(), BadInput> {
    let (entries, mask) = outside_mask(part, poly)?;
    poly.screen_clipped.set(1u32);
    if mask == 0 {
        return Ok(());
    }
    poly.polygon_clip_mask.set(mask);
    or_undone(|| {
        let mut listed = Some(entries);
        for (bit, edge, inside_past) in [
            (1, Edge::X(poly.window.clip_screen[2].get()), true),
            (2, Edge::X(poly.window.clip_screen[3].get()), false),
            (4, Edge::Y(poly.window.clip_screen[0].get()), true),
            (8, Edge::Y(poly.window.clip_screen[1].get()), false),
        ] {
            if mask & bit != 0 {
                listed = clip_pass(part, poly, course, listed, edge, inside_past)?;
                if poly.polygon_vertex_count.get() < 3 {
                    break;
                }
            }
        }
        Ok(())
    })
}
