//! Seeded random inputs and the comparison the oracle's tests share: float bits of every kind
//! (random bits, zeros, infinities, NaNs of both signs with their payloads, denormals, the
//! largest and smallest finite values, whole numbers) and whole numbers up to the ends of
//! `i32`; results compare bit for bit, any two NaNs counting as equal.

use crate::math::x87;

pub(super) struct Rng(u64);

impl Rng {
    pub(super) fn new(seed: u64) -> Self {
        Rng(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    pub(super) fn flag(&mut self) -> bool {
        self.next() & 1 == 1
    }

    pub(super) fn float(&mut self) -> u32 {
        const SPECIAL: [u32; 16] = [
            0x0000_0000,
            0x8000_0000,
            0x7F80_0000,
            0xFF80_0000,
            0x7FC0_0000,
            0xFFC0_0000,
            0x7F80_0001,
            0xFF80_0001,
            0x0000_0001,
            0x8000_0001,
            0x007F_FFFF,
            0x7F7F_FFFF,
            0xFF7F_FFFF,
            0x3F80_0000,
            0xBF80_0000,
            0x0080_0000,
        ];
        match self.next() % 4 {
            0 => SPECIAL[(self.next() % 16) as usize],
            1 => (self.next() as i32 % 100_000) as f32 as i32 as f32 as u32,
            _ => self.next() as u32,
        }
    }

    pub(super) fn whole(&mut self) -> i32 {
        const EDGE: [i32; 6] = [0, 1, -1, i32::MIN, i32::MAX, 0x100];
        match self.next() % 3 {
            0 => EDGE[(self.next() % 6) as usize],
            1 => (self.next() % 2001) as i32 - 1000,
            _ => self.next() as i32,
        }
    }
}

/// Cases per rounding mode in `each_rounding`.
const ROUNDING_CASES: usize = 20_000;

/// Runs `check` at every rounding mode, `ROUNDING_CASES` times each, on a fixed seed.
pub(super) fn each_rounding(seed: u64, mut check: impl FnMut(&mut Rng)) {
    let word = x87::control_word();
    for mode in 0..4u16 {
        x87::set_control_word(word & 0xF3FF | mode << 10);
        let mut rng = Rng::new(seed);
        for _ in 0..ROUNDING_CASES {
            check(&mut rng);
        }
    }
    x87::set_control_word(word);
}

/// Results that compare equal when every float in them is bit for bit the same, except that
/// two NaNs of any sign and payload count as equal: the payload a NaN result carries is left
/// to the compiler (after a NaN already exists), so only that a NaN came out is checked.
pub(super) trait SameResult {
    fn same_result(&self, other: &Self) -> bool;
}

/// Float bits: equal, or both NaN.
impl SameResult for u32 {
    fn same_result(&self, other: &Self) -> bool {
        self == other || (f32::from_bits(*self).is_nan() && f32::from_bits(*other).is_nan())
    }
}

/// Whole numbers and flags: exactly equal.
impl SameResult for i32 {
    fn same_result(&self, other: &Self) -> bool {
        self == other
    }
}

impl SameResult for bool {
    fn same_result(&self, other: &Self) -> bool {
        self == other
    }
}

impl<T: SameResult, const N: usize> SameResult for [T; N] {
    fn same_result(&self, other: &Self) -> bool {
        self.iter().zip(other).all(|(a, b)| a.same_result(b))
    }
}

/// Tuples: every element on its own.
macro_rules! same_result_tuple {
    ($($name:ident $index:tt),+) => {
        impl<$($name: SameResult),+> SameResult for ($($name,)+) {
            fn same_result(&self, other: &Self) -> bool {
                $(self.$index.same_result(&other.$index))&&+
            }
        }
    };
}
same_result_tuple!(A 0, B 1);
same_result_tuple!(A 0, B 1, C 2);
same_result_tuple!(A 0, B 1, C 2, D 3);

/// `assert_eq!` for results made of float bits (see `SameResult`).
macro_rules! assert_same {
    ($left:expr, $right:expr $(,)?) => {
        assert_same!($left, $right, "")
    };
    ($left:expr, $right:expr, $($message:tt)+) => {{
        let (left, right) = ($left, $right);
        assert!(
            $crate::math::x87_checks::inputs::SameResult::same_result(&left, &right),
            "{}\n  left: {left:x?}\n right: {right:x?}",
            format_args!($($message)+)
        );
    }};
}
pub(super) use assert_same;
