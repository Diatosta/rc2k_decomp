//! The race's sound objects: how many players and objects there are, and each object's number.

/// The race's sound objects.
pub const OBJECTS: u32 = 8;

/// Players with a sound object: one when the menu's single switch is 1, else two.
pub fn sound_players(single: u32) -> u32 {
    if single == 1 { 1 } else { 2 }
}

/// Objects in use: the players', or 4 in mode 2 (arcade), which also sets
/// `RaceSounds.mark_car_voices`. Returns the count and whether that is set.
pub fn sound_objects(players: u32, mode: u32) -> (u32, bool) {
    if mode == 2 {
        (4, true)
    } else {
        (players, false)
    }
}

/// An object's first dword before its owner is set: -1 ..= -8.
pub fn object_mark(n: u32) -> u32 {
    (-(n as i32) - 1) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts() {
        assert_eq!(sound_players(1), 1);
        assert_eq!(sound_players(0), 2);
        assert_eq!(sound_objects(2, 2), (4, true));
        assert_eq!(sound_objects(1, 0), (1, false));
        assert_eq!(object_mark(0), u32::MAX);
        assert_eq!(object_mark(7), (-8i32) as u32);
    }
}
