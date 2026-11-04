//! The four players' names and the players that keep a copy of them.

use crate::menu::names_store::NAME_BUFFERS;
use crate::menu::players_store::PLAYER_TABLE;

pub const PLAYERS: usize = 4;
pub const NAME_LEN: usize = 0x20;

/// Where player `player`'s name is (the address the cars keep).
pub fn name(player: usize) -> u32 {
    NAME_BUFFERS.player_names.addr() + (player * NAME_LEN) as u32
}

/// Which player's name a player record's name pointer (`Player::name`) names; None for an
/// address that is no name's start.
pub fn index_of(name_at: u32) -> Option<usize> {
    (0..PLAYERS).find(|&k| name(k) == name_at)
}

/// Player `player`'s name as a pointer bounded by the names' store (the original runs on
/// past a name that lacks its end).
pub fn name_ptr(player: usize) -> crate::pointers::GamePtr {
    crate::pointers::GamePtr::to(&NAME_BUFFERS).offset(
        (std::mem::offset_of!(crate::menu::names_store::Names, player_names) + player * NAME_LEN)
            as u32,
    )
}

/// Player `player`'s name, the whole 0x20 bytes.
/// The player's name whose start a record's name pointer is; refused for any other
/// address.
pub fn name_index(name_at: u32) -> Result<usize, crate::errors::BadInput> {
    index_of(name_at).ok_or(crate::errors::BadInput(
        "a name pointer off the players' names",
    ))
}

/// Player `player`'s name replaced.
pub fn set_name_bytes(player: usize, name: &[u8; NAME_LEN]) {
    for (i, &b) in name.iter().enumerate() {
        NAME_BUFFERS.player_names.set(player * NAME_LEN + i, b);
    }
}

pub fn name_bytes(player: usize) -> [u8; NAME_LEN] {
    std::array::from_fn(|i| NAME_BUFFERS.player_names.get(player * NAME_LEN + i))
}

/// The four players' records.
pub fn cars() -> &'static [crate::session::Shared<crate::menu::player::Player>; 4] {
    &PLAYER_TABLE.players
}
