//! `menu::begin_player_name`: typing a player's name begins.
//!
//! The current player's name is kept (`menu/player_name.rs`), cleared (`menu::clear_text`), the
//! entry goes on (`entry.typing` = 1) and the length typed (`loop_flags.text_length`) is set to 0,
//! whenever the Enter comes. The action of the
//! name's item of the four players' details pages. Refuses no current player or a player without a
//! name.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::player_name::{kept, name_place};
use crate::menu::text_entry::{Text, field, put_field, with_entry};

pub fn begin_player_name(menus: &'static Menus) -> Result<(), Refused> {
    let at = name_place(menus)?.place(menus);
    // The copy is written, not read.
    let (mut text, mut copy) = (field(at)?, Text::default());
    with_entry(menus, |e| e.begin_player_name(&mut text, &mut copy));
    put_field(at, &text)?;
    put_field(kept(menus), &copy)?;
    Ok(())
}
