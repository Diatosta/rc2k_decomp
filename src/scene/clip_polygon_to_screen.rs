//! `scene::clip_polygon_to_screen` (0x44D15A): cuts the polygon to the screen's four
//! edges.
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
use crate::race::owner::Race;
use crate::scene::clip::{Edge, below, clip_pass, listed_with_records, or_undone};
use crate::session::Float;

// ---- game data ----

// ---- Rust version ----

/// The polygon's entries and the mask of the edges some vertex is past.
fn outside_mask(
    poly: &crate::scene::polygon_work::PolygonWork,
    scene: &crate::scene::owner::Scene,
) -> Result<(crate::scene::polygon::Listed, u32), BadInput> {
    let drawing = &scene.drawing;
    let entries = listed_with_records(poly)?;
    let edge = |cell: &Float| cell.get() as f64;
    let (left, right, top, bottom) = (
        edge(&drawing.scene.clip_screen[2]),
        edge(&drawing.scene.clip_screen[3]),
        edge(&drawing.scene.clip_screen[0]),
        edge(&drawing.scene.clip_screen[1]),
    );
    let mut mask = 0;
    for entry in &entries {
        let record = entry.record.get();
        let x = f32::from_bits(crate::scene::point_records::vertex(
            &poly.quad_vertices,
            record,
            0xC,
        )) as f64;
        let y = f32::from_bits(crate::scene::point_records::vertex(
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
    poly: &crate::scene::polygon_work::PolygonWork,
    race: &Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    let drawing = &scene.drawing;
    let course = &race.course;
    let (entries, mask) = outside_mask(poly, scene)?;
    poly.screen_clipped.set_bits(1u32);
    if mask == 0 {
        return Ok(());
    }
    poly.polygon_clip_mask.set_bits(mask);
    or_undone(|| {
        let mut listed = Some(entries);
        for (bit, edge, inside_past) in [
            (1, Edge::X(drawing.scene.clip_screen[2].bits()), true),
            (2, Edge::X(drawing.scene.clip_screen[3].bits()), false),
            (4, Edge::Y(drawing.scene.clip_screen[0].bits()), true),
            (8, Edge::Y(drawing.scene.clip_screen[1].bits()), false),
        ] {
            if mask & bit != 0 {
                listed = clip_pass(poly, course, listed, edge, inside_past, mask)?;
                if poly.polygon_vertex_count.bits() < 3 {
                    break;
                }
            }
        }
        Ok(())
    })
}
