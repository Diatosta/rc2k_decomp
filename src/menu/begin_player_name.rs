//! `menu::begin_player_name`: typing a player's name begins.
//!
//! Unless the Enter came as the key that skips name entry: the current player's name is kept
//! (`menu/player_name.rs`), cleared (`menu::clear_text`), and the entry goes on
//! (`entry.typing` = 1); then, either way, the length typed
//! (`loop_flags.text_length`) is set to 0. The action of the name's item of the four players'
//! details pages. Refuses no current player or a player without a name. The menus and draw
//! lists run its `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::player_name::{kept, name_place};
use crate::menu::text_entry::{Text, field, put_field, with_entry};

pub fn begin_player_name(menus: &'static Menus) -> Result<(), Refused> {
    let with_4 = { menus.screens.flow.enter_skips_name_entry.get() } == 1;
    let at = if with_4 {
        None
    } else {
        Some(name_place(menus)?.place(menus))
    };
    // The copy is written, not read.
    let (mut text, mut copy) = (at.map_or(Ok(Text::default()), field)?, Text::default());
    with_entry(menus, |e| e.begin_player_name(with_4, &mut text, &mut copy));
    if let Some(at) = at {
        put_field(at, &text)?;
        put_field(kept(menus), &copy)?;
    }
    Ok(())
}
