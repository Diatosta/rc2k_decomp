//! Helpers of the differential tests that compare the plain-float versions with the
//! original assembly blocks kept as references (`asm_reference` modules of this
//! directory's files): one lock for all of them (they write shared statics and the
//! control word), a seeded generator, and the two control words the tests run at.

use std::sync::MutexGuard;

/// Holds the lock of the tests that share statics or the control word
/// (`crate::store::test_lock`, shared with every test of the statics).
pub(crate) fn lock() -> MutexGuard<'static, ()> {
    crate::store::test_lock()
}

/// A seeded generator of the inputs.
pub(crate) struct Rng(pub(crate) u64);

impl Rng {
    pub(crate) fn u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 32) as u32
    }

    /// A value in `0 .. n`.
    pub(crate) fn below(&mut self, n: u32) -> u32 {
        self.u32() % n
    }

    /// The bits of a float: mostly normal values in the game's ranges, now and then
    /// zeros, small, large, whole numbers and halves.
    pub(crate) fn float(&mut self) -> u32 {
        let unit = f64::from(self.u32()) / 4294967296.0;
        let sign = if self.u32() & 1 == 0 { 1.0 } else { -1.0 };
        let v = match self.below(10) {
            0 => 0.0,
            1 => -0.0,
            2 => unit * 1e-5,
            3 => unit * 1e9,
            4 => f64::from(self.below(200_000)) - 100_000.0,
            5 => (f64::from(self.below(2000)) - 1000.0) / 2.0,
            6 => unit * 2.0,
            7 => unit * 100.0,
            _ => unit * 20_000.0,
        };
        ((sign * v) as f32).to_bits()
    }

    /// A whole number in the screen's / world's range.
    pub(crate) fn int(&mut self) -> i32 {
        match self.below(4) {
            0 => self.below(2000) as i32 - 1000,
            1 => self.below(2_000_000) as i32 - 1_000_000,
            2 => 0,
            _ => (self.u32() >> 8) as i32 - (1 << 23),
        }
    }
}

/// Runs `f` at the control word 0x027F (nearest) and at 0x0E7F (chop), then puts 0x027F back.
pub(crate) fn at_each_control_word(mut f: impl FnMut()) {
    for cw in [0x027F, 0x0E7F] {
        crate::math::x87::set_control_word(cw);
        f();
    }
    crate::math::x87::set_control_word(0x027F);
}

/// Two floats' bits are the same, or both are NaN.
pub(crate) fn same(a: u32, b: u32) -> bool {
    a == b || (f32::from_bits(a).is_nan() && f32::from_bits(b).is_nan())
}

/// The game's dwords (IDA addresses) as they were, put back when dropped.
pub(crate) struct Saved(Vec<(usize, u32)>);

impl Saved {
    pub(crate) fn new(ida: &[usize]) -> Saved {
        use crate::framework::{game_read, rva};
        Saved(
            ida.iter()
                .map(|&a| (a, unsafe { game_read::<u32>(rva(a)) }))
                .collect(),
        )
    }
}

impl Drop for Saved {
    fn drop(&mut self) {
        use crate::framework::{game_write, rva};
        for &(a, v) in &self.0 {
            unsafe { game_write(rva(a), v) };
        }
    }
}

/// Two floats' bits are within `ulps` units in the last place (same sign), or both NaN.
pub(crate) fn within_ulps(a: u32, b: u32, ulps: u32) -> bool {
    if same(a, b) {
        return true;
    }
    let (x, y) = (f32::from_bits(a), f32::from_bits(b));
    if x.is_nan() || y.is_nan() {
        return false;
    }
    if (x == 0.0) && (y == 0.0) {
        return true;
    }
    (a >> 31 == b >> 31) && (a & 0x7FFF_FFFF).abs_diff(b & 0x7FFF_FFFF) <= ulps
}
