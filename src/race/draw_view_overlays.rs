//! `race::draw_view_overlays` (0x4559A6): what a view draws over its scene, unless
//! `unk_71AF80` is set (as `race::draw_view_scene`): the HUD's overlay states, render
//! state 3 = 3, the view's marks (`race::run_view_marks`), the overlay states again, the
//! race's sound tick (eax 0xE in the original), a flat triangle
//! (`race::draw_flat_triangle`), the depth buffer on, the filters; the own car drawn
//! (`race::draw_own_car`) with `dword_621900` 0 meanwhile (put back after), the screen's
//! tint, the draw batch flushed, the filters.
//!
//! In:  nothing (no step reads a register its caller gives; edx's "read" in the liveness is
//!      the marks' `imul`)
//! Out: nothing its one caller, `race::draw_race_scene`, reads

use crate::errors::Refused;
use crate::physics::car_store::CAR;
use crate::scene::draw_switches_store::DRAW_SWITCHES;

/// The Rust version (see the header).
pub unsafe fn draw_view_overlays() -> Result<(), Refused> {
    use crate::hud::overlay_states::overlay_states;
    use crate::race::frame_views::{draw_flat_triangle, draw_own_car, draw_screen_tint};
    use crate::scene::set_filters::set_filters;
    unsafe {
        if CAR.view_covered.get() != 0 {
            return Ok(());
        }
        overlay_states();
        crate::scene::set_render_state::set_render_state(3, 3);
        crate::race::frame_views::run_view_marks()?;
        overlay_states();
        crate::sound::race_sound_tick::race_sound_tick()?;
        draw_flat_triangle();
        crate::scene::depth_buffer_on::depth_buffer(true);
        set_filters();
        let kept: u32 = DRAW_SWITCHES.far_on.bits();
        DRAW_SWITCHES.far_on.set_bits(0u32);
        let drawn = draw_own_car();
        DRAW_SWITCHES.far_on.set_bits(kept);
        drawn?;
        draw_screen_tint()?;
        crate::race::flush_draw_batch::flush_draw_batch()?;
        set_filters();
    }
    Ok(())
}
