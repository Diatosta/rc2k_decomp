//! `menu::fit_player_names`: cuts each player's name until it is no wider than
//! 350 pixels.
//!
//! For each of the four player names (`menu::fit_name`): the name is measured with the font
//! (`text::measure_text`) and, while it is wider than 0x15E, loses its last character and is
//! measured again. A measured width does not depend on the x it starts from, so each is measured
//! from 0 (`measure_text::only_measures`). Refuses a name that would be cut to before its start,
//! and what the measuring refuses.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::player_names::{NameAt, PLAYERS};
use crate::text::draw_text::GameCanvas;
use crate::text::measure_string_by_id::MEASURING_FONT;
use crate::text::measure_text::measure_text_at;
use rc2k_core::menu::text_entry;

/// More characters than a name is looked through for its end.
const LONGEST: usize = 0x1000;

/// `menu::fit_name`. Returns the width of the name as it is left.
pub fn fit_name(
    menus: &'static Menus,
    player: usize,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<u32, Refused> {
    let at = NameAt::of(player);
    let mut name = at.tail(&menus.play);
    let Some(len) = (0..LONGEST).find(|&n| name.get(n) == Some(&0)) else {
        return Err(BadInput("a name without its end").into());
    };
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
