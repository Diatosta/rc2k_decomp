//! Player 1's details page; the other players' pages share this file's code
//! (`draw_player2_details` is the same, `draw_player34_details` is the page without the second
//! view: see `Layout`).
//! - The controller's item (9) of the current menu is greyed out when there is no controller,
//!   else made choosable.
//! - The current player's name, with the cursor after it while a name is being typed.
//! - The nationality's string and its flag after it.
//! - The view and, on the pages that have it, the second view (a 5 is made 0 first).
//! - The race number and the co-driver's sex.
//! - The controller sensitivity's slider sprites.
//!
//! Refuses a player past the four, a player without a name, an index past a string table and what
//! its parts refuse, before the controller's item is switched.

use crate::errors::{BadInput, Refused};
use crate::menu::player::Player;
use crate::text::draw_text::Canvas;

// ---- game data ----

/// What differs between the players' pages: players 1 and 2 have the controller's item at 9
/// and the second view's line; players 3 and 4 have it at 8 and no second view, the lines below
/// moving up one.
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

fn string(
    table: &[crate::memory::session::PlainDword],
    index: u32,
    count: u32,
) -> Result<u32, Refused> {
    if index >= count {
        return Err(BadInput("an index past its string table").into());
    }
    Ok(table[index as usize].get())
}

pub fn draw_player_page(
    o: crate::owners::Owners<'static>,
    input: &crate::input::owner::Input,
    layout: Layout,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    let screens = &menus.screens;
    let Some(cell) = crate::menu::player::player(menus, screens.event.read().current_player) else {
        return Err(BadInput("a player past the four").into());
    };
    let player: Player = cell.get();
    if player.name_player().is_none() {
        return Err(BadInput("a player without its name").into());
    }
    // The logic is in `rc2k_core::screens::player_details`; the strings looked up (and refused)
    // before anything is drawn, the second view of 5 made 0 then.
    let nationality_id = string(
        &menus.tables.captions.nationalities,
        player.nationality,
        NATIONALITIES,
    )?;
    let view_id = string(&menus.tables.captions.views, player.view, VIEWS)?;
    let second_view_id = if layout.second_view {
        let view = if player.second_view == 5 {
            0
        } else {
            player.second_view
        };
        let id = string(&menus.tables.captions.second_views, view, SECOND_VIEWS)?;
        if player.second_view == 5 {
            cell.update(|p| p.second_view = 0);
        }
        Some(id)
    } else {
        None
    };
    let details = rc2k_core::screens::player_details::Details {
        controller_item: layout.controller_item,
        controllers: input.devices.controller_count.get() != 0,
        name: crate::screens::game_screen::player_name_of(crate::menu::player_names::name_of(
            &player,
        )?),
        typing: screens.entry.read().typing,
        blink: screens.draw_work.read().cursor_on,
        nationality: player.nationality,
        nationality_id,
        view_id,
        second_view_id,
        race_number: player.race_number,
        codriver_sex: player.codriver_sex,
        sensitivity: player.controller_sensitivity,
    };
    rc2k_core::screens::player_details::player_details(
        &mut crate::screens::game_screen::DetailsScreen::new(canvas, o),
        &details,
    )?;

    Ok(())
}
