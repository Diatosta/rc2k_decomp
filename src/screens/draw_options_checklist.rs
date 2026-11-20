//! A player's options page: its title (a string, a space and the player's number), the page's
//! lines, the slider (the inspection score percent) with its knob and two labels, and a tick or
//! cross per option.

use crate::errors::Refused;
use crate::memory::session::PlainDword;
use crate::text::draw_text::Canvas;

/// (id, x, y, font) of the page's lines.
pub fn draw_options_checklist(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    // The logic is in `rc2k_core::screens::settings_pages::options_checklist`.
    {
        // The fault flags end with their field's -1: `menu::inspection_faults` writes only
        // the thirteen before it, each 0 or 1.
        let faults = &menus.lists.inspection_faults;
        let end = faults
            .iter()
            .position(|f| f.get() == u32::MAX)
            .ok_or(crate::errors::BadInput("the fault flags without their end"))?;
        let ticks: Vec<u32> = faults[..end].iter().map(PlainDword::get).collect();
        rc2k_core::screens::settings_pages::options_checklist(
            &mut crate::screens::game_screen::SettingsScreen::new(canvas, o),
            menus.screens.event.read().current_player,
            menus.screens.state.read().driver_stats.score as i32,
            &ticks,
        )
    }
}
