//! `screens::draw_visual_preferences` (0x47A4C7): the values on the visual preferences page.
//!
//!
//! For the current player (`event.current_player`), at x = 0x1D1 from y = 0xC8 on every 0x1D, in the font
//! at 0x51E8C4: string 0x56B + the setting ("não", "sim") for the progress bar, the driver's
//! information, the pace notes, the rear mirror, the message bar, the rev counter, the car's
//! information and the local map, then string 0x373 + the units' setting ("imperial", "métrico").
//! Each id is stored in `draw_work.figure` first. In the draw list 0x758CAA
//! (`menu::visual_preferences_loop`). Refuses no current player and what `draw_string_by_id`
//! refuses. The menus and draw lists run its `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::player::Player;
use crate::menu::player_settings::current;
use crate::text::draw_text::Canvas;

// ---- game data ----

// `draw_work.figure`
const NO: u32 = 0x56B;
const IMPERIAL: u32 = 0x373;

/// The rows, in the order drawn: the setting and the string it counts from.
fn rows(p: &Player) -> [(u32, u32); 9] {
    [
        (p.progress_bar, NO),
        (p.driver_info, NO),
        (p.pace_notes, NO),
        (p.rear_mirror, NO),
        (p.message_display, NO),
        (p.rev_counter, NO),
        (p.car_info, NO),
        (p.map_area, NO),
        (p.metric_units, IMPERIAL),
    ]
}

/// Returns the carry.
pub fn draw_visual_preferences(menus: &Menus, canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    // Converted: `rc2k_core::screens::settings_pages::visual_preferences`.
    let player = current(menus)?.get();
    let ids = rows(&player).map(|(value, first)| value.wrapping_add(first));
    rc2k_core::screens::settings_pages::visual_preferences(
        &mut crate::screens::game_screen::SettingsScreen::new(canvas),
        ids,
    )
}
