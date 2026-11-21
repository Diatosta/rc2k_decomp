//! `scene::draw_tinted_quad`: a flat quad on the screen in `RaceVars.quad_colour[0]`'s colour,
//! see-through by `RaceVars.quad_colour[1]`'s low byte, through the polygon pipeline (used for the
//! HUD's backdrops, the service area's screen, the menus): as `scene::draw_flat_quad` but the
//! texture places 0, the colour whole, each entry's `light` 0xFFFFFF, `lamp_light` 0, `fog` 0xFF
//! and `amount` the see-through byte; no texture scaling; with three vertices or more after the
//! clip: `scene::depth_buffer_on::depth_buffer` off, mode 3, `SceneDraw.polygon_textures[0]` /
//! `[1]` -1, `SceneDraw.polygon_sides[0]` 0, `[1]` 1, `scene::draw_polygon`, mode 0, the
//! polygon-drawn flag 1, `scene::depth_buffer_on`; the colour back to white. The quad's cells
//! (`RaceVars.quad_colour[0]` .. `RaceVars.quad_corners[5]`) are set by the caller. The clip, the
//! depth buffer calls and the draw are `draw_flat_quad`'s.

use crate::errors::Refused;
use crate::math::x87;

pub fn draw_tinted_quad(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), Refused> {
    for (n, rec) in ctx.part.polygon.entries.iter().enumerate().take(4) {
        rec.record
            .set(crate::scene::point_records::of_quad(n as u32));
        rec.u.set(0);
        rec.v.set(0);
    }
    let float = |value: u32| x87::from_int(value as i32);
    let [left, right, top, bottom] = {
        let corners = ctx.view.quad.borrow().corners;
        [corners[0], corners[1], corners[4], corners[5]].map(float)
    };
    let v = |n: usize, o: usize, x: f32| {
        ctx.view.polygon.quads.borrow_mut().words[8 * n + o / 4] = x.to_bits();
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
    let depth = ctx.reads.numbers.k4096_b.get();
    for n in 0..4 {
        v(n, 0x18, depth);
    }
    let see = ctx.view.quad.borrow().colour[1] as u8;
    ctx.view.polygon.scalars.borrow_mut().polygon_colour = ctx.view.quad.borrow().colour[0];
    for item in ctx.part.polygon.entries.iter().take(4) {
        let rec = &item;
        rec.point.set(0xFFFF_FFFF);
        rec.amount.set(see);
        rec.light.set(0xFF_FFFF);
        rec.lamp_light.set(0);
        rec.fog.set(0xFF);
    }
    ctx.view.polygon.scalars.borrow_mut().polygon_vertex_count = 4;
    ctx.view.polygon.scalars.borrow_mut().polygon_next_record = crate::scene::polygon::pool_start();
    for n in 0..4u32 {
        crate::scene::polygon::list_cell(&ctx.part.polygon, n as usize).set(n);
    }
    crate::scene::polygon::list_cell(&ctx.part.polygon, 4).set(0);
    ctx.view.polygon.scalars.borrow_mut().polygon_next_entry = 4;
    crate::scene::clip_polygon_to_screen::clip_polygon_to_screen(
        ctx.part,
        &ctx.view.polygon,
        ctx.reads.course(),
    )?;
    if ctx.view.polygon.scalars.borrow().polygon_vertex_count >= 3 {
        crate::scene::depth_buffer_on::depth_buffer(ctx.part, false);
        ctx.view.face.polygon_mode.set(3);
        ctx.view.polygon.scalars.borrow_mut().polygon_textures[0] = 0xFFFF_FFFF;
        ctx.view.polygon.scalars.borrow_mut().polygon_textures[1] = 0xFFFF_FFFF;
        ctx.view.polygon.scalars.borrow_mut().polygon_sides[0] = 0;
        ctx.view.polygon.scalars.borrow_mut().polygon_sides[1] = 1;
        crate::scene::draw_polygon::draw_polygon_in_game(
            ctx.view,
            ctx.part,
            ctx.reads,
            ctx.reads.screen,
        )?;
        ctx.view.face.polygon_mode.set(0);
        crate::scene::depth_buffer_on::depth_buffer(ctx.part, true);
    }
    ctx.view.polygon.scalars.borrow_mut().polygon_colour = 0xFF_FFFF;
    Ok(())
}
