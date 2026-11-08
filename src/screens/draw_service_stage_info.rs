//! `screens::draw_service_stage_info` (0x480064): the stage panel opened from the service
//! area. It is `draw_stage_info`'s panel with two differences at the top: the stage heading
//! is drawn only in mode 2 with `dword_755968` at 0, and a second heading text (string 0x482)
//! is built in its own buffer without being drawn here.
//!
//! Reached through the draw list at 0x758D86 of the screen loop `menu::view_stage_loop`, which the
//! menu
//! action `menu::view_stage_details` starts when the event type `dword_755954` is not 3. That
//! action is the last of 22 items (all at 747, 551) of the service area's menu at 0x74D24C.
//!
//! Screen: confirmed by the owner on 2026-10-10: the stage details that can be opened from the
//! service area of a championship, the screen after the first stage details where the car is set up
//! and repaired before the stage. Named `draw_stage_info_alt` until then. The menus and draw lists
//! run its `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::screens::draw_stage_heading::draw_stage_heading;
use crate::screens::draw_stage_info::{TITLE_X, TITLE_Y, event_stage, event_title};
use crate::screens::stage_panel::{
    TITLE_FONT, build_second_heading, draw_records_from, draw_stage_body, label,
};
use crate::text::draw_text::Canvas;

// ---- game data ----

// ---- Rust version ----

/// Returns the carry flag of `draw_stage_records`.
pub fn draw_service_stage_info(menus: &Menus, canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    let screens = &menus.screens;
    let tables = &menus.tables;
    label(canvas, event_title(menus)?, TITLE_FONT, TITLE_X, TITLE_Y)?;
    let (record, _) = event_stage(screens)?;
    if menus.play.champ_view.event_mode.get() == 2 && menus.screens.event.stage_pages.get() == 0 {
        draw_stage_heading(record, canvas)?;
    }
    build_second_heading(record)?;
    draw_stage_body(tables, record, false, canvas)?;
    draw_records_from(
        menus,
        &crate::saves::best_times::STAGE_TIMES,
        record,
        canvas,
    )
}
