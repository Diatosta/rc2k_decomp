//! `race::draw_view_overlays` (0x4559A6): what a view draws over its scene, unless `unk_71AF80` is
//! set (as `race::draw_view_scene`): the HUD's overlay states, render state 3 = 3, the view's marks
//! (`race::run_view_marks`), the overlay states again, the race's sound tick, a flat triangle
//! (`race::draw_flat_triangle`), the depth buffer on, the filters; the own car drawn
//! (`race::draw_own_car`) without the far clip (the original set `dword_621900` 0 meanwhile and put
//! it back after: `DrawCtx::far_clip`), the screen's tint, the draw batch flushed, the filters.

use crate::errors::Refused;

pub fn draw_view_overlays(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    system: &crate::system::owner::System,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let race = ctx.race;
    use crate::hud::overlay_states::overlay_states;
    use crate::race::frame_views::{draw_flat_triangle, draw_own_car, draw_screen_tint};
    use crate::scene::set_filters::set_filters;
    if ctx.view.car.view_covered.get() != 0 {
        return Ok(());
    }
    overlay_states();
    crate::scene::set_render_state::set_render_state(3, 3);
    let begun = crate::debug::timing::start();
    crate::race::frame_views::run_view_marks(ctx, system, effects)?;
    crate::debug::timing::stop(crate::debug::timing::Stage::Effects, begun);
    overlay_states();
    draw_flat_triangle();
    crate::scene::depth_buffer_on::depth_buffer(true);
    set_filters();
    {
        // Not around the view's marks: they run `drive::set_listener`, on the shared store.
        let _held = ctx.view.matrices.hold(&race.frame.matrices);
        draw_own_car(
            &crate::race::view_scratch::DrawCtx {
                far_clip: false,
                ..*ctx
            },
            system,
            effects,
        )?;
    }
    draw_screen_tint(ctx, &ctx.view.fog)?;
    crate::race::flush_draw_batch::flush_draw_batch(race)?;
    set_filters();
    Ok(())
}
