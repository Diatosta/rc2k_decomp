//! Which cars the views show: by the players, a network game (this machine's car only) and a
//! playing replay's layout (the replay frame head's byte +5).

use crate::Invalid;

/// What the choice reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShownSetup {
    /// The players (views).
    pub players: u32,
    /// The replay's mode (2 playing: the views are laid out by `layout`).
    pub replay_mode: u32,
    /// The playing replay's layout (1 .. 5), the replay frame head's byte +5: 1 one
    /// view of car 1, 2 two views with split 1 (side by side), 3 two views with split 2; 4 and 5
    /// (a replay with the extra tape) one view of the third or the fourth car.
    pub layout: u32,
    /// A network game on (non-zero) (only this machine's car is shown).
    pub network: u32,
    /// This machine's car in a network game.
    pub local_car: u32,
    /// A replay of the own format: the players' cars, by player (the layouts name players, not
    /// cars); with none the player is the car of the same number.
    pub player_cars: [u32; 8],
    /// How many of `player_cars` are set (0: the player is the car).
    pub player_count: u32,
    /// The cars whose sounds are all heard (each listed by its own number), the first so many: a
    /// replay of more than two cars; 0 for the default choice.
    pub heard_cars: u32,
}

/// What it sets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShownCars {
    /// The cars the two views show.
    pub cars: [u32; 2],
    /// The views.
    pub views: u32,
    /// The split (set only by a layout).
    pub split: u32,
    /// The two cars' blocks.
    pub blocks: [u32; 2],
    /// Each car's view (-1: not shown).
    pub listed: [u32; 8],
    /// The first car of the group of players a layout splits the screen between, 0 for none:
    /// the further views (the third, the fourth) show the cars after it.
    pub split_first: u32,
}

pub use crate::race::replay_v2::SPLIT_LAYOUT;

/// A car block's size.
pub const BLOCK: u32 = 0xFE0;

impl ShownCars {
    /// The shown cars set with the blocks from `first_block` on. Gives back this machine's car
    /// (network game) or the second view's block. Refused before anything: a car past 7 (it
    /// would write past the eight blocks).
    pub fn choose(&mut self, s: &ShownSetup, first_block: u32) -> Result<u32, Invalid> {
        let network = s.network != 0;
        if network && s.local_car > 7 {
            return Err(Invalid("a network car above 7 (past the eight)"));
        }
        self.cars = [0, 0];
        // A split screen of three or four players has a view for each (their cars the players' own).
        self.views = s.players.min(4);
        if self.views != 1 {
            self.cars[1] = 1;
        }
        // The car of player `p`.
        let car = |p: u32| {
            if s.player_count == 0 {
                p
            } else {
                s.player_cars
                    .get(p as usize)
                    .filter(|_| p < s.player_count)
                    .copied()
                    .unwrap_or(p)
            }
        };
        use crate::race::layouts::{Shown, shown};
        // The groups and pairs of players a layout splits the screen between, if it is one (a
        // replay of the first format has no player count: its layouts are named below).
        let parts = u8::try_from(s.layout)
            .ok()
            .and_then(|l| shown(l, s.player_count as usize))
            .filter(|shown| !matches!(shown, Shown::Player(_)));
        let group = parts.filter(|shown| matches!(shown, Shown::Group { .. }));
        let split_replay = s.replay_mode == 2 && group.is_some();
        self.split_first = 0;
        if s.replay_mode == 2 {
            let (views, first, second, split) = match (parts, s.layout) {
                // A group of players, each in its part: as in the race.
                (Some(Shown::Group { first, count }), _) => (
                    count as u32,
                    car(first as u32),
                    Some(car(first as u32 + 1)),
                    0,
                ),
                // Two players, side by side (split 1) or one above the other (split 2).
                (Some(Shown::Pair { first, stacked }), _) => (
                    2,
                    car(first as u32),
                    Some(car(first as u32 + 1)),
                    if stacked { 2 } else { 1 },
                ),
                (_, 1) => (1, car(1), None, 0),
                (_, 2) => (2, car(0), Some(car(1)), 1),
                (_, 3) => (2, car(0), Some(car(1)), 2),
                // A replay with the extra tape: the third player's car alone, the fourth's ...
                (_, l @ 4..=9) => (1, car(l - 2), None, 0),
                _ => (1, car(0), None, 0),
            };
            if let Some(Shown::Group { first, .. }) = group {
                self.split_first = first as u32;
            }
            self.views = views;
            self.cars[0] = first;
            if let Some(c) = second {
                self.cars[1] = c;
            }
            self.split = split;
        }
        // A replay playing shows the cars its layout names, whether or not the link is still up.
        let network = network && s.replay_mode != 2;
        if network {
            self.cars = [s.local_car, 0];
        }
        self.blocks = self
            .cars
            .map(|c| first_block.wrapping_add(c.wrapping_mul(BLOCK)));
        self.listed = [u32::MAX; 8];
        // Each shown car's view (the cars are below eight: a layout names 0 or 1).
        self.listed[self.cars[0] as usize] = 0;
        if self.views != 1 && !network {
            self.listed[self.cars[1] as usize] = 1;
        }
        if (s.replay_mode != 2 || split_replay) && !network {
            for view in 2..self.views {
                self.listed[car(self.split_first + view) as usize] = view;
            }
        }
        // Every car of a replay of more than two is heard: each is its own sound object's.
        for car in 0..s.heard_cars.min(8) {
            self.listed[car as usize] = car;
        }
        Ok(if network { s.local_car } else { self.blocks[1] })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choices() {
        let mut c = ShownCars::default();
        let two = ShownSetup {
            players: 2,
            ..Default::default()
        };
        assert_eq!(c.choose(&two, 0x1000), Ok(0x1000 + BLOCK));
        assert_eq!(
            (c.cars, c.views, c.listed[..3].to_vec()),
            ([0, 1], 2, vec![0, 1, u32::MAX])
        );
        // Three and four players: a view each, the further ones showing their own player's car.
        let four = ShownSetup {
            players: 4,
            ..Default::default()
        };
        c.choose(&four, 0).unwrap();
        assert_eq!((c.cars, c.views), ([0, 1], 4));
        assert_eq!(c.listed[..5].to_vec(), vec![0, 1, 2, 3, u32::MAX]);
        let laid = ShownSetup {
            players: 2,
            replay_mode: 2,
            layout: 1,
            ..Default::default()
        };
        c.choose(&laid, 0).unwrap();
        assert_eq!((c.cars, c.views, c.split), ([1, 1], 1, 0));
        let third = ShownSetup {
            players: 3,
            replay_mode: 2,
            layout: 4,
            ..Default::default()
        };
        c.choose(&third, 0).unwrap();
        assert_eq!(
            (c.cars[0], c.views, c.listed[2], c.listed[0]),
            (2, 1, 0, u32::MAX)
        );
        c.choose(&ShownSetup { layout: 5, ..third }, 0).unwrap();
        assert_eq!((c.cars[0], c.views, c.listed[3]), (3, 1, 0));
        let network = ShownSetup {
            players: 1,
            network: 1,
            local_car: 5,
            ..Default::default()
        };
        assert_eq!(c.choose(&network, 0), Ok(5));
        assert_eq!((c.cars, c.listed[5], c.listed[0]), ([5, 0], 0, u32::MAX));
        // A replay of a network race, played while the link is up: its layout wins.
        let replayed = ShownSetup {
            replay_mode: 2,
            layout: 1,
            ..network
        };
        assert_eq!(c.choose(&replayed, 0), Ok(0));
        assert_eq!(
            (c.cars, c.views, c.listed[1], c.listed[5]),
            ([1, 0], 1, 0, u32::MAX)
        );
        // A replay of the own format names players: the players are cars 1, 3 and 4, the rest are
        // the computer's.
        let own = ShownSetup {
            players: 1,
            replay_mode: 2,
            player_cars: [1, 3, 4, 0, 0, 0, 0, 0],
            player_count: 3,
            ..Default::default()
        };
        for (layout, cars, views) in [
            (0, [1, 3], 1),
            (1, [3, 3], 1),
            (2, [1, 3], 2),
            (4, [4, 3], 1),
        ] {
            c.choose(&ShownSetup { layout, ..own }, 0).unwrap();
            assert_eq!((c.cars[0], c.views), (cars[0], views), "layout {layout}");
            if views == 2 {
                assert_eq!(c.cars[1], cars[1]);
            }
        }
        // A local split-screen race's replay in its layout: a view for each player, each car heard.
        let split = ShownSetup {
            players: 4,
            replay_mode: 2,
            layout: SPLIT_LAYOUT,
            player_cars: [0, 1, 2, 3, 0, 0, 0, 0],
            player_count: 4,
            heard_cars: 6,
            ..Default::default()
        };
        c.choose(&split, 0).unwrap();
        assert_eq!((c.cars, c.views, c.split), ([0, 1], 4, 0));
        assert_eq!(c.listed[..7].to_vec(), vec![0, 1, 2, 3, 4, 5, u32::MAX]);
        // Eight players: the second group shows the last four, a view each; the first the first four.
        let eight = ShownSetup {
            players: 1,
            replay_mode: 2,
            layout: 11,
            player_cars: [0, 1, 2, 3, 4, 5, 6, 7],
            player_count: 8,
            ..Default::default()
        };
        c.choose(&eight, 0).unwrap();
        assert_eq!((c.cars, c.views, c.split_first), ([4, 5], 4, 4));
        assert_eq!(c.listed[3..].to_vec(), vec![u32::MAX, 0, 1, 2, 3]);
        c.choose(
            &ShownSetup {
                layout: 10,
                ..eight
            },
            0,
        )
        .unwrap();
        assert_eq!((c.cars, c.views, c.split_first), ([0, 1], 4, 0));
        assert_eq!(c.listed[..5].to_vec(), vec![0, 1, 2, 3, u32::MAX]);
        // Six players: layout 11 is the fifth and the sixth as a pair, side by side (split 1), 12
        // the same one above the other (split 2): two views, no group.
        for (layout, split) in [(11, 1), (12, 2)] {
            c.choose(
                &ShownSetup {
                    layout,
                    player_count: 6,
                    ..eight
                },
                0,
            )
            .unwrap();
            assert_eq!(
                (c.cars, c.views, c.split, c.split_first),
                ([4, 5], 2, split, 0)
            );
        }
        // The pair takes the players' own cars when the players are not the first cars.
        c.choose(
            &ShownSetup {
                layout: 12,
                player_count: 6,
                player_cars: [0, 1, 2, 3, 6, 7, 0, 0],
                ..eight
            },
            0,
        )
        .unwrap();
        assert_eq!((c.cars, c.views, c.split), ([6, 7], 2, 2));
        // A layout the players do not have is none: the first car alone.
        c.choose(
            &ShownSetup {
                layout: 11,
                player_count: 5,
                ..eight
            },
            0,
        )
        .unwrap();
        assert_eq!((c.cars[0], c.views, c.split_first), (0, 1, 0));
        // The layout names nothing with fewer than three players.
        let two = ShownSetup {
            player_count: 2,
            ..split
        };
        c.choose(&two, 0).unwrap();
        assert_eq!(c.views, 1);
        let bad = ShownSetup {
            local_car: 8,
            ..network
        };
        assert!(c.choose(&bad, 0).is_err());
    }
}
