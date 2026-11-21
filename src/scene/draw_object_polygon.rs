//! `scene::draw_object_polygon`: draws a polygon of an object, as it is if it lies on the
//! screen and as triangles if it has to be clipped. The polygon's entries are the work table's
//! first, in order.
//!
//! The pool of records, the list (0, 1, 2 .. and a 0) and the next free entry are set as
//! `scene::draw_clipped_polygon` sets them; `scene::project_polygon`; then
//! `scene::needs_clipping` says whether a polygon of three vertices or more has one that is
//! nearer than 4096.0, or left of or above that same number, or at or past the screen's right
//! or bottom edge. If none, or with exactly three vertices: the whole-polygon counter + 1 and
//! `scene::draw_clipped_polygon`. Else the fan counter + 1, the entries are kept, and the
//! polygon is drawn as a fan of triangles: the first three entries as they are, then for
//! n = 3 .. the entries 0, n - 1 and n from the kept ones, each with the number of vertices at
//! 3, where it stays. Three call sites, all in `scene::draw_object_face`. Refuses a polygon of
//! no vertices or of more than 64 (the kept entries' room).
//!
//! The logic is in the core crate: the test, the list, the choice and the fan through
//! `rc2k_core::scene::polygons` (compares of stored floats: exact).

use crate::errors::{BadInput, Refused};

// ---- game data ----

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Project,
    Draw,
}

/// `scene::needs_clipping` on the work table's records.
fn needs_clipping(
    part: &crate::scene::scratch_store::PartScratch,
    poly: &crate::scene::polygon_work_store::PolygonWork,
    course: crate::race::owner_course::CourseReads<'_>,
    count: u32,
) -> Result<bool, BadInput> {
    let polygon_work = &part.polygon;
    use rc2k_core::scene::polygons::{Edges, Vertex};
    if (count as i32).wrapping_sub(3) < 0 {
        return Ok(false);
    }
    let edges = Edges {
        least: course.numbers.zero.get(),
        right: poly.window.borrow().clip_screen[3],
        bottom: poly.window.borrow().clip_screen[1],
    };
    rc2k_core::scene::polygons::needs_clipping(count, &edges, |n| {
        let record = crate::scene::polygon::entry(polygon_work, n)?.record.get();
        if record == 0 {
            return Err(BadInput("an entry without its record"));
        }
        let value = |at: u32| {
            f32::from_bits(crate::scene::point_records::vertex(
                part,
                &poly.quad_vertices,
                record,
                at,
            ))
        };
        Ok(Vertex {
            x: value(0xC),
            y: value(0x1C),
            depth: value(0x18),
        })
    })
}

// Logic: `rc2k_core::scene::polygons`
pub fn draw_object_polygon(
    part: &crate::scene::scratch_store::PartScratch,
    poly: &crate::scene::polygon_work_store::PolygonWork,
    course: crate::race::owner_course::CourseReads<'_>,
    step: &mut dyn FnMut(Step) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let polygon_work = &part.polygon;
    use rc2k_core::scene::polygons::{drawn_whole, fan, first_list, fits};
    let count: u32 = poly.polygon_vertex_count.get();
    if !fits(count) {
        return Err(BadInput("a polygon of no vertices, or of more than 64").into());
    }
    poly.polygon_next_record
        .set(crate::scene::polygon::pool_start());
    for (n, number) in first_list(count).enumerate() {
        crate::scene::polygon::list_cell(polygon_work, n).set(number);
    }
    poly.polygon_next_entry.set(count);
    step(Step::Project)?;
    if drawn_whole(count, needs_clipping(part, poly, course, count)?) {
        return step(Step::Draw);
    }
    // the first `count` entries kept from entry 64 (`fits`: at most 64)
    let (entries, kept) = polygon_work.entries.split_at(64);
    for (to, from) in kept.iter().zip(&entries[..count as usize]) {
        to.copy_from(from);
    }
    poly.polygon_vertex_count.set(3u32);
    step(Step::Draw)?;
    poly.polygon_vertex_count.set(count);
    for triangle in fan(count) {
        for (to, from) in triangle.into_iter().enumerate() {
            entries[to].copy_from(&kept[from]);
        }
        poly.polygon_vertex_count.set(3u32);
        step(Step::Draw)?;
    }
    Ok(())
}
