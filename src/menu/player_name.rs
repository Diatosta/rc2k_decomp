//! What the players' details pages' name entry shares: the current player's name, and
//! the copy it is kept in while it is typed (`unk_755884`, 0x1F bytes; the other text
//! entries keep theirs at `unk_7558A4`).
//!
//! - `sub_467D20`: edi = the current player's name (`Player::name`), eax kept;
//! - `sub_467D34`: the name's 0x1F bytes to the copy; `sub_467D50`: back again;
//! - `sub_467DA5`: unless the length typed (`dword_75556D`) is 0, one less, and a NUL
//!   there.
//!
//! Inlined into `menu::begin_player_name` and `menu::edit_player_name`.

use crate::errors::{BadInput, Refused};
use crate::menu::player_settings::current;
use crate::menu::text_entry::{TEXT_LEN, length, set_length};
use crate::place::Place;

/// The current player's name as a place bounded by the names' store (the original runs on
/// past a name that lacks its end).
pub fn name_place() -> Result<crate::pointers::GamePtr, Refused> {
    Ok(crate::menu::player_names::name_ptr(name()?))
}

pub fn kept() -> Place {
    Place::from(&crate::menu::owner::menus().play.names.kept_name)
}

/// Whose name the current player has.
pub fn name() -> Result<usize, Refused> {
    Ok(crate::menu::player_names::name_of(&current()?.get())?)
}

pub fn bring_back_name(name: impl Into<Place>) -> Result<(), BadInput> {
    let kept = kept().read(TEXT_LEN, "the kept name")?;
    name.into().write(&kept, "a player's name")
}

pub fn backspace_name(name: impl Into<Place>) -> Result<(), BadInput> {
    let length = length();
    if length != 0 {
        set_length(length.wrapping_sub(1));
        name.into()
            .add(length.wrapping_sub(1) as u32)
            .write(&[0], "a player's name")?;
    }
    Ok(())
}
