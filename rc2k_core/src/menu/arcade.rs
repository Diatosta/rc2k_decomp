//! The arcade's levels: which are open, and what a level's end does
//! (from `menu::arcade_level_1` .. `arcade_level_5` 0x487CC9 .. 0x487DE5,
//! `menu::arcade_level_end` 0x48AD73 and `menu::arcade_finale` 0x488155). A level's
//! "opened" byte is `Arcade.levels_won[level - 1]`; winning a level opens the next
//! (`Arcade.levels_won[level]`).

use crate::Invalid;

/// The arcade's levels, 1 .. 5.
pub const LEVELS: u32 = 5;

/// The cheat "Unlock Arcade": bit 0 of the cheats.
pub const UNLOCK_ARCADE: u32 = 1;

/// The menu's sound for a level not open.
pub const CLOSED_SOUND: u32 = 8;

/// A level item: open with the cheat on, or with its opened byte (`opened`, asked only
/// without the cheat) 1.
pub fn level_open(cheats: u32, opened: impl FnOnce() -> u8) -> bool {
    cheats & UNLOCK_ARCADE != 0 || opened() == 1
}

/// `menu::arcade_level_end`: the level was won (`Results.positions[0]`, the place, 1).
pub fn won(place: u32) -> bool {
    place == 1
}

/// The won level, whose next level's byte is marked. Refused: a level past 5 (the byte
/// past the table).
pub fn won_level(level: u32) -> Result<u32, Invalid> {
    if level > LEVELS {
        return Err(Invalid("an arcade level past 5"));
    }
    Ok(level)
}

/// Won the first time: the next level's byte still 0 (then it is made 1 and the secret
/// car shown).
pub fn first_win(next_opened: u8) -> bool {
    next_opened == 0
}

/// `menu::arcade_finale`: the levels left after `level` (`RaceState.results.stage_index`, the finale's
/// drive), wrapping as the original's `sub`.
pub fn levels_left(level: u32) -> u32 {
    LEVELS.wrapping_sub(level)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_open_and_end() {
        assert!(level_open(1, || unreachable!()));
        assert!(level_open(0x8000_0001, || unreachable!()));
        assert!(level_open(2, || 1));
        assert!(!level_open(2, || 2));
        assert!(!level_open(0, || 0));
        assert!(won(1) && !won(0) && !won(2));
        assert_eq!(won_level(5), Ok(5));
        assert!(won_level(6).is_err());
        assert!(first_win(0) && !first_win(1));
        assert_eq!(levels_left(1), 4);
        assert_eq!(levels_left(6), u32::MAX);
    }
}
