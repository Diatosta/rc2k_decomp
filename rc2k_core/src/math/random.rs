//! The game's random numbers (RAL.EXE 0x401864 .. 0x4018C8): a 32-bit shift register
//! stepped once a number, seeded at start from the clock and stirred with a number the
//! start finds (from `math::next_random`, `random_below`,
//! `seed_random`, `stir_random`, `random_step`).

/// The generator.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Random {
    pub state: u32,
}

/// The clock at the start, as the start keeps it.
#[derive(Clone, Copy, Debug, Default)]
pub struct StartClock {
    /// Hour, minute, second, hundredths.
    pub time: u32,
    /// Day, month, year.
    pub date: u32,
    /// The day of the week.
    pub weekday: u8,
}

/// One step of the generator from `state`, `mixed_in` mixed into the result.
pub fn step(state: u32, mixed_in: u32) -> u32 {
    let mut low = state as u8;
    low ^= (state & 0x40_0000 != 0) as u8;
    low ^= (low & 4 != 0) as u8;
    low ^= (low & 2 != 0) as u8;
    (state & !0xFF | low as u32).rotate_right(1) ^ mixed_in
}

impl Random {
    /// The next number (the state moved on).
    pub fn draw(&mut self) -> u32 {
        self.state = step(self.state, 0);
        self.state
    }

    /// A number under `limit`; `None` for a limit of 0 (the original divides by it).
    pub fn below(&mut self, limit: u32) -> Option<u32> {
        (limit != 0).then(|| self.draw() % limit)
    }

    /// The state from the start's clock and date, turned by the day of the week (left as it
    /// is when that step gives 0).
    pub fn seed(&mut self, clock: StartClock) {
        let seed = step(clock.time, clock.date);
        if seed != 0 {
            let turns = clock.weekday.wrapping_add(8) as u32;
            self.state = seed.rotate_left(turns & 31);
        }
    }

    /// `number` stirred into the state (left as it is when that gives 0).
    pub fn stir(&mut self, number: u32) {
        let stirred = step(number, self.state);
        if stirred != 0 {
            self.state = stirred;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_step_shifts_and_feeds_back() {
        assert_eq!(step(0, 0), 0);
        // bit 22 set: the low bit fed back, then the whole turned right
        assert_eq!(step(0x40_0000, 0), (0x40_0001u32).rotate_right(1));
        assert_eq!(step(1, 0xFFFF_FFFF), 1u32.rotate_right(1) ^ 0xFFFF_FFFF);
    }

    #[test]
    fn a_draw_moves_the_state() {
        let mut r = Random { state: 0x1234_5678 };
        let a = r.draw();
        assert_eq!(a, step(0x1234_5678, 0));
        assert_eq!(r.state, a);
        assert_eq!(r.draw(), step(a, 0));
    }

    #[test]
    fn below_stays_under_the_limit() {
        let mut r = Random { state: 0xDEAD_BEEF };
        for limit in [1, 2, 7, 100, 0xFFFF_FFFF] {
            assert!(r.below(limit).unwrap() < limit);
        }
        assert_eq!(r.below(0), None);
    }

    #[test]
    fn seed_and_stir_leave_a_zero_step_alone() {
        let mut r = Random { state: 5 };
        r.seed(StartClock::default());
        assert_eq!(r.state, 5);
        let clock = StartClock {
            time: 0x0C1E_2D00,
            date: 0x07D0_0A14,
            weekday: 3,
        };
        r.seed(clock);
        let expected = step(clock.time, clock.date).rotate_left(11);
        assert_eq!(r.state, expected);
        r.stir(0);
        assert_eq!(r.state, step(0, expected));
    }
}
