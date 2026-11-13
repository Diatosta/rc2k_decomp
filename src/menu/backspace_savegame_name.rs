//! `menu::backspace_savegame_name`: takes the last character off the name of the
//! highlighted saved game: if the text entry's length is not 0, it is lowered by one and a
//! NUL is written there.
//!
//! Also called by the replay name editor, which is a slip kept as is (see `edit_replay_name`).
//! Refuses a name past the list and a length outside 0 .. 0x1F.

use crate::errors::BadInput;
use crate::menu::name_lists::{NAME_LEN, savegame_name};
use crate::menu::owner::Menus;
use crate::menu::text_entry::{entry as text_entry, length, put_entry};
use crate::saves::owner::Saves;

pub fn backspace_savegame_name(menus: &Menus, saves: &'static Saves) -> Result<(), BadInput> {
    if length(menus) == 0 {
        return Ok(());
    }
    let name = savegame_name(menus, saves)?;
    let mut e = text_entry(menus);
    if let Some(at) = e.backspace(NAME_LEN)? {
        put_entry(menus, &e);
        name.add(at as u32).write(&[0], "a saved game's name")?;
    }
    Ok(())
}
