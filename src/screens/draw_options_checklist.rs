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
use crate::menu::event_store::EVENT_STATE;
use crate::menu::lists_store::MENU_LISTS;
use crate::menu::state_store::MENU_STATE;
use crate::text::draw_text::Canvas;

/// (id, x, y, font) of the page's lines.
/// The Rust version.
pub fn draw_options_checklist(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::settings_pages::options_checklist`.
    {
        let mut at = MENU_LISTS.inspection_faults[0].addr() as u32;
        let mut ticks = Vec::new();
        loop {
            let tick = crate::pointers::GamePtr::new(at).read::<u32>("a checklist tick")?;
            if tick == u32::MAX {
                break;
            }
            ticks.push(tick);
            at += 4;
        }
        rc2k_core::screens::settings_pages::options_checklist(
            &mut crate::screens::game_screen::SettingsScreen::new(canvas),
            EVENT_STATE.current_player.get(),
            MENU_STATE.driver_stats.score.get() as i32,
            &ticks,
        )
    }
}
