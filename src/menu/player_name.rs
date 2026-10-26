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
use crate::menu::names_store::NAME_BUFFERS;
use crate::menu::player_settings::current;
use crate::menu::text_entry::{TEXT_LEN, length, set_length};
use crate::pointers::GamePtr;

pub fn kept() -> u32 {
    NAME_BUFFERS.kept_name.addr()
}

/// The current player's name.
pub fn name() -> Result<u32, Refused> {
    let name = GamePtr::new(current()?)
        .read::<rc2k_core::menu::player::Player>("a player")?
        .name;
    if name == 0 {
        return Err(BadInput("a player without a name").into());
    }
    Ok(name)
}

pub fn bring_back_name(name: u32) -> Result<(), BadInput> {
    let kept = GamePtr::new(kept()).read_bytes(TEXT_LEN, "the kept name")?;
    GamePtr::new(name).write_bytes(&kept, "a player\'s name")
}

pub fn backspace_name(name: u32) -> Result<(), BadInput> {
    let length = length();
    if length != 0 {
        set_length(length.wrapping_sub(1));
        GamePtr::new(name.wrapping_add(length.wrapping_sub(1) as u32))
            .write::<u8>(0, "a player\'s name")?;
    }
    Ok(())
}
