//! The two other name lists the text entry works on, each 100 names of 0x20 bytes: the saved
//! games (`byte_526BB4`, ten to a page, page in `dword_75523C`) and the replays
//! (`byte_528934`, twenty to a page, page in `dword_755248`). Which list is which comes from
//! the code that reads `savedata\savegame\svgameNN` and `savedata\replay\replayNN` into them.

use crate::errors::BadInput;
use crate::menu::pages_store::MENU_PAGES;
use crate::service::state::menu_item;
use crate::session::{Bytes, Dword};

pub const NAME_LEN: usize = 0x20;
const NAMES: u32 = 100;

pub const SAVEGAME_PAGE_RVA: &Dword = &MENU_PAGES.save_page; // dword_75523C
const SAVEGAMES_PER_PAGE: u32 = 10;

pub const REPLAY_PAGE_RVA: &Dword = &MENU_PAGES.replay_page; // dword_755248
const REPLAYS_PER_PAGE: u32 = 20;

fn highlighted<const N: usize>(
    names: &Bytes<N>,
    page: &Dword,
    per_page: u32,
) -> Result<u32, BadInput> {
    let index = page
        .get()
        .wrapping_mul(per_page)
        .wrapping_add(menu_item())
        .wrapping_sub(1);
    if index >= NAMES {
        return Err(BadInput("highlighted name past the list"));
    }
    Ok((names.addr() + index as usize * NAME_LEN) as u32)
}

/// The name of the highlighted saved game.
pub fn savegame_name() -> Result<u32, BadInput> {
    highlighted(
        &crate::saves::write_savegame_slot::SAVE_GAMES.names,
        SAVEGAME_PAGE_RVA,
        SAVEGAMES_PER_PAGE,
    )
}

/// The name of the highlighted replay.
pub fn replay_name() -> Result<u32, BadInput> {
    highlighted(
        &crate::race::replay::REPLAYS.names,
        REPLAY_PAGE_RVA,
        REPLAYS_PER_PAGE,
    )
}
