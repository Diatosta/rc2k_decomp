//! Unless `hold_car_revs`, for each player's view block (the second's when `split` is not 0)
//! whose +0xFA8 is 0 and +0x728 is set: while its flash is on (+0x730 at most 0, or +0x72C
//! modulo +0x730 below half of +0x730), `hud::draw_view_strip` for view `split` (the second's
//! plus 3) with texture `polygon_textures` 0xC4 (0xC5); then the strips' shared tail.

use crate::errors::{BadInput, Refused};
use crate::hud::draw_view_strip::Overlay;
use crate::hud::overlay_pass::shown_layout;

/// Whether the strip of the view block `handle` names is shown (see the header).
fn flashing(shared: &crate::race::owner_shared::RaceShared, handle: u32) -> Result<bool, BadInput> {
    let car = shared
        .car_blocks()
        .of(handle)
        .ok_or(BadInput("a view's block"))?;
    Ok(rc2k_core::hud::strips::strip(
        car.view_covered.get(),
        car.notice_shown.get(),
        car.notice_time.get(),
        car.notice_time_b.get(),
    )
    .shown())
}

pub fn draw_view_strips(overlay: &Overlay) -> Result<(), Refused> {
    use crate::hud::draw_view_strip::{draw_view_strip, overlay_tail};
    let Overlay { race, scratch, .. } = *overlay;
    use rc2k_core::hud::strips::{SECOND_VIEW, TEXTURES};
    {
        if race.shared.replay_file().hold_car_revs.get() == 0 {
            if flashing(&race.shared, race.shared.view.get().view_blocks[0].get())? {
                scratch.polygon.polygon_textures[0].set(TEXTURES[0]);
                draw_view_strip(overlay, shown_layout(&race.shared))?;
            }
            if shown_layout(&race.shared) != 0
                && flashing(&race.shared, race.shared.view.get().view_blocks[1].get())?
            {
                scratch.polygon.polygon_textures[0].set(TEXTURES[1]);
                draw_view_strip(
                    overlay,
                    shown_layout(&race.shared).wrapping_add(SECOND_VIEW),
                )?;
            }
        }
        overlay_tail(overlay)
    }
}
