//! `menu::begin_player_name` (0x467C34): typing a player's name begins.
//!
//! Unless the Enter came with ah = 4 (`g_enterSkipsNameEntry` at 1): the current player's name kept
//! (`menu/player_name.rs`), cleared (`menu::clear_text`), and the entry going on (`option_work.typing` =
//! 1); then, either way, the length typed (`loop_flags.text_length`) 0. The action of the name's item of the
//! four players' details pages. Refuses no current player or a player without a name. The menus and
//! draw lists run its `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::player_name::{kept, name_place};
use crate::menu::text_entry::{Text, field, put_field, with_entry};

pub fn begin_player_name(menus: &Menus) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::text_entry::Entry::begin_player_name`.
    let with_4 = { menus.screens.flow.enter_skips_name_entry.get() } == 1;
    let at = if with_4 {
        None
    } else {
        Some(name_place(menus)?)
    };
    // The copy is written, not read.
    let (mut text, mut copy) = (at.map_or(Ok(Text::default()), field)?, Text::default());
    with_entry(menus, |e| e.begin_player_name(with_4, &mut text, &mut copy));
    if let Some(at) = at {
        put_field(at, &text)?;
        put_field(kept(), &copy)?;
    }
    Ok(())
}
