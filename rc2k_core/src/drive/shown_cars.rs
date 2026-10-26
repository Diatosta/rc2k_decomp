//! Which cars the views show (converted 2026-10-21 from `drive::choose_shown_cars`
//! 0x4560EA): the split screen's set-up by the players, the screen layout and a replay
//! (a reading).

use crate::Invalid;

/// What the choice reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShownSetup {
    /// `dword_63DB94`: the players (views).
    pub players: u32,
    /// `dword_5C5CE0`: 2 lays out by `layout`.
    pub screen_mode: u32,
    /// `dword_63DB88`: the layout (1 .. 3).
    pub layout: u32,
    /// `dword_52A910`: a replay on.
    pub replay: u32,
    /// `dword_63DBA0`: the replay's car.
    pub replay_car: u32,
}

/// What it sets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShownCars {
    /// `dword_63DBA8`, `dword_63DBAC`: the cars the two views show.
    pub cars: [u32; 2],
    /// `dword_63DB98`: the views.
    pub views: u32,
    /// `dword_6238A4`: the split (set only by a layout).
    pub split: u32,
    /// `off_63DBB0`, `off_63DBB4`: the two cars' blocks.
    pub blocks: [u32; 2],
    /// `dword_641680` ..: each car's view (-1: not shown).
    pub listed: [u32; 8],
}

/// A car block's size.
pub const BLOCK: u32 = 0xFE0;

impl ShownCars {
    /// `drive::choose_shown_cars`: the blocks from `first_block` on. The original's eax:
    /// the replay's car, or the second view's block. A replay car past 7 is refused
    /// before anything (it would write past the eight).
    pub fn choose(&mut self, s: &ShownSetup, first_block: u32) -> Result<u32, Invalid> {
        let replay = s.replay != 0;
        if replay && s.replay_car > 7 {
            return Err(Invalid("a replay car above 7 (past dword_641680's eight)"));
        }
        self.cars = [0, 0];
        self.views = s.players;
        if self.views != 1 {
            self.cars[1] = 1;
        }
        if s.screen_mode == 2 {
            let (views, first, second, split) = match s.layout {
                1 => (1, 1, None, 0),
                2 => (2, 0, Some(1), 1),
                3 => (2, 0, Some(1), 2),
                _ => (1, 0, None, 0),
            };
            self.views = views;
            self.cars[0] = first;
            if let Some(c) = second {
                self.cars[1] = c;
            }
            self.split = split;
        }
        if replay {
            self.cars = [s.replay_car, 0];
        }
        self.blocks = self
            .cars
            .map(|c| first_block.wrapping_add(c.wrapping_mul(BLOCK)));
        self.listed = [u32::MAX; 8];
        Ok(if replay {
            self.listed[s.replay_car as usize] = 0;
            s.replay_car
        } else {
            self.listed[0] = 0;
            if s.players != 1 {
                self.listed[1] = 1;
            }
            self.blocks[1]
        })
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
        let laid = ShownSetup {
            players: 2,
            screen_mode: 2,
            layout: 1,
            ..Default::default()
        };
        c.choose(&laid, 0).unwrap();
        assert_eq!((c.cars, c.views, c.split), ([1, 1], 1, 0));
        let replay = ShownSetup {
            players: 1,
            replay: 1,
            replay_car: 5,
            ..Default::default()
        };
        assert_eq!(c.choose(&replay, 0), Ok(5));
        assert_eq!((c.cars, c.listed[5], c.listed[0]), ([5, 0], 0, u32::MAX));
        let bad = ShownSetup {
            replay_car: 8,
            ..replay
        };
        assert!(c.choose(&bad, 0).is_err());
    }
}
