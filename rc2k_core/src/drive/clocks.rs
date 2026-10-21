//! The race's and the replay's clocks (converted 2026-10-21 from
//! `drive::race_clock_tick` 0x445554 and `replay_clock_tick` 0x460C20).

/// The race's clock in tenths (`dword_61383C` .. `dword_613844`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RaceClock {
    /// `dword_61383C`: the scene clock's last reading.
    pub now: u32,
    /// `dword_613840`: the time counted up to (whole tenths).
    pub counted: u32,
    /// `dword_613844`: the tenths of this tick.
    pub tenths: u32,
}

/// How far behind the counted time may fall (1000).
const MOST_BEHIND: u32 = 1000;

impl RaceClock {
    /// `drive::race_clock_tick` after the scene clock's reading `now`: the counted time
    /// brought within 1000 of it (by the sign of the difference), the whole tenths since
    /// counted. The tenths x 10 as `mul` leaves it (the original's eax and edx).
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

/// The replay's clock (`dword_5C2534` hundredths of `dword_5C2530` frames).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReplayClock {
    pub frame: u32,
    pub hundredths: u32,
}

impl ReplayClock {
    /// `drive::replay_clock_tick`: a hundredth on, at 100 (or past) the next frame.
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
