//! The computer drivers of a network race ("bots"): the host's lobby setting, how it travels
//! with the host's record, how many cars it adds, which cars they are and how fast they drive.
//!
//! The setting (0 = off, 1 ..= 7) is kept in bits 1 ..= 3 of the host's record's direction byte
//! (bit 0 is the stage's direction), so it reaches every machine with the other lobby settings.
//! The bots take the car places after the players': with `players` players, bot `k` is car
//! `players + k`. Every machine drives them with the game's own computer driver, from the same
//! start; the host's position and extra messages for them (`Message::BotRace`) keep the
//! machines' copies together as the players' own do.

/// The most bots a setting can ask for.
pub const MOST_SETTING: u32 = 7;
/// The cars a race holds.
pub const MOST_CARS: u32 = 8;

const SHIFT: u32 = 1;
const MASK: u8 = 0b1110;

/// The bots' setting held in a record's direction byte.
pub fn setting(direction: u8) -> u32 {
    u32::from((direction & MASK) >> SHIFT)
}

/// The direction byte `direction` with the bots' setting `setting` (held to `MOST_SETTING`).
pub fn with_setting(direction: u8, setting: u32) -> u8 {
    (direction & !MASK) | ((setting.min(MOST_SETTING) as u8) << SHIFT)
}

/// The direction byte with the stage's direction (its bit 0) changed to `direction`'s and the
/// bots' setting kept.
pub fn with_direction(held: u8, direction: u8) -> u8 {
    (held & MASK) | (direction & 1)
}

/// The setting after `setting` when the host presses the button: one more, then off again.
pub fn next_setting(setting: u32) -> u32 {
    if setting >= MOST_SETTING {
        0
    } else {
        setting + 1
    }
}

/// The bots a race has: the setting, held to the places the players leave free.
pub fn count(setting: u32, players: u32) -> u32 {
    setting.min(MOST_CARS.saturating_sub(players))
}

/// Whether car `car` is a bot's in a race of `players` players and `bots` bots.
pub fn is_bot(car: u32, players: u32, bots: u32) -> bool {
    car >= players && car < players + bots
}

/// The kinds of the bots' cars: `candidates` (the kinds a bot may drive, in a fixed order the
/// same on every machine) taken one after another from a place that depends on the host's own
/// car kind, round again when there are fewer than `bots`.
pub fn kinds(candidates: &[u32], host_kind: u32, bots: u32) -> Vec<u32> {
    if candidates.is_empty() {
        return Vec::new();
    }
    let start = host_kind as usize % candidates.len();
    (0..bots as usize)
        .map(|k| candidates[(start + k) % candidates.len()])
        .collect()
}

/// The share of the computer driver's pace bot `k` has (see `start_blend`): the first bot the
/// fastest, each next a little slower, so that the bots do not all drive alike.
pub fn pace_share(k: u32) -> f32 {
    (0.95 - 0.04 * k as f32).max(0.6)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_setting_travels_in_the_direction_byte() {
        assert_eq!(setting(0), 0);
        assert_eq!(setting(1), 0);
        assert_eq!(setting(with_setting(1, 5)), 5);
        assert_eq!(with_setting(1, 5) & 1, 1);
        assert_eq!(setting(with_setting(0xFF, 3)), 3);
        assert_eq!(with_setting(0, 9), with_setting(0, 7));
        assert_eq!(with_direction(with_setting(0, 4), 1), with_setting(1, 4));
        assert_eq!(with_direction(with_setting(1, 4), 0), with_setting(0, 4));
        assert_eq!(setting(with_direction(with_setting(0, 6), 1)), 6);
    }

    #[test]
    fn the_button_cycles() {
        let mut n = 0;
        let mut seen = Vec::new();
        for _ in 0..9 {
            n = next_setting(n);
            seen.push(n);
        }
        assert_eq!(seen, [1, 2, 3, 4, 5, 6, 7, 0, 1]);
    }

    #[test]
    fn the_bots_fill_only_free_places() {
        assert_eq!(count(0, 2), 0);
        assert_eq!(count(3, 2), 3);
        assert_eq!(count(7, 2), 6);
        assert_eq!(count(7, 8), 0);
        assert_eq!(count(7, 9), 0);
        assert!(is_bot(2, 2, 3) && is_bot(4, 2, 3));
        assert!(!is_bot(1, 2, 3) && !is_bot(5, 2, 3));
    }

    #[test]
    fn the_kinds_are_the_same_everywhere() {
        let candidates = [4, 9, 12];
        assert_eq!(kinds(&candidates, 0, 2), vec![4, 9]);
        assert_eq!(kinds(&candidates, 7, 4), vec![9, 12, 4, 9]);
        assert!(kinds(&[], 3, 2).is_empty());
        assert!(pace_share(0) > pace_share(1) && pace_share(7) >= 0.6);
    }
}
