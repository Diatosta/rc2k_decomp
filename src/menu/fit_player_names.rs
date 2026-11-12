//! `menu::fit_player_names` (0x411445): cuts each player's name until it is no wider than
//! 350 pixels.
//!
//! For each of the four names in `g_playerNames` (`menu::fit_name`, 0x411474, a plain
//! function here): the name is measured with the font at 0x51E824 (`text::measure_text`)
//! and, while it is wider than 0x15E, loses its last character and is measured again.
//! The x given to a measuring is the width the one before gave (the first gets the
//! caller's ecx): the original leaves it in ecx. A measured width does not depend on it, so
//! each is measured from 0 (`measure_text::only_measures`).
//! One caller, the main menu's set-up `menu::set_up_front_end`.
//! Refuses a name that would be cut to before its start, and what the measuring refuses.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::player_names::{NameAt, PLAYERS};
use crate::text::draw_text::GameCanvas;
use crate::text::measure_string_by_id::MEASURING_FONT;
use crate::text::measure_text::measure_text_at;
use rc2k_core::menu::text_entry;

// ---- game data ----

/// More characters than a name is looked through for its end.
const LONGEST: usize = 0x1000;

/// `menu::fit_name` (0x411474). Returns the width of the name as it is left.
pub fn fit_name(
    menus: &'static Menus,
    player: usize,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<u32, Refused> {
    let at = NameAt::of(player);
    let mut name = at.tail(menus);
    let Some(len) = (0..LONGEST).find(|&n| name.get(n) == Some(&0)) else {
        return Err(BadInput("a name without its end").into());
    };
    // The logic is in `rc2k_core::menu::text_entry::fit_name`.
    let font = crate::text::measure_text::only_measures(MEASURING_FONT, &gfx.fonts.typefaces)?;
    text_entry::fit_name(len, |len| -> Result<u32, Refused> {
        at.add(len as u32).place(menus).write(&[0], "a name")?;
        name[len] = 0;
        Ok({ measure_text_at(font, &name, 0, &mut GameCanvas::new(gfx)) }?.width)
    })
}

pub fn fit_player_names(
    menus: &'static Menus,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    for player in 0..PLAYERS {
        fit_name(menus, player, gfx)?;
    }
    Ok(())
}
