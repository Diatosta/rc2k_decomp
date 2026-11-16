//! The values on the visual preferences page for the current player: "no" / "yes" for the
//! progress bar, the driver's information, the pace notes, the rear mirror, the message bar, the
//! rev counter, the car's information and the local map, then the units' setting ("imperial" or
//! "metric"). Refuses no current player and what `draw_string_by_id` refuses.

use crate::errors::Refused;
use crate::menu::player::Player;
use crate::menu::player_settings::current;
use crate::text::draw_text::Canvas;

// ---- game data ----

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

/// Returns whether the last text drawn was too wide for its box.
pub fn draw_visual_preferences(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let menus = o.menus;
    // The logic is in `rc2k_core::screens::settings_pages::visual_preferences`.
    let player = current(menus)?.get();
    let ids = rows(&player).map(|(value, first)| value.wrapping_add(first));
    rc2k_core::screens::settings_pages::visual_preferences(
        &mut crate::screens::game_screen::SettingsScreen::new(canvas, o),
        ids,
    )
}
