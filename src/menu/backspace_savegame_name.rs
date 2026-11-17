//! `menu::backspace_savegame_name`: takes the last character off the name of the
//! highlighted saved game: if the text entry's length is not 0, it is lowered by one and a
//! NUL is written there.
//!
//! `backspace_name` does the same for any name place (the replay name editor's).
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
    backspace_name(menus, savegame_name(menus, saves)?)
}

/// Takes the last character off the entry being typed into `name`.
pub fn backspace_name(
    menus: &Menus,
    name: crate::memory::place::BytePlace,
) -> Result<(), BadInput> {
    if length(menus) == 0 {
        return Ok(());
    }
    let mut e = text_entry(menus);
    if let Some(at) = e.backspace(NAME_LEN)? {
        put_entry(menus, &e);
        name.add(at as u32).write(&[0], "a saved game's name")?;
    }
    Ok(())
}
