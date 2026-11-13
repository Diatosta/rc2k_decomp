//! `race::draw_view_overlays` (0x4559A6): what a view draws over its scene, unless `view_covered` (`view_covered`, the player is out of a two-player race) is
//! set (as `race::draw_view_scene`): the HUD's overlay states, render state 3 = 3, the view's marks
//! (`race::run_view_marks`), the overlay states again, the race's sound tick, a flat triangle
//! (`race::draw_flat_triangle`), the depth buffer on, the filters; the own car drawn
//! (`race::draw_own_car`) without the far clip (the original set `far_on` 0 meanwhile and put
//! it back after: `DrawCtx::far_clip`), the screen's tint, the draw batch flushed, the filters.

use crate::errors::Refused;

pub fn draw_view_overlays(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    system: &crate::system::owner::System,
    sound: &'static crate::sound::owner::Sound,
    gfx: &crate::gfx::owner::Gfx,
    cheats: &crate::menu::cheats_store::Cheats,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let race = ctx.race;
    use crate::hud::overlay_states::overlay_states;
    use crate::race::frame_views::{draw_flat_triangle, draw_own_car, draw_screen_tint};
    use crate::scene::set_filters::set_filters;
    if ctx.view.car.view_covered.get() != 0 {
        return Ok(());
    }
    overlay_states(ctx.part);
    crate::scene::set_render_state::set_render_state(ctx.part, 3, 3);
    let begun = crate::debug::timing::start();
    crate::race::frame_views::run_view_marks(ctx, system, sound, gfx, effects)?;
    crate::debug::timing::stop(crate::debug::timing::Stage::Effects, begun);
    overlay_states(ctx.part);
    draw_flat_triangle(ctx.part, ctx.scene);
    crate::scene::depth_buffer_on::depth_buffer(ctx.part, true);
    set_filters(race, ctx.part);
    {
        // Not around the view's marks: they run `drive::set_listener`, on the shared store.
        let _held = ctx.view.matrices.hold(ctx.reads.matrices, ctx.part);
        draw_own_car(
            &crate::race::view_scratch::DrawCtx {
                far_clip: false,
                ..*ctx
            },
            system,
            gfx,
            cheats,
            effects,
        )?;
    }
    draw_screen_tint(ctx, &ctx.view.fog)?;
    set_filters(race, ctx.part);
    Ok(())
}
