//! The two other name lists the text entry works on, each 100 names of 0x20 bytes: the saved
//! games (`byte_526BB4`, ten to a page, page in `dword_75523C`) and the replays
//! (`byte_528934`, twenty to a page, page in `dword_755248`). Which list is which comes from
//! the code that reads `savedata\savegame\svgameNN` and `savedata\replay\replayNN` into them.

use crate::errors::BadInput;
use crate::place::Place;
use crate::race::owner::race;
use crate::saves::owner::saves;
use crate::service::state::menu_item;
use crate::session::{Bytes, Dword};

pub const NAME_LEN: usize = 0x20;
const NAMES: u32 = 100;

pub const SAVEGAME_PAGE_RVA: &Dword = &crate::menu::owner::MENUS.tables.pages.save_page; // dword_75523C
const SAVEGAMES_PER_PAGE: u32 = 10;

pub const REPLAY_PAGE_RVA: &Dword = &crate::menu::owner::MENUS.tables.pages.replay_page; // dword_755248
const REPLAYS_PER_PAGE: u32 = 20;

fn highlighted<const N: usize>(
    names: &'static Bytes<N>,
    page: &Dword,
    per_page: u32,
) -> Result<Place, BadInput> {
    let index = page
        .get()
        .wrapping_mul(per_page)
        .wrapping_add(menu_item())
        .wrapping_sub(1);
    if index >= NAMES {
        return Err(BadInput("highlighted name past the list"));
    }
    Ok(Place::In(names, index as usize * NAME_LEN))
}

/// The name of the highlighted saved game.
pub fn savegame_name() -> Result<Place, BadInput> {
    highlighted(
        &saves().save_games.names,
        SAVEGAME_PAGE_RVA,
        SAVEGAMES_PER_PAGE,
    )
}

/// The name of the highlighted replay.
pub fn replay_name() -> Result<Place, BadInput> {
    highlighted(
        &race().shared.replays.names,
        REPLAY_PAGE_RVA,
        REPLAYS_PER_PAGE,
    )
}
