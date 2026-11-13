//! What the players' details pages' name entry shares: the current player's name, and the copy
//! it is kept in while it is typed (the kept name, 0x1F bytes; the other text entries keep
//! theirs at the text backup).
//!
//! - `menu::player_name`: the current player's name (`Player::name`);
//! - the name's 0x1F bytes to the copy, and back again;
//! - unless the length typed (`loop_flags.text_length`) is 0, one less, and a NUL there.
//!
//! Inlined into `menu::begin_player_name` and `menu::edit_player_name`.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::player_names::NameAt;
use crate::menu::player_settings::current;
use crate::menu::text_entry::{TEXT_LEN, length, set_length};
use crate::place::BytePlace;

/// The current player's name as a place bounded by the names' area (a name that lacks its end
/// is read on past it).
pub fn name_place(menus: &Menus) -> Result<NameAt, Refused> {
    Ok(NameAt::of(name(menus)?))
}

pub fn kept(menus: &'static Menus) -> BytePlace {
    BytePlace::from(&menus.play.names.kept_name)
}

/// Whose name the current player has.
pub fn name(menus: &Menus) -> Result<usize, Refused> {
    Ok(crate::menu::player_names::name_of(&current(menus)?.get())?)
}

pub fn bring_back_name(menus: &'static Menus, name: BytePlace) -> Result<(), BadInput> {
    let kept = kept(menus).read(TEXT_LEN, "the kept name")?;
    name.write(&kept, "a player's name")
}

pub fn backspace_name(menus: &Menus, name: BytePlace) -> Result<(), BadInput> {
    let length = length(menus);
    if length != 0 {
        set_length(menus, length.wrapping_sub(1));
        name.add(length.wrapping_sub(1) as u32)
            .write(&[0], "a player's name")?;
    }
    Ok(())
}
