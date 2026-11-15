//! The stage panel opened from the service area, started by `menu::view_stage_details` when
//! the event type is not 3: `draw_stage_info`'s panel with two differences at the top: the stage
//! heading is drawn only in mode 2 with `event.stage.pages` at 0, and a second heading text
//! (string 0x482) is built in its own buffer without being drawn. Seen: the stage details that can
//! be opened from a championship's service area, before the stage.

use crate::errors::Refused;
use crate::screens::draw_stage_heading::draw_stage_heading;
use crate::screens::draw_stage_info::{TITLE_X, TITLE_Y, event_stage, event_title};
use crate::screens::stage_panel::{
    TITLE_FONT, build_second_heading, draw_records_from, draw_stage_body, label,
};
use crate::text::draw_text::Canvas;

// ---- game data ----

/// Returns whether `draw_stage_records`' last text was too wide for its box.
pub fn draw_service_stage_info(
    o: crate::owners::Owners<'static>,
    system: &crate::system::owner::System,
    gfx: &crate::gfx::owner::Gfx,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let menus = o.menus;
    let screens = &menus.screens;
    label(
        crate::strings::append_string_by_id::StringOwners::of(o),
        canvas,
        event_title(&menus.play.champ_view, &menus.screens.event, system)?,
        TITLE_FONT,
        TITLE_X,
        TITLE_Y,
    )?;
    let (record, _) = event_stage(screens, system)?;
    if menus.play.champ_view.event_mode.get() == 2 && menus.screens.event.stage.pages.get() == 0 {
        draw_stage_heading(o, record, canvas)?;
    }
    build_second_heading(
        crate::strings::append_string_by_id::StringOwners::of(o),
        record,
    )?;
    draw_stage_body(o, gfx, record, false, canvas)?;
    draw_records_from(o, system, &o.saves.best_tables.stage_times, record, canvas)
}
