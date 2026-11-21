//! Whose name a best-time entry, a standings row or a rival carries: a player's name or a driver's
//! name string.
//!
//! The files (`hst.dat`, the saved games) keep a player's name as one of four fixed numbers,
//! and a driver's name as the number of its string. Those numbers are read and written here only (`from_file` / `to_file`); the program
//! works with the kind.

/// How many players have a name.
const PLAYERS: usize = 4;
/// The file's number of the first player's name.
const FIRST_PLAYER_NUMBER: u32 = 0x755784;
/// The file's numbers of the players' names are this far apart.
const PLAYER_STEP: u32 = 0x20;

/// A name: a player's (counted from 0) or a driver's string.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NameId {
    /// The name of player `n` (0 .. 3).
    Player(u8),
    /// The name string with this number.
    String(u32),
}

impl Default for NameId {
    /// String 0: what an emptied entry holds.
    fn default() -> Self {
        Self::String(0)
    }
}

impl NameId {
    /// The number a best-scores list ends with (-1).
    pub const LIST_END: Self = Self::String(u32::MAX);

    /// The name the file's number `number` stands for: a player's when it is one of the four
    /// player numbers, otherwise a string (whatever the number, so that `to_file` gives
    /// every file number back).
    pub const fn from_file(number: u32) -> Self {
        let mut player = 0;
        while player < PLAYERS {
            if number == FIRST_PLAYER_NUMBER + PLAYER_STEP * player as u32 {
                return Self::Player(player as u8);
            }
            player += 1;
        }
        Self::String(number)
    }

    /// The file's number for this name.
    pub const fn to_file(self) -> u32 {
        match self {
            Self::Player(player) => FIRST_PLAYER_NUMBER + PLAYER_STEP * player as u32,
            Self::String(number) => number,
        }
    }

    /// The name of player `player` (counted from 0).
    pub const fn of_player(player: usize) -> Self {
        Self::Player(player as u8)
    }

    /// Which player's name this is; None for a string.
    pub const fn player(self) -> Option<usize> {
        match self {
            Self::Player(player) => Some(player as usize),
            Self::String(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The four players' numbers are the files' numbers, and every number
    /// (the players', the strings', any other) goes through `from_file` and back unchanged.
    #[test]
    fn the_files_numbers_come_back() {
        assert_eq!(NameId::from_file(0x755784), NameId::Player(0));
        assert_eq!(NameId::from_file(0x7557C4), NameId::Player(2));
        assert_eq!(NameId::from_file(0x7557C5), NameId::String(0x7557C5));
        assert_eq!(NameId::from_file(0x7557E4), NameId::Player(3));
        assert_eq!(NameId::of_player(3).to_file(), 0x7557E4);
        for number in [
            0,
            1,
            0x5A6,
            0x755783,
            0x755784,
            0x7557A4,
            0x7557C4,
            0x7557E4,
            0x7557E5,
            u32::MAX,
        ] {
            assert_eq!(NameId::from_file(number).to_file(), number);
        }
        assert_eq!(NameId::from_file(0x7557A4).player(), Some(1));
        assert_eq!(NameId::String(7).player(), None);
    }
}
