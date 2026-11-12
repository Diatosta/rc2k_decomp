//! `menu::backspace_savegame_name` (0x46D5FD): takes the last character off the name of the
//! highlighted saved game: if the text entry's length is not 0, it is lowered by one and a
//! NUL is written there.
//!
//! Also called by the replay name editor, which is a slip in the original (see
//! `edit_replay_name`). Refuses a name past the list and a length outside 0 .. 0x1F.

use crate::errors::BadInput;
use crate::menu::name_lists::{NAME_LEN, savegame_name};
use crate::menu::owner::Menus;
use crate::menu::text_entry::{entry as text_entry, length, put_entry};

pub fn backspace_savegame_name(menus: &Menus) -> Result<(), BadInput> {
    // The logic is in `rc2k_core::menu::text_entry::Entry::backspace`.
    if length() == 0 {
        return Ok(());
    }
    let name = savegame_name(menus)?;
    let mut e = text_entry(menus);
    if let Some(at) = e.backspace(NAME_LEN)? {
        put_entry(menus, &e);
        name.add(at as u32).write(&[0], "a saved game's name")?;
    }
    Ok(())
}
