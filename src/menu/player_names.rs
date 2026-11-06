//! The four players' names and the players that keep a copy of them.

use crate::menu::owner::Play;
use crate::menu::owner::menus;

pub const PLAYERS: usize = 4;
pub const NAME_LEN: usize = 0x20;

/// The original's address of player `player`'s name (`g_playerNames`, IDA 0x755784 +
/// 0x20 player): what the saved games and the best-times entries carry for the name
/// (`Player::to_file_bytes`, the entries' `name_id`). A name, never read as memory: the
/// files get the original's bytes, and an entry kept in `var\hst.dat` still names its
/// player in a later run (the names' place in Rust's memory changes from run to run).
pub fn name_address(player: usize) -> u32 {
    let names = crate::menu::names_store::START
        + core::mem::offset_of!(crate::menu::names_store::Names, player_names);
    (names + player * NAME_LEN) as u32
}

/// Which player's name an entry's name address (`name_address`) names; None for an address
/// that is no name's start.
pub fn index_of(name_at: u32) -> Option<usize> {
    (0..PLAYERS).find(|&k| name_address(k) == name_at)
}

/// Whose name the player record `player` has; refused for a record without a name (the
/// original reads through its null pointer).
pub fn name_of(player: &rc2k_core::menu::player::Player) -> Result<usize, crate::errors::BadInput> {
    player
        .name_player()
        .ok_or(crate::errors::BadInput("a player without a name"))
}

/// Player `player`'s name as a pointer bounded by the names' store (the original runs on
/// past a name that lacks its end).
pub fn name_ptr(player: usize) -> crate::pointers::GamePtr {
    crate::pointers::GamePtr::to(&menus().play.names).offset(
        (std::mem::offset_of!(crate::menu::names_store::Names, player_names) + player * NAME_LEN)
            as u32,
    )
}

/// Player `player`'s name replaced.
pub fn set_name_bytes(player: usize, name: &[u8; NAME_LEN]) {
    for (i, &b) in name.iter().enumerate() {
        menus()
            .play
            .names
            .player_names
            .set(player * NAME_LEN + i, b);
    }
}

/// Player `player`'s name, the whole 0x20 bytes.
pub fn name_bytes(player: usize) -> [u8; NAME_LEN] {
    std::array::from_fn(|i| menus().play.names.player_names.get(player * NAME_LEN + i))
}

/// The four players' records.
pub fn cars(play: &Play) -> &'_ [crate::session::Shared<crate::menu::player::Player>; 4] {
    &play.players.players
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The names' addresses are the original's, the same in every run.
    #[test]
    fn name_addresses_are_the_originals() {
        assert_eq!(
            (0..PLAYERS).map(name_address).collect::<Vec<_>>(),
            [0x755784, 0x7557A4, 0x7557C4, 0x7557E4]
        );
        assert_eq!(index_of(0x7557C4), Some(2));
        assert_eq!(index_of(0x7557C5), None);
    }
}
