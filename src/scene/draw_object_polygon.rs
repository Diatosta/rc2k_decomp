//! `scene::draw_object_polygon` (0x451F69): draws a polygon of an object, as it is if
//! it lies on the screen and as triangles if it has to be clipped.
//!
//! In:  nothing; the polygon's entries are the work table's first, in order
//! Out: the original leaves every register changed; the Rust version keeps them
//!
//! The pool of records, the list (0, 1, 2 .. and a 0) and the next free entry are set as
//! `scene::draw_clipped_polygon` sets them; `scene::project_polygon`; then `sub_451EED` says
//! whether a polygon of three vertices or more has one that is nearer than `flt_5C5D90`, or
//! left of or above that same number, or at or past the screen's right or bottom edge. If none,
//! or with exactly three vertices: `dword_639960` + 1 and `scene::draw_clipped_polygon`. Else
//! `dword_639964` + 1, the entries are kept at `unk_976610`, and the polygon is drawn as a fan
//! of triangles: the first three entries as they are, then for n = 3 .. the entries 0, n - 1
//! and n from the kept ones (`sub_45204A` copies one), each with the number of vertices at 3,
//! where it stays. Three call sites, all in `sub_450BDA`. Refuses a polygon of no vertices or
//! of more than 64 (the kept entries' room).
//!
//! Converted 2026-10-21: the test, the list, the choice and the fan through
//! `rc2k_core::scene::polygons` (compares of stored floats: exact).

use crate::errors::{BadInput, Refused};
use crate::scene::draw_store::DRAW;

// ---- game data ----

// ---- Rust version ----

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Project,
    Draw,
}

/// `sub_451EED` on the work table's records.
fn needs_clipping(
    course: &crate::race::owner_course::Course,
    count: u32,
) -> Result<bool, BadInput> {
    use rc2k_core::scene::polygons::{Edges, Vertex};
    if (count as i32).wrapping_sub(3) < 0 {
        return Ok(false);
    }
    let edges = Edges {
        least: course.numbers.zero.get(),
        right: DRAW.clip_screen[3].get(),
        bottom: DRAW.clip_screen[1].get(),
    };
    rc2k_core::scene::polygons::needs_clipping(count, &edges, |n| {
        let record = crate::scene::polygon::entry(n)?.record.get();
        if record == 0 {
            return Err(BadInput("an entry without its record"));
        }
        let value = |at: u32| f32::from_bits(crate::scene::point_records::vertex(record, at));
        Ok(Vertex {
            x: value(0xC),
            y: value(0x1C),
            depth: value(0x18),
        })
    })
}

// Converted: `rc2k_core::scene::polygons`
pub fn draw_object_polygon(
    race: &crate::race::owner::Race,
    step: &mut dyn FnMut(Step) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let physics = &race.physics;
    use rc2k_core::scene::polygons::{drawn_whole, fan, first_list, fits};
    let count: u32 = DRAW.polygon_vertex_count.bits();
    if !fits(count) {
        return Err(BadInput("a polygon of no vertices, or of more than 64").into());
    }
    DRAW.polygon_next_record
        .set_bits(crate::scene::polygon::pool_start());
    for (n, number) in first_list(count).into_iter().enumerate() {
        crate::scene::polygon::list_cell(n).set(number);
    }
    DRAW.polygon_next_entry.set_bits(count);
    step(Step::Project)?;
    if drawn_whole(count, needs_clipping(&race.course, count)?) {
        physics
            .ground
            .whole_polygon_count
            .set_bits(physics.ground.whole_polygon_count.bits().wrapping_add(1));
        return step(Step::Draw);
    }
    physics
        .ground
        .split_polygon_count
        .set_bits(physics.ground.split_polygon_count.bits().wrapping_add(1));
    // the first `count` entries kept from entry 64 (`fits`: at most 64)
    let (entries, kept) = crate::scene::polygon_store::POLYGON_WORK
        .entries
        .split_at(64);
    for (to, from) in kept.iter().zip(&entries[..count as usize]) {
        to.copy_from(from);
    }
    DRAW.polygon_vertex_count.set_bits(3u32);
    step(Step::Draw)?;
    DRAW.polygon_vertex_count.set_bits(count);
    for triangle in fan(count) {
        for (to, from) in triangle.into_iter().enumerate() {
            entries[to].copy_from(&kept[from]);
        }
        DRAW.polygon_vertex_count.set_bits(3u32);
        step(Step::Draw)?;
    }
    Ok(())
}
