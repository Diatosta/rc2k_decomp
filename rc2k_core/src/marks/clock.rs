//! The marks' clock (converted 2026-10-21 from `marks::tick_clock` 0x4D2BDC): each frame
//! the scene clock's ticks since the last, a quarter and a sixteenth of them, one over
//! them and scaled. As in `math::geometry`, what the x87 keeps on its stack is an `f64`,
//! rounded to `f32` where the original stores it.

/// The clock's variables (`g_marksClockStarted` .. `g_marksClockBefore`, 0x28 bytes).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Clock {
    pub started: u32,
    pub last: u32,
    pub ticks: u32,
    /// The ticks' bits taken as a float (section 16) times a scale (`flt_764FD4`).
    pub ticks_scaled: f32,
    pub sixteenth: u32,
    pub quarter: u32,
    pub reciprocal: f32,
    pub reciprocal_scaled: f32,
    pub now: u32,
    pub before: u32,
}

/// The constants it uses: the first reciprocal (`flt_93CC10`), the reciprocal's scale
/// (`flt_7651BC`), the ticks' (`flt_93CC14`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scales {
    pub first: f32,
    pub reciprocal: f32,
    pub ticks: f32,
}

impl Clock {
    /// One frame at scene clock `clock`; `running` the settings' +0x10 is 1 (else no
    /// ticks). The first time only started, the clock kept and the first reciprocal.
    pub fn tick(&mut self, clock: u32, running: bool, k: &Scales) {
        if self.started == 0 {
            self.started = 1;
            [self.last, self.now, self.before] = [clock; 3];
            self.ticks = 0;
            self.reciprocal = k.first;
            return;
        }
        self.now = clock;
        self.before = self.last;
        self.last = clock;
        let ticks = if running {
            clock.wrapping_sub(self.before)
        } else {
            0
        };
        self.ticks = ticks;
        self.quarter = ticks >> 2;
        self.sixteenth = ticks >> 4;
        let one_over = 1.0 / ticks as i32 as f64;
        self.reciprocal = one_over as f32;
        self.reciprocal_scaled = (one_over * k.reciprocal as f64) as f32;
        self.ticks_scaled = (f32::from_bits(ticks) as f64 * k.ticks as f64) as f32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock() {
        let k = Scales {
            first: 0.5,
            reciprocal: 3.0,
            ticks: 2.0,
        };
        let mut c = Clock::default();
        c.tick(100, true, &k);
        assert_eq!(
            (c.started, c.last, c.now, c.before, c.reciprocal),
            (1, 100, 100, 100, 0.5)
        );
        c.tick(116, true, &k);
        assert_eq!((c.ticks, c.quarter, c.sixteenth, c.before), (16, 4, 1, 100));
        assert_eq!((c.reciprocal, c.reciprocal_scaled), (0.0625, 0.1875));
        assert_eq!(c.ticks_scaled, f32::from_bits(16) * 2.0);
        c.tick(120, false, &k);
        assert_eq!((c.ticks, c.reciprocal), (0, f32::INFINITY));
        c.tick(3, true, &k);
        assert_eq!(c.ticks, 3u32.wrapping_sub(120));
        assert!(c.reciprocal < 0.0);
    }
}
