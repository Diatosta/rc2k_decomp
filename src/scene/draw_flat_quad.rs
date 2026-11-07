//! `scene::draw_flat_quad` (0x44AF85): a flat quad on the screen through the polygon
//! pipeline: `g_polygonDrawn` 0; the four polygon records' vertex pointers
//! (`dword_623E40` .. 0x20 apart), texture places (`dword_623DF0` .. u, `dword_623E00` ..
//! v), the corners' x (`dword_623E10` left, `dword_623E14` right) and y (`dword_623E20`
//! top, `dword_623E24` bottom) as floats (through `dword_62400C`), depth `dword_5C5E20`;
//! the entries -1, colour `dword_623DE8`'s low 24 bits, the records' +0x18 0; four
//! vertices, the record pool, the list 0 .. 3; then `scene::scale_texture_coordinates`,
//! `scene::clip_polygon_to_screen` and, with three vertices or more left, mode 6
//! (`g_polygonMode`), `dword_6240E0` 1, `dword_6240E4` 0, `scene::depth_buffer_off`,
//! `scene::draw_polygon`, `g_polygonDrawn` 1, `scene::depth_buffer_on`; the colour back to
//! white.
//!
//! In:  nothing (`dword_623DE8` .. `dword_623E24`, set by the caller)
//! Out: eax 0, ecx 4, esi the record pool going into the calls; then as they leave them
//!
//! Callers: `sub_45B86E` (three), and code at 0x4436B2, 0x4437C1, 0x45BCD8, 0x45BDD0. The
//! calls (the scale, the clip, the depth buffer off, the draw, the depth buffer on) are
//! their Rust versions; a refusal of theirs comes after the writes.

use crate::errors::Refused;
use crate::math::x87;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::scene::owner::scene;
use crate::session::Dword;

/// The Rust version (see the header). It reads no register, and what it leaves is read by
/// none of its callers (register liveness, 2026-10-23).
pub fn draw_flat_quad(race: &Race, course: &Course) -> Result<(), Refused> {
    let shared = &race.shared;
    let drawing = &scene().drawing;
    drawing.scene.polygon_drawn.set_bits(0);
    for n in 0..4 {
        let rec = &crate::scene::polygon_store::POLYGON_WORK.entries[n];
        rec.record
            .set(crate::scene::point_records::of_quad(n as u32));
        rec.u.set(shared.vars.quad_uv[n].bits());
        rec.v.set(shared.vars.quad_uv[4 + n].bits());
    }
    let float = |cell: &Dword| {
        let v = x87::from_int(cell.bits() as i32);
        drawing.scene.scratch.set_bits(v);
        v
    };
    let [left, right, top, bottom] = [
        &shared.vars.quad_corners[0],
        &shared.vars.quad_corners[1],
        &shared.vars.quad_corners[4],
        &shared.vars.quad_corners[5],
    ]
    .map(float);
    let v = |n: usize, o: usize, x: u32| {
        shared.vars.quad_vertices[8 * n + o / 4].set_bits(x);
    };
    for (n, x, y) in [
        (0, left, top),
        (1, right, top),
        (2, right, bottom),
        (3, left, bottom),
    ] {
        v(n, 0xC, x);
        v(n, 0x1C, y);
    }
    let depth: u32 = course.numbers.k4096_b.bits();
    for n in 0..4 {
        v(n, 0x18, depth);
    }
    for n in 0..4 {
        crate::scene::polygon_store::POLYGON_WORK.entries[n]
            .point
            .set(0xFFFF_FFFF);
    }
    drawing
        .scene
        .polygon_colour
        .set_bits(shared.vars.quad_colour[0].bits() & 0xFF_FFFF);
    for n in 0..4 {
        crate::scene::polygon_store::POLYGON_WORK.entries[n]
            .light
            .set(0);
    }
    drawing.scene.polygon_vertex_count.set_bits(4);
    drawing
        .scene
        .polygon_next_record
        .set_bits(crate::scene::polygon::pool_start());
    for n in 0..4u32 {
        crate::scene::polygon::list_cell(n as usize).set(n);
    }
    crate::scene::polygon::list_cell(4).set(0);
    drawing.scene.polygon_next_entry.set_bits(4);
    crate::scene::scale_texture_coordinates::scale_texture_coordinates()?;
    crate::scene::clip_polygon_to_screen::clip_polygon_to_screen(race)?;
    if drawing.scene.polygon_vertex_count.bits() >= 3 {
        shared.view.polygon_mode.set_bits(6);
        drawing.scene.polygon_sides[0].set_bits(1);
        drawing.scene.polygon_sides[1].set_bits(0);
        crate::scene::depth_buffer_on::depth_buffer(false);
        crate::scene::draw_polygon::draw_polygon_in_game(race)?;
        drawing.scene.polygon_drawn.set_bits(1);
        crate::scene::depth_buffer_on::depth_buffer(true);
    }
    drawing.scene.polygon_colour.set_bits(0xFF_FFFF);
    Ok(())
}
