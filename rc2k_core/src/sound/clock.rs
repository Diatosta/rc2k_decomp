//! The clock of the race's sounds.

/// The race clock's fields, in the game's order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RaceClock {
    /// When the last turn was (milliseconds).
    pub last: u32,
    pub turns_a_second: f32,
    /// The last turn's milliseconds, at most 1000.
    pub step: u32,
    pub turns: u32,
}

/// The least time between turns, in milliseconds.
pub(crate) const TURN_MS: u32 = 20;

impl RaceClock {
    /// Set going at `now`: 50 turns a second, none counted (the step left as it was).
    pub fn start(&mut self, now: u32) {
        self.last = now;
        self.turns_a_second = 50.0;
        self.turns = 0;
    }

    /// Whether a turn is due at `now` (20 ms or more since the last, unsigned): it then becomes the
    /// last, its milliseconds held to 1000, and the turns a second are 1000 / them (whole).
    pub fn step(&mut self, now: u32) -> bool {
        let passed = now.wrapping_sub(self.last);
        if passed < TURN_MS {
            return false;
        }
        self.last = self.last.wrapping_add(passed);
        self.step = passed.min(1000);
        self.turns_a_second = (1000 / self.step) as f32;
        self.turns = self.turns.wrapping_add(1);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turns() {
        let mut c = RaceClock {
            last: 0,
            turns_a_second: 0.0,
            step: 7,
            turns: 9,
        };
        c.start(100);
        assert_eq!(
            c,
            RaceClock {
                last: 100,
                turns_a_second: 50.0,
                step: 7,
                turns: 0
            }
        );
        assert!(!c.step(119));
        assert!(c.step(130));
        assert_eq!(
            (c.last, c.step, c.turns_a_second, c.turns),
            (130, 30, 33.0, 1)
        );
        assert!(c.step(5000));
        assert_eq!((c.step, c.turns_a_second), (1000, 1.0));
    }
}
