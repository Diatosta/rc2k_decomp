//! The race's sound turn: when one is due, which cars are heard
//! (from `sound::race_sound_tick` 0x421BA9, `sound::car_sounds_all` 0x40D7AB's choice and
//! `sound::play_cue_queues` 0x40ED13's).

/// `sound::race_sound_tick`: a turn is due once 20 ms (0x14) of the scene clock have gone
/// since the last (unsigned, so a clock gone back counts as far on).
pub fn tick_due(clock: u32, last: u32) -> bool {
    clock.wrapping_sub(last) >= 0x14
}

/// `sound::play_cue_queues`: the cars whose spoken cues are played, the first player's
/// and, unless there is one player (`one_player` 1), the second's (asked only then).
pub fn cue_cars(first: u32, second: impl FnOnce() -> u32, one_player: u32) -> Vec<u32> {
    let mut cars = vec![first];
    if one_player != 1 {
        cars.push(second());
    }
    cars
}

/// No car (-1).
pub const NO_CAR: u32 = u32::MAX;
/// A float's +infinity, as the original's starting distance.
pub const FAR: u32 = 0x7F80_0000;
/// The marks the two computer's cars' objects carry (+0) while their sounds are made: the
/// nearest -2, the next -1.
pub const NEAREST_MARK: u32 = 0xFFFF_FFFE;
pub const NEXT_MARK: u32 = 0xFFFF_FFFF;

/// `sound::car_sounds_all`'s choice of the two computer's cars nearest the listener: the
/// squared distances are compared as integers (signed: the bits of a float that is not
/// negative).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Nearest {
    pub first: u32,
    pub second: u32,
    pub first_distance: u32,
    pub second_distance: u32,
}

impl Default for Nearest {
    fn default() -> Self {
        Nearest {
            first: NO_CAR,
            second: NO_CAR,
            first_distance: FAR,
            second_distance: FAR,
        }
    }
}

impl Nearest {
    /// Car `n` at squared distance `d` (a float's bits).
    pub fn consider(&mut self, n: u32, d: u32) {
        if self.first_distance as i32 > d as i32 {
            self.second_distance = self.first_distance;
            self.second = self.first;
            self.first_distance = d;
            self.first = n;
        } else if self.second_distance as i32 > d as i32 {
            self.second_distance = d;
            self.second = n;
        }
    }

    /// The nearest and the next for this turn: swapped when the nearest was last turn's
    /// next or the next last turn's nearest, so that each keeps its voice.
    pub fn kept(&self, last_nearest: u32, last_next: u32) -> (u32, u32) {
        if self.first == last_next || self.second == last_nearest {
            (self.second, self.first)
        } else {
            (self.first, self.second)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks_and_cues() {
        assert!(!tick_due(119, 100));
        assert!(tick_due(120, 100));
        assert!(tick_due(5, 100));
        assert_eq!(cue_cars(2, || unreachable!(), 1), vec![2]);
        assert_eq!(cue_cars(2, || 3, 0), vec![2, 3]);
    }

    #[test]
    fn nearest() {
        let mut n = Nearest::default();
        n.consider(4, 0x4000_0000);
        n.consider(5, 0x3F00_0000);
        n.consider(6, 0x4100_0000);
        n.consider(7, 0x3F80_0000);
        assert_eq!((n.first, n.second), (5, 7));
        assert_eq!(n.kept(1, 2), (5, 7));
        assert_eq!(n.kept(7, 2), (7, 5));
        assert_eq!(n.kept(1, 5), (7, 5));
    }
}
