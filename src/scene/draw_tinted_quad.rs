//! `scene::draw_tinted_quad` (0x44B538): a flat quad on the screen in `dword_623DE8`'s
//! colour, see-through by `dword_623DEC`'s low byte, through the polygon pipeline (858
//! callers: the HUD's backdrops, the service area's screen, the menus): as
//! `scene::draw_flat_quad` but the texture places 0, the colour whole, each record's +0x18
//! 0xFFFFFF, +0x1C 0, +0x20 0xFF and +0x21 the see-through byte; no texture scaling; with
//! three vertices or more after the clip: `scene::depth_buffer_off`, mode 3,
//! `dword_6240BC` / `dword_6240C0` -1, `dword_6240E0` 0, `dword_6240E4` 1,
//! `scene::draw_polygon`, mode 0, `g_polygonDrawn` 1, `scene::depth_buffer_on`; the
//! colour back to white.
//!
//! In:  nothing (`dword_623DE8` .. `dword_623E24`, set by the caller)
//!
//! The calls as `draw_flat_quad`'s (the clip, the depth buffer, the draw), their Rust
//! versions.

use crate::errors::Refused;
use crate::math::x87;
use crate::session::Dword;

/// The Rust version (see the header). It reads no register, and what it leaves is read by
/// none of its 856 call sites (register liveness, 2026-10-23).
pub fn draw_tinted_quad(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    let shared = &race.shared;
    let drawing = &scene.drawing;
    drawing.scene.polygon_drawn.set_bits(0);
    for n in 0..4 {
        let rec = &crate::scene::polygon_store::POLYGON_WORK.entries[n];
        rec.record
            .set(crate::scene::point_records::of_quad(n as u32));
        rec.u.set(0);
        rec.v.set(0);
    }
    let float = |cell: &Dword| x87::from_int(cell.bits() as i32);
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
    let depth: u32 = race.course.numbers.k4096_b.bits();
    for n in 0..4 {
        v(n, 0x18, depth);
    }
    let see = shared.vars.quad_colour[1].bits() as u8;
    drawing
        .scene
        .polygon_colour
        .set_bits(shared.vars.quad_colour[0].bits());
    for n in 0..4 {
        let rec = &crate::scene::polygon_store::POLYGON_WORK.entries[n];
        rec.point.set(0xFFFF_FFFF);
        rec.amount.set(see);
        rec.light.set(0xFF_FFFF);
        rec.lamp_light.set(0);
        rec.fog.set(0xFF);
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
    crate::scene::clip_polygon_to_screen::clip_polygon_to_screen(race, scene)?;
    if drawing.scene.polygon_vertex_count.bits() >= 3 {
        crate::scene::depth_buffer_on::depth_buffer(false);
        shared.view.polygon_mode.set_bits(3);
        drawing.scene.polygon_textures[0].set_bits(0xFFFF_FFFF);
        drawing.scene.polygon_textures[1].set_bits(0xFFFF_FFFF);
        drawing.scene.polygon_sides[0].set_bits(0);
        drawing.scene.polygon_sides[1].set_bits(1);
        crate::scene::draw_polygon::draw_polygon_in_game(race, scene)?;
        shared.view.polygon_mode.set_bits(0);
        drawing.scene.polygon_drawn.set_bits(1);
        crate::scene::depth_buffer_on::depth_buffer(true);
    }
    drawing.scene.polygon_colour.set_bits(0xFF_FFFF);
    Ok(())
}
