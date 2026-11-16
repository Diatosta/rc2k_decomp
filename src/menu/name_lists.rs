//! The two other name lists the text entry works on, each 100 names of 0x20 bytes: the saved
//! games (the sound file path buffer, ten to a page, page in `pages.save_page`) and the replays
//! (the picture path buffer, twenty to a page, page in `pages.replay_page`). Which list is
//! which comes from the code that reads `savedata\savegame\svgameNN` and
//! `savedata\replay\replayNN` into them.

use crate::errors::BadInput;
use crate::memory::place::BytePlace;
use crate::memory::session::PlainDword;
use crate::menu::owner::Menus;
use crate::race::owner::Race;
use crate::saves::owner::Saves;
use crate::service::state::menu_item;

pub const NAME_LEN: usize = 0x20;
const NAMES: u32 = 100;

const SAVEGAMES_PER_PAGE: u32 = 10;

const REPLAYS_PER_PAGE: u32 = 20;

fn highlighted(
    menus: &Menus,
    names: &'static dyn crate::memory::place::Run,
    page: &PlainDword,
    per_page: u32,
) -> Result<BytePlace, BadInput> {
    let index = page
        .get()
        .wrapping_mul(per_page)
        .wrapping_add(menu_item(menus))
        .wrapping_sub(1);
    if index >= NAMES {
        return Err(BadInput("highlighted name past the list"));
    }
    Ok(BytePlace::In(names, index as usize * NAME_LEN))
}

/// The name of the highlighted saved game.
pub fn savegame_name(menus: &Menus, saves: &'static Saves) -> Result<BytePlace, BadInput> {
    highlighted(
        menus,
        &saves.save_games.names,
        &menus.tables.pages.save_page,
        SAVEGAMES_PER_PAGE,
    )
}

/// The name of the highlighted replay.
pub fn replay_name(menus: &Menus, race: &'static Race) -> Result<BytePlace, BadInput> {
    highlighted(
        menus,
        &race.shared.replays().names,
        &menus.tables.pages.replay_page,
        REPLAYS_PER_PAGE,
    )
}
