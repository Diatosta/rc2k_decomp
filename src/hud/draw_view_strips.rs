//! Unless `hold_car_revs`, for each player's view block (the second's when `split` is not 0)
//! whose `view_covered` is 0 and `notice_shown` is set: while its flash is on (`notice_time_b` at
//! most 0, or `notice_time` modulo `notice_time_b` below half of `notice_time_b`),
//! `hud::draw_view_strip` for view `split` (the second's
//! plus 3) with texture `polygon_textures` 0xC4 (0xC5); then the strips' shared tail.

use crate::errors::{BadInput, Refused};
use crate::hud::draw_view_strip::Overlay;
use crate::hud::overlay_pass::shown_layout;

/// Whether the strip of the view block `handle` names is shown (see the header).
fn flashing(shared: &crate::race::owner_shared::RaceShared, handle: u32) -> Result<bool, BadInput> {
    let blocks = shared.car_blocks();
    let car = blocks.of(handle).ok_or(BadInput("a view's block"))?;
    Ok(rc2k_core::hud::strips::strip(
        car.view_covered,
        car.notice_shown,
        car.notice_time,
        car.notice_time_b,
    )
    .shown())
}

pub fn draw_view_strips(overlay: &Overlay) -> Result<(), Refused> {
    use crate::hud::draw_view_strip::{draw_view_strip, overlay_tail};
    let Overlay { race, scratch, .. } = *overlay;
    use rc2k_core::hud::strips::{SECOND_VIEW, TEXTURES};
    // Three or four players have no notice strips: their parts are small.
    if shown_layout(&race.shared) > 2 {
        return Ok(());
    }
    {
        if race.shared.replay_file().hold_car_revs.get() == 0 {
            if flashing(&race.shared, race.shared.view.get().view_blocks[0].get())? {
                scratch.polygon.scalars.borrow_mut().polygon_textures[0] = TEXTURES[0];
                draw_view_strip(overlay, shown_layout(&race.shared))?;
            }
            if shown_layout(&race.shared) != 0
                && flashing(&race.shared, race.shared.view.get().view_blocks[1].get())?
            {
                scratch.polygon.scalars.borrow_mut().polygon_textures[0] = TEXTURES[1];
                draw_view_strip(
                    overlay,
                    shown_layout(&race.shared).wrapping_add(SECOND_VIEW),
                )?;
            }
        }
        overlay_tail(overlay)
    }
}
