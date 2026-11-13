//! The physics' constants with the `patch` settings worked in, once: gravity, a ground record's
//! slide, acceleration and brake grip. The settings (keys in `var\ral.cfg`) are `fizpatch` (the
//! switch), then `patchmaxdrawdistance`, `patchstickygrass`, `patchgravity`, `patchdamages`,
//! `patchbrakegrip`, `patchaccelgrip`, `patchslidegrip` and the grips of grass, tarmac, snow, mud
//! and gravel, the last ten in percent. Floats: each figure is one quotient (a whole number over
//! 100) or a product whose first step is exact, so it is correctly rounded at double and
//! extended precision.

/// The least and the most each setting may be.
pub const RANGES: [(u32, u32); 12] = [
    (4, 0x10),
    (0, 1),
    (10, 200),
    (0, 100_000),
    (10, 200),
    (10, 200),
    (10, 200),
    (10, 200),
    (10, 200),
    (10, 200),
    (10, 200),
    (10, 200),
];
/// The ground records' first three floats to begin with: 1.0, 0.9 (one bit above the nearest
/// float) and 1.7.
pub const PLAIN: [u32; 3] = [0x3F80_0000, 0x3F66_6667, 0x3FD9_999A];
/// The third float of these records: 1.2.
pub const OTHER_THIRD: ([usize; 2], u32) = ([3, 10], 0x3F99_999A);
pub const NINE_POINT_EIGHT: u32 = 0x411C_CCCD;
pub const TEN: u32 = 0x4120_0000;
/// By record: which of the ten factors its three floats are first multiplied by.
pub const BY_RECORD: [usize; 12] = [6, 9, 8, 7, 5, 8, 5, 5, 7, 5, 7, 9];
/// The factors the first, second and third float are then multiplied by.
pub const BY_FLOAT: [usize; 3] = [4, 3, 2];

/// What the patches read and write (as bits).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Patches {
    /// The settings' switch.
    pub switch: u32,
    /// The twelve settings, each held to its `RANGES` bounds.
    pub settings: [u32; 12],
    /// The ten percentages as factors.
    pub factors: [u32; 10],
    /// Set once applied.
    pub applied: u32,
    /// The gravity.
    pub gravity: u32,
    /// The grip figures of the 12 ground records.
    pub records: [[u32; 3]; 12],
    /// The drag of surfaces 7 and 8.
    pub tens: [u32; 2],
}

fn percent(number: u32, hundred: f32) -> u32 {
    ((number as i32 as f64 / hundred as f64) as f32).to_bits()
}

/// Works the settings in (`hundred` is 100.0); nothing if applied already.
pub fn apply_patches(p: &mut Patches, hundred: f32) {
    if p.applied != 0 {
        return;
    }
    p.applied = 1;
    for (s, (least, most)) in p.settings.iter_mut().zip(RANGES) {
        // The sign of the difference, whatever the overflow.
        if (s.wrapping_sub(least) as i32) < 0 {
            *s = least;
        }
        if (s.wrapping_sub(most) as i32) >= 0 {
            *s = most;
        }
    }
    p.gravity = NINE_POINT_EIGHT;
    for (n, record) in p.records.iter_mut().enumerate() {
        *record = PLAIN;
        if OTHER_THIRD.0.contains(&n) {
            record[2] = OTHER_THIRD.1;
        }
    }
    p.tens = [TEN; 2];
    if p.settings[3] > 100 {
        p.factors[1] = percent(p.settings[3], hundred);
    }
    if p.switch == 0 {
        return;
    }
    for n in 0..10 {
        p.factors[n] = percent(p.settings[2 + n], hundred);
    }
    let factor = |n: usize| f32::from_bits(p.factors[n]) as f64;
    p.gravity = ((f32::from_bits(p.gravity) as f64 * factor(0)) as f32).to_bits();
    for (record, first) in p.records.iter_mut().zip(BY_RECORD) {
        for (v, then) in record.iter_mut().zip(BY_FLOAT) {
            *v = ((f32::from_bits(*v) as f64 * factor(first) * factor(then)) as f32).to_bits();
        }
    }
    if p.settings[1] == 0 {
        p.tens = [0; 2];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patches() {
        let mut p = Patches {
            switch: 1,
            settings: [8, 0, 50, 4321, 200, 100, 100, 100, 100, 100, 100, 100],
            ..Default::default()
        };
        apply_patches(&mut p, 100.0);
        assert_eq!(p.applied, 1);
        assert_eq!(p.settings[3], 4321);
        assert_eq!(f32::from_bits(p.factors[0]), 0.5);
        assert_eq!(f32::from_bits(p.factors[1]), 43.21);
        assert_eq!(f32::from_bits(p.gravity), 9.8f32 * 0.5);
        // Record 0: by factor 6 (setting 8, 100%), then its third by factor 2 (200%).
        assert_eq!(f32::from_bits(p.records[0][0]), 1.0);
        assert_eq!(p.records[0][2], (1.7f32 * 2.0).to_bits());
        assert_eq!(p.tens, [0; 2]);
        let before = p;
        apply_patches(&mut p, 100.0);
        assert_eq!(p, before);
        // Held to the ranges, switched off: only the plain values.
        let mut q = Patches {
            settings: [0, u32::MAX, 9, 2, 0, 1, 2, 3, 4, 5, 6, 0x8000_0000],
            ..Default::default()
        };
        apply_patches(&mut q, 100.0);
        assert_eq!(q.settings[0], 4);
        assert_eq!(q.settings[1], 0);
        // 0x80000000 less 10 is not below 0 by the sign: held to the top instead.
        assert_eq!(q.settings[11], 200);
        assert_eq!(q.factors, [0; 10]);
        assert_eq!(q.records[3], [PLAIN[0], PLAIN[1], OTHER_THIRD.1]);
        assert_eq!(q.tens, [TEN; 2]);
    }
}
