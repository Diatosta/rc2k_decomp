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
}

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
        if s.replay_mode == 2 {
            let (views, first, second, split) = match s.layout {
                1 => (1, 1, None, 0),
                2 => (2, 0, Some(1), 1),
                3 => (2, 0, Some(1), 2),
                // A replay with the extra tape: the third or the fourth car alone.
                4 => (1, 2, None, 0),
                5 => (1, 3, None, 0),
                _ => (1, 0, None, 0),
            };
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
        if s.replay_mode != 2 && !network {
            for view in 2..self.views {
                self.listed[view as usize] = view;
            }
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
        let bad = ShownSetup {
            local_car: 8,
            ..network
        };
        assert!(c.choose(&bad, 0).is_err());
    }
}
