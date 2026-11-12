//! `scene::draw_tinted_quad` (0x44B538): a flat quad on the screen in `RaceVars.quad_colour[0]`'s
//! colour, see-through by `RaceVars.quad_colour[1]`'s low byte, through the polygon pipeline (858
//! callers: the HUD's backdrops, the service area's screen, the menus): as
//! `scene::draw_flat_quad` but the texture places 0, the colour whole, each record's +0x18
//! 0xFFFFFF, +0x1C 0, +0x20 0xFF and +0x21 the see-through byte; no texture scaling; with
//! three vertices or more after the clip: `scene::depth_buffer_off`, mode 3,
//! `SceneDraw.polygon_textures[0]` / `SceneDraw.polygon_textures[1]` -1, `SceneDraw.polygon_sides[0]` 0, `SceneDraw.polygon_sides[1]` 1,
//! `scene::draw_polygon`, mode 0, `g_polygonDrawn` 1, `scene::depth_buffer_on`; the
//! colour back to white.
//!
//! In:  nothing (`RaceVars.quad_colour[0]` .. `RaceVars.quad_corners[5]`, set by the caller)
//!
//! The calls as `draw_flat_quad`'s (the clip, the depth buffer, the draw), their Rust
//! versions.

use crate::errors::Refused;
use crate::math::x87;
use crate::session::Dword;

pub fn draw_tinted_quad(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    let shared = &race.shared;
    ctx.view.polygon.polygon_drawn.set(0);
    for n in 0..4 {
        let rec = &ctx.part.polygon.entries[n];
        rec.record
            .set(crate::scene::point_records::of_quad(n as u32));
        rec.u.set(0);
        rec.v.set(0);
    }
    let float = |cell: &Dword| x87::from_int(cell.get() as i32);
    let [left, right, top, bottom] = [
        &ctx.view.quad.corners[0],
        &ctx.view.quad.corners[1],
        &ctx.view.quad.corners[4],
        &ctx.view.quad.corners[5],
    ]
    .map(float);
    let v = |n: usize, o: usize, x: f32| {
        ctx.view.polygon.quad_vertices[8 * n + o / 4].set(x.to_bits());
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
    let depth = race.course.numbers.k4096_b.get();
    for n in 0..4 {
        v(n, 0x18, depth);
    }
    let see = shared.vars.quad_colour[1].get() as u8;
    ctx.view
        .polygon
        .polygon_colour
        .set(shared.vars.quad_colour[0].get());
    for n in 0..4 {
        let rec = &ctx.part.polygon.entries[n];
        rec.point.set(0xFFFF_FFFF);
        rec.amount.set(see);
        rec.light.set(0xFF_FFFF);
        rec.lamp_light.set(0);
        rec.fog.set(0xFF);
    }
    ctx.view.polygon.polygon_vertex_count.set(4);
    ctx.view
        .polygon
        .polygon_next_record
        .set(crate::scene::polygon::pool_start());
    for n in 0..4u32 {
        crate::scene::polygon::list_cell(ctx.part.polygon, n as usize).set(n);
    }
    crate::scene::polygon::list_cell(ctx.part.polygon, 4).set(0);
    ctx.view.polygon.polygon_next_entry.set(4);
    crate::scene::clip_polygon_to_screen::clip_polygon_to_screen(
        ctx.part,
        &ctx.view.polygon,
        race,
    )?;
    if ctx.view.polygon.polygon_vertex_count.get() >= 3 {
        crate::scene::depth_buffer_on::depth_buffer(ctx.part, false);
        ctx.view.face.polygon_mode.set(3);
        ctx.view.polygon.polygon_textures[0].set(0xFFFF_FFFF);
        ctx.view.polygon.polygon_textures[1].set(0xFFFF_FFFF);
        ctx.view.polygon.polygon_sides[0].set(0);
        ctx.view.polygon.polygon_sides[1].set(1);
        crate::scene::draw_polygon::draw_polygon_in_game(
            &ctx.view.polygon,
            ctx.part,
            race,
            &ctx.view.face,
            scene,
            ctx.reads,
            &ctx.system.session.screen,
        )?;
        ctx.view.face.polygon_mode.set(0);
        ctx.view.polygon.polygon_drawn.set(1);
        crate::scene::depth_buffer_on::depth_buffer(ctx.part, true);
    }
    ctx.view.polygon.polygon_colour.set(0xFF_FFFF);
    Ok(())
}
