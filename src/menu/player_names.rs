//! The four players' names and the players that keep a copy of them.

use crate::menu::owner::Menus;
use crate::menu::owner::Play;

pub const PLAYERS: usize = 4;
pub const NAME_LEN: usize = 0x20;

/// Whose name the player record `player` has; refused for a record without a name.
pub fn name_of(player: &rc2k_core::menu::player::Player) -> Result<usize, crate::errors::BadInput> {
    player
        .name_player()
        .ok_or(crate::errors::BadInput("a player without a name"))
}

/// A place in the names' area (the four names, then what follows them): a name's start, or
/// a byte on in it. Reads run to the area's end, as a name that lacks its end is read on past
/// it.
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
    pub fn tail(self, play: &Play) -> Vec<u8> {
        let names = &play.slots.names;
        (self.0..crate::memory::place::Run::len(names))
            .map(|n| crate::memory::place::Run::get(names, n))
            .collect()
    }
}

impl NameAt {
    /// The place in `menus`' names' area.
    pub fn place(self, menus: &'static Menus) -> crate::memory::place::BytePlace {
        crate::memory::place::BytePlace::In(&menus.play.slots.names, self.0)
    }
}

/// Player `player`'s name replaced.
pub fn set_name_bytes(menus: &Menus, player: usize, name: &[u8; NAME_LEN]) {
    for (i, &b) in name.iter().enumerate() {
        menus
            .play
            .slots
            .names
            .player_names
            .set(player * NAME_LEN + i, b);
    }
}

/// Player `player`'s name, the whole 0x20 bytes.
pub fn name_bytes(menus: &Menus, player: usize) -> [u8; NAME_LEN] {
    std::array::from_fn(|i| {
        menus
            .play
            .slots
            .names
            .player_names
            .get(player * NAME_LEN + i)
    })
}

/// The four players' records.
pub fn cars(play: &Play) -> &'_ [crate::memory::session::Shared<crate::menu::player::Player>; 4] {
    &play.slots.records
}
