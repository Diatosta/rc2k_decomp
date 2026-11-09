//! `race::draw_view_overlays` (0x4559A6): what a view draws over its scene, unless
//! `unk_71AF80` is set (as `race::draw_view_scene`): the HUD's overlay states, render
//! state 3 = 3, the view's marks (`race::run_view_marks`), the overlay states again, the
//! race's sound tick (eax 0xE in the original), a flat triangle
//! (`race::draw_flat_triangle`), the depth buffer on, the filters; the own car drawn
//! (`race::draw_own_car`) with `dword_621900` 0 meanwhile (put back after), the screen's
//! tint, the draw batch flushed, the filters.

use crate::errors::Refused;

/// The Rust version (see the header).
pub fn draw_view_overlays(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    system: &crate::system::owner::System,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    let draw_switches = &scene.draw_switches;
    use crate::hud::overlay_states::overlay_states;
    use crate::race::frame_views::{draw_flat_triangle, draw_own_car, draw_screen_tint};
    use crate::scene::set_filters::set_filters;
    if crate::race::frame::drawn_car().view_covered.get() != 0 {
        return Ok(());
    }
    overlay_states();
    crate::scene::set_render_state::set_render_state(3, 3);
    crate::race::frame_views::run_view_marks(race, scene, system, effects, &ctx.view.fog)?;
    overlay_states();
    draw_flat_triangle(scene);
    crate::scene::depth_buffer_on::depth_buffer(true);
    set_filters();
    let kept: u32 = draw_switches.far_on.bits();
    draw_switches.far_on.set_bits(0u32);
    let drawn = draw_own_car(ctx, system, effects);
    draw_switches.far_on.set_bits(kept);
    drawn?;
    draw_screen_tint(race, scene, &ctx.view.fog)?;
    crate::race::flush_draw_batch::flush_draw_batch(race)?;
    set_filters();
    Ok(())
}
