//! The race's and the replay's clocks.

/// The race's clock in tenths.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RaceClock {
    /// The scene clock's last reading.
    pub now: u32,
    /// The time counted up to (whole tenths).
    pub counted: u32,
    /// The tenths of this tick.
    pub tenths: u32,
}

/// How far the counted time may fall behind the scene clock.
const MOST_BEHIND: u32 = 1000;

impl RaceClock {
    /// One tick at the scene clock's reading `now`: the counted time is brought within
    /// `MOST_BEHIND` of it (signed difference), then on by the whole tenths since. Gives back
    /// the tenths x 10 as a 64-bit product split (low, high).
    pub fn tick(&mut self, now: u32) -> (u32, u32) {
        self.now = now;
        if (now.wrapping_sub(self.counted).wrapping_sub(MOST_BEHIND) as i32) >= 0 {
            self.counted = now.wrapping_sub(MOST_BEHIND);
        }
        self.tenths = now.wrapping_sub(self.counted) / 10;
        let product = u64::from(self.tenths) * 10;
        self.counted = self.counted.wrapping_add(product as u32);
        (product as u32, (product >> 32) as u32)
    }
}

/// The replay's clock (`ReplayViewBlock.replay_frame` hundredths of `ReplayViewBlock.replay_block`
/// frames).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReplayClock {
    pub frame: u32,
    pub hundredths: u32,
}

impl ReplayClock {
    /// A hundredth on; at 100 or more, the next frame.
    pub fn tick(&mut self) {
        self.hundredths = self.hundredths.wrapping_add(1);
        if self.hundredths >= 100 {
            self.hundredths = 0;
            self.frame = self.frame.wrapping_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn race() {
        let mut c = RaceClock {
            counted: 100,
            ..Default::default()
        };
        assert_eq!(c.tick(125), (20, 0));
        assert_eq!((c.now, c.counted, c.tenths), (125, 120, 2));
        assert_eq!(c.tick(5000), (1000, 0));
        assert_eq!(c.counted, 5000);
        // A clock gone back: the unsigned difference counts as far on.
        assert_eq!(c.tick(4000), (0xFFFF_FC18 / 10 * 10, 0));
    }

    #[test]
    fn replay() {
        let mut r = ReplayClock {
            frame: 1,
            hundredths: 98,
        };
        r.tick();
        assert_eq!(
            r,
            ReplayClock {
                frame: 1,
                hundredths: 99
            }
        );
        r.tick();
        assert_eq!(
            r,
            ReplayClock {
                frame: 2,
                hundredths: 0
            }
        );
    }
}
