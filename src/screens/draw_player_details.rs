//! `screens::draw_player_details` (0x46F894): player 1's details page. The players'
//! pages share this file's code: `draw_player2_details` (0x4677B5) is the same code,
//! `draw_player34_details` (0x467975) the page without the second view (`Layout`).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! - The controller's item (9) of the current menu greyed out when there is no
//!   controller (`g_controllerCount`), else made choosable (`menu::disable_item`,
//!   `menu::enable_item`).
//! - The player of the player `dword_75598C` (`Player`): its name at (0x1BB, 0xD4) in
//!   the font at 0x51E8C4, and while a name is being typed (`dword_7556A8` not 0) the
//!   cursor after it (`screens::draw_name_cursor`).
//! - The nationality's string (`dword_758816[nationality]`, kept in `seconds`,
//!   0x7559AC) at (0x1BB, 0xEF); measured with the font at 0x51E824 from the x the text
//!   left, its flag (sprite 0x1C + nationality) at 0x1CA + the width, y 0xF3.
//! - The view `dword_7589BA[view]` at y 0x10A; the second view
//!   `dword_7589DA[second_view]` at 0x125, a second view of 5 made 0 first (only on the
//!   pages that have it; on the others the lines below are 0x1B higher).
//! - `race_number` as a number at 0x140; the co-driver's sex's string (0x18A + it, kept in
//!   `dword_7559EC`) at 0x15B.
//! - Sprite 4 at (0x1BA, 0x176) and sprite 6 at (0x1BA + 2 * `controller_sensitivity`,
//!   0x176), the
//!   blits' palette being what the last text left in ebx.
//!
//! In the draw list at 0x758CBA (the four players' details screens). Refuses a player past the
//! four, a player without its name, an index past its string table, and what its parts refuse;
//! since the conversion (2026-10-20) before the controller's item is switched (the original
//! switches it first). The menus and draw lists run its `entry` (`menu::table_calls`); the
//! registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::menus;
use crate::menu::player::Player;
use crate::session::Dword;
use crate::text::draw_text::Canvas;

// ---- game data ----

// seconds
// dword_7559EC
const NATIONALITIES_RVA: &[Dword] = &crate::menu::owner::MENUS.tables.captions.nationalities; // dword_758816
const VIEWS_RVA: &[Dword] = &crate::menu::owner::MENUS.tables.captions.views; // dword_7589BA
const SECOND_VIEWS_RVA: &[Dword] = &crate::menu::owner::MENUS.tables.captions.second_views; // dword_7589DA
/// What differs between the players' pages: players 1 and 2 (`sub_46F894`,
/// `sub_4677B5`) have the controller's item at 9 and the second view's line; players 3
/// and 4 (`sub_467975`) have it at 8 and no second view, the lines below moving up one.
#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub controller_item: u32,
    pub second_view: bool,
}

pub const FULL: Layout = Layout {
    controller_item: 9,
    second_view: true,
};
pub const SHORT: Layout = Layout {
    controller_item: 8,
    second_view: false,
};
/// The tables' lengths (each ended by -1): 40 nationalities, seven views, six second
/// views.
const NATIONALITIES: u32 = 40;
const VIEWS: u32 = 7;
const SECOND_VIEWS: u32 = 6;
// ---- Rust version ----

fn string(table: &[Dword], index: u32, count: u32) -> Result<u32, Refused> {
    if index >= count {
        return Err(BadInput("an index past its string table").into());
    }
    Ok(table[index as usize].get())
}

pub fn draw_player_page(layout: Layout, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let screens = &menus().screens;
    let Some(cell) = crate::menu::player::player(screens.event.current_player.get()) else {
        return Err(BadInput("a player past the four").into());
    };
    let player: Player = cell.get();
    if player.name_player().is_none() {
        return Err(BadInput("a player without its name").into());
    }
    // Converted: `rc2k_core::screens::player_details`; the strings looked up (and refused)
    // before anything is drawn, the second view of 5 made 0 then.
    let nationality_id = string(NATIONALITIES_RVA, player.nationality, NATIONALITIES)?;
    let view_id = string(VIEWS_RVA, player.view, VIEWS)?;
    let second_view_id = if layout.second_view {
        let view = if player.second_view == 5 {
            0
        } else {
            player.second_view
        };
        let id = string(SECOND_VIEWS_RVA, view, SECOND_VIEWS)?;
        if player.second_view == 5 {
            cell.update(|p| p.second_view = 0);
        }
        Some(id)
    } else {
        None
    };
    let details = rc2k_core::screens::player_details::Details {
        controller_item: layout.controller_item,
        controllers: crate::input::store::DEVICES.controller_count.get() != 0,
        name: crate::screens::game_screen::player_name_of(crate::menu::player_names::name_of(
            &player,
        )?),
        typing: screens.option_work.typing.get() != 0,
        blink: screens.draw_work.cursor_on.get(),
        nationality: player.nationality,
        nationality_id,
        view_id,
        second_view_id,
        race_number: player.race_number,
        codriver_sex: player.codriver_sex,
        sensitivity: player.controller_sensitivity,
    };
    rc2k_core::screens::player_details::player_details(
        &mut crate::screens::game_screen::DetailsScreen::new(canvas),
        &details,
    )?;

    Ok(())
}
