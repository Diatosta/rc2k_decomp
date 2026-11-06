//! `screens::draw_options_checklist` (0x47DEA2): a player's options page: its title (string
//! 0x190, a space and the player's number, in `byte_A8BC54`) at (0x190, 0x14), the page's
//! lines (strings 0x524 .. 0x531 at x 0x190 / 0x186, three fonts), the slider (`gfx::blit_bar`
//! at (0x128, 0x1E3), 0xD0 wide, `dword_75AC80` percent) with its knob and two labels, and a
//! tick or cross per option (`dword_759C76` .., -1 ends them; sprite 0x9E + (value ^ 1), its
//! palette the same value) at x 0x190 from y 0x82 every 0x1B (`dword_7559EC`, `hhmm` the
//! glyph, `g_markerX` 0x190).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x75927E). The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::owner::menus;
use crate::text::draw_text::Canvas;

/// (id, x, y, font) of the page's lines.
/// The Rust version.
pub fn draw_options_checklist(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let menus = menus();
    // Converted: `rc2k_core::screens::settings_pages::options_checklist`.
    {
        let mut ticks = Vec::new();
        loop {
            let tick = crate::session::read_on_in(
                &menus.tables.lists,
                &menus.tables.lists.inspection_faults,
                ticks.len(),
                "a checklist tick",
            )?;
            if tick == u32::MAX {
                break;
            }
            ticks.push(tick);
        }
        rc2k_core::screens::settings_pages::options_checklist(
            &mut crate::screens::game_screen::SettingsScreen::new(canvas),
            menus.screens.event.current_player.get(),
            menus.screens.state.driver_stats.score.get() as i32,
            &ticks,
        )
    }
}
