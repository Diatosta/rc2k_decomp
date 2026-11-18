//! The computer drivers of a network race ("bots"): the host's lobby setting, how it travels
//! with the host's record, how many cars it adds, which cars they are and how fast they drive.
//!
//! The setting (0 = off, 1 ..= 7) is kept in bits 1 ..= 3 of the host's record's direction byte
//! (bit 0 is the stage's direction), so it reaches every machine with the other lobby settings.
//! Bit 4 holds the way the bots start: on the grid with the players (clear), or spread on the
//! course ahead of and behind them as the arcade's opponents are (set, `arcade_places`).
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
const ARCADE_BIT: u8 = 0b1_0000;

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
    (held & !1) | (direction & 1)
}

/// Whether the bots start spread on the course like the arcade's opponents (bit 4 of the
/// host's direction byte), not on the grid.
pub fn arcade_start(direction: u8) -> bool {
    direction & ARCADE_BIT != 0
}

/// The direction byte `direction` with the way the bots start set to `arcade`.
pub fn with_arcade_start(direction: u8, arcade: bool) -> u8 {
    (direction & !ARCADE_BIT) | if arcade { ARCADE_BIT } else { 0 }
}

/// Players whose checkpoints lie within this many of the group's lowest share their bots.
pub const NEAR: u32 = 40;
/// The checkpoints between one bot and the next, ahead of or behind a group of players.
pub const SPREAD: u32 = 20;

/// Where each of `bots` bots starts in the arcade way, as a checkpoint, given the checkpoint of
/// each player and the stage's `checkpoint_count`. The players are grouped: sorted by
/// checkpoint, a player within `NEAR` of the lowest of the group before joins it, else begins a
/// new group (at the race's start every player is in one group, since the grid is one place).
/// The bots are dealt to the groups one after another; within a group they stand alternately
/// ahead of the group's lowest checkpoint and behind it, `SPREAD` apart, and ahead only where
/// there is no road behind. No bot goes past the stage's last checkpoints.
pub fn arcade_places(players: &[u32], bots: u32, checkpoint_count: u32) -> Vec<u32> {
    let mut sorted = players.to_vec();
    sorted.sort_unstable();
    let mut bases: Vec<u32> = Vec::new();
    for c in sorted {
        if bases.last().is_none_or(|&b| c - b > NEAR) {
            bases.push(c);
        }
    }
    if bases.is_empty() {
        bases.push(0);
    }
    let last = checkpoint_count.saturating_sub(2);
    let mut ahead = vec![0u32; bases.len()];
    let mut behind = vec![0u32; bases.len()];
    (0..bots as usize)
        .map(|k| {
            let g = k % bases.len();
            let base = bases[g];
            let turn = k / bases.len();
            let back = (behind[g] + 1) * SPREAD;
            if turn % 2 == 1 && base >= back {
                behind[g] += 1;
                base - back
            } else {
                ahead[g] += 1;
                (base + ahead[g] * SPREAD).min(last)
            }
        })
        .collect()
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
    fn the_way_to_start_rides_beside_the_setting() {
        let d = with_arcade_start(with_setting(1, 5), true);
        assert!(arcade_start(d));
        assert_eq!((setting(d), d & 1), (5, 1));
        assert_eq!(with_direction(d, 0) & 1, 0);
        assert!(arcade_start(with_direction(d, 0)));
        assert_eq!(setting(with_arcade_start(d, false)), 5);
        assert!(!arcade_start(with_arcade_start(d, false)));
        assert!(!arcade_start(with_setting(0xF, 7) & !ARCADE_BIT));
    }

    #[test]
    fn the_arcade_places_follow_the_players() {
        // Players together at the start: one group, no road behind, all ahead.
        assert_eq!(arcade_places(&[2, 3], 3, 500), vec![22, 42, 62]);
        // Players apart: the bots are dealt to each group, behind where there is road.
        assert_eq!(arcade_places(&[2, 300], 2, 500), vec![22, 320]);
        assert_eq!(arcade_places(&[100], 2, 500), vec![120, 80]);
        assert_eq!(arcade_places(&[100, 400], 4, 500), vec![120, 420, 80, 380]);
        // Near players share; the end of the stage holds the bots.
        assert_eq!(arcade_places(&[100, 130], 1, 500), vec![120]);
        assert_eq!(arcade_places(&[490], 2, 500), vec![498, 470]);
        assert!(arcade_places(&[], 0, 500).is_empty());
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
