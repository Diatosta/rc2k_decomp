//! The two other name lists the text entry works on, each 100 names of 0x20 bytes: the saved
//! games (the sound file path buffer, ten to a page, page in `pages.save_page`) and the replays
//! (the picture path buffer, twenty to a page, page in `pages.replay_page`). Which list is which comes from
//! the code that reads `savedata\savegame\svgameNN` and `savedata\replay\replayNN` into them.

use crate::errors::BadInput;
use crate::place::BytePlace;
use crate::race::owner::race;
use crate::saves::owner::saves;
use crate::service::state::menu_item;
use crate::session::PlainDword;

pub const NAME_LEN: usize = 0x20;
const NAMES: u32 = 100;

pub const SAVEGAME_PAGE_RVA: &PlainDword =
    &crate::menu::owner::MENUS.unchecked().tables.pages.save_page;
const SAVEGAMES_PER_PAGE: u32 = 10;

pub const REPLAY_PAGE_RVA: &PlainDword = &crate::menu::owner::MENUS
    .unchecked()
    .tables
    .pages
    .replay_page;
const REPLAYS_PER_PAGE: u32 = 20;

fn highlighted(
    names: &'static dyn crate::place::Run,
    page: &PlainDword,
    per_page: u32,
) -> Result<BytePlace, BadInput> {
    let index = page
        .get()
        .wrapping_mul(per_page)
        .wrapping_add(menu_item())
        .wrapping_sub(1);
    if index >= NAMES {
        return Err(BadInput("highlighted name past the list"));
    }
    Ok(BytePlace::In(names, index as usize * NAME_LEN))
}

/// The name of the highlighted saved game.
pub fn savegame_name() -> Result<BytePlace, BadInput> {
    highlighted(
        &saves().save_games.names,
        SAVEGAME_PAGE_RVA,
        SAVEGAMES_PER_PAGE,
    )
}

/// The name of the highlighted replay.
pub fn replay_name() -> Result<BytePlace, BadInput> {
    highlighted(
        &race().shared.replays.names,
        REPLAY_PAGE_RVA,
        REPLAYS_PER_PAGE,
    )
}
