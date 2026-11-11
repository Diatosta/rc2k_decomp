//! The four players' names and the players that keep a copy of them.

use crate::menu::owner::Menus;
use crate::menu::owner::Play;

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

/// A place in the names' area (the four names, then what follows them): a name's start, or
/// a byte on in it. Reads run to the area's end, as the original runs on past a name that
/// lacks its end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NameAt(usize);

impl NameAt {
    /// Player `player`'s name.
    pub fn of(player: usize) -> Self {
        Self(player * NAME_LEN)
    }

    /// `n` bytes on.
    pub fn add(self, n: u32) -> Self {
        Self(self.0.wrapping_add(n as usize))
    }

    /// The bytes from here to the end of the area (none past it).
    pub fn tail(self) -> Vec<u8> {
        let names = &crate::menu::owner::menus().play.names;
        (self.0..crate::place::Run::len(names))
            .map(|n| crate::place::Run::get(names, n))
            .collect()
    }
}

impl From<NameAt> for crate::place::BytePlace {
    fn from(at: NameAt) -> Self {
        crate::place::BytePlace::In(&crate::menu::owner::menus().play.names, at.0)
    }
}

/// Player `player`'s name replaced.
pub fn set_name_bytes(menus: &Menus, player: usize, name: &[u8; NAME_LEN]) {
    for (i, &b) in name.iter().enumerate() {
        menus.play.names.player_names.set(player * NAME_LEN + i, b);
    }
}

/// Player `player`'s name, the whole 0x20 bytes.
pub fn name_bytes(menus: &Menus, player: usize) -> [u8; NAME_LEN] {
    std::array::from_fn(|i| menus.play.names.player_names.get(player * NAME_LEN + i))
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
