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

pub fn kept() -> *mut u8 {
    NAME_BUFFERS.kept_name.addr() as *mut u8
}

/// The current player's name.
pub fn name() -> Result<*mut u8, Refused> {
    let name = unsafe { current()?.read_unaligned() }.name;
    if name == 0 {
        return Err(BadInput("a player without a name").into());
    }
    Ok(name as *mut u8)
}

pub unsafe fn bring_back_name(name: *mut u8) {
    unsafe { std::ptr::copy_nonoverlapping(kept(), name, TEXT_LEN) };
}

pub unsafe fn backspace_name(name: *mut u8) {
    let length = length();
    if length != 0 {
        set_length(length.wrapping_sub(1));
        unsafe { name.offset(length.wrapping_sub(1) as isize).write(0) };
    }
}
