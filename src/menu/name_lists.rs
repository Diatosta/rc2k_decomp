//! The two other name lists the text entry works on, each 100 names of 0x20 bytes: the saved
//! games (`byte_526BB4`, ten to a page, page in `dword_75523C`) and the replays
//! (`byte_528934`, twenty to a page, page in `dword_755248`). Which list is which comes from
//! the code that reads `savedata\savegame\svgameNN` and `savedata\replay\replayNN` into them.

use crate::errors::BadInput;
use crate::framework::{game_ptr, game_read, rva};
use crate::service::state::menu_item;

pub const NAME_LEN: usize = 0x20;
const NAMES: u32 = 100;

pub const SAVEGAME_NAMES_RVA: usize = rva(0x526BB4); // byte_526BB4
pub const SAVEGAME_PAGE_RVA: usize = rva(0x75523C); // dword_75523C
const SAVEGAMES_PER_PAGE: u32 = 10;

pub const REPLAY_NAMES_RVA: usize = rva(0x528934); // byte_528934
pub const REPLAY_PAGE_RVA: usize = rva(0x755248); // dword_755248
const REPLAYS_PER_PAGE: u32 = 20;

fn highlighted(names_rva: usize, page_rva: usize, per_page: u32) -> Result<*mut u8, BadInput> {
    let index = unsafe { game_read::<u32>(page_rva) }
        .wrapping_mul(per_page)
        .wrapping_add(menu_item())
        .wrapping_sub(1);
    if index >= NAMES {
        return Err(BadInput("highlighted name past the list"));
    }
    Ok(game_ptr::<u8>(names_rva).wrapping_add(index as usize * NAME_LEN))
}

/// The name of the highlighted saved game.
pub fn savegame_name() -> Result<*mut u8, BadInput> {
    highlighted(SAVEGAME_NAMES_RVA, SAVEGAME_PAGE_RVA, SAVEGAMES_PER_PAGE)
}

/// The name of the highlighted replay.
pub fn replay_name() -> Result<*mut u8, BadInput> {
    highlighted(REPLAY_NAMES_RVA, REPLAY_PAGE_RVA, REPLAYS_PER_PAGE)
}
