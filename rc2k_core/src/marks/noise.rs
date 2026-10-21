//! The marks' noise tables (converted 2026-10-21 from `marks::seed_noise` 0x4C1697): 0x100
//! steps of the marks' seed (times 0x9D plus 0x47), each giving an entry, the last first,
//! of three tables: its low 18 bits, and those of it turned right 10 and 20. And the noise
//! curve (converted 2026-10-21 from `marks::make_noise_curve` 0x4C2247, `init_marks`' step
//! 5): a walk of 0x1000 steps, smoothed 0xA0 times, the random tables and the records'
//! layout. Its roundings (`fistp`, and a float stored) follow the x87's control word: they
//! come in as closures, the x87's in the game.

/// The entries of a table.
pub const ENTRIES: usize = 0x100;

/// The seed's step.
pub fn next_seed(seed: u32) -> u32 {
    seed.wrapping_mul(0x9D).wrapping_add(0x47)
}

/// The three values a seed gives.
pub fn values(seed: u32) -> [u32; 3] {
    [0, 10, 20].map(|turn| seed.rotate_right(turn) & 0x3FFFF)
}

/// `seed_noise`: the three tables (entry 0xFF first) and the seed after.
pub fn seed_noise(mut seed: u32) -> ([[u32; ENTRIES]; 3], u32) {
    let mut tables = [[0; ENTRIES]; 3];
    for n in (0..ENTRIES).rev() {
        seed = next_seed(seed);
        for (table, v) in tables.iter_mut().zip(values(seed)) {
            table[n] = v;
        }
    }
    (tables, seed)
}

/// The curve's length (`g_marksCurve`, `g_marksCurveCopy`).
pub const CURVE_LEN: usize = 0x1000;
/// The walk's step: held to -0x1000 .. 0x1000, moved by 2 each time.
pub const LEAST_STEP: i32 = -0x1000;
pub const MOST_STEP: i32 = 0x1000;
pub const CHANGE: i32 = 2;
/// The seed the curve starts from.
pub const CURVE_SEED: u32 = 0x7FF_FFFF;
/// How many times the curve is smoothed.
pub const SMOOTHINGS: u32 = 0xA0;
/// The random tables' entries.
pub const RANDOMS: usize = 0x800;

/// The walk's end: the curve, the value and the step, the running sum of the seeds and the
/// seed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Walk {
    pub curve: Vec<i32>,
    pub value: i32,
    pub step: i32,
    pub sum: u32,
    pub seed: u32,
}

/// The walk: from `CURVE_SEED`, each entry the value << 4 scaled (`scaled`); the step moved
/// by 2 up, or down when the running sum of the seeds (from `sum`) carries, held to its
/// limits; the value plus the step, damped (`damped`).
pub fn walk(
    sum: u32,
    mut scaled: impl FnMut(i32) -> i32,
    mut damped: impl FnMut(i32) -> i32,
) -> Walk {
    let mut w = Walk {
        curve: vec![0; CURVE_LEN],
        value: 0,
        step: 0,
        sum,
        seed: CURVE_SEED,
    };
    for n in 0..CURVE_LEN {
        w.curve[n] = scaled(w.value << 4);
        let mut next = w.step.wrapping_add(CHANGE);
        w.seed = next_seed(w.seed);
        let (sum, carry) = w.sum.overflowing_add(w.seed);
        w.sum = sum;
        if carry {
            next = next.wrapping_sub(CHANGE).wrapping_sub(CHANGE);
        }
        w.step = next.clamp(LEAST_STEP, MOST_STEP);
        w.value = damped(w.value.wrapping_add(w.step));
    }
    w
}

/// One smoothing: the curve copied, and each of 1 .. 0xFFD made from its neighbours and
/// itself in the copy (`weighted(before, it, after)`).
pub fn smooth(curve: &mut [i32], copy: &mut [i32], mut weighted: impl FnMut(i32, i32, i32) -> i32) {
    copy.copy_from_slice(curve);
    for n in 1..0xFFE {
        curve[n] = weighted(copy[n - 1], copy[n], copy[n + 1]);
    }
}

/// Whether a remainder's divisor can be divided by (`idiv` traps on 0, and on -1 or 1 it
/// is pointless; refused).
pub fn divisor_fits(d: i32) -> bool {
    d.unsigned_abs() >= 2
}

/// The random tables, from the last entry: three of 20-bit randoms (the seed and it turned
/// right 10 and 20) and the seed's top three bits turned (`seed >> 30` taken as 3 bits of
/// it turned right 30), for the spread; then two of remainders by the divisors, each its own
/// seed. Returns the tables, the three-bit values, the remainders and the seed after.
pub fn random_tables(mut seed: u32, divisors: [i32; 2]) -> RandomTables {
    let mut t = RandomTables {
        randoms: [[0; RANDOMS]; 3],
        threes: [0; RANDOMS],
        remainders: [[0; RANDOMS]; 2],
        seed,
    };
    for n in (0..RANDOMS).rev() {
        seed = next_seed(seed);
        for (k, turn) in [0u32, 10, 20].into_iter().enumerate() {
            t.randoms[k][n] = seed.rotate_right(turn) & 0xF_FFFF;
        }
        t.threes[n] = (seed.rotate_right(30) & 7) as i32;
    }
    for n in (0..RANDOMS).rev() {
        for (k, d) in divisors.into_iter().enumerate() {
            seed = next_seed(seed);
            t.remainders[k][n] = (seed as i64 % d as i64) as u32;
        }
    }
    t.seed = seed;
    t
}

/// `random_tables`' result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RandomTables {
    pub randoms: [[u32; RANDOMS]; 3],
    pub threes: [i32; RANDOMS],
    pub remainders: [[u32; RANDOMS]; 2],
    pub seed: u32,
}

/// The curve records' (0x20 of 0x80 bytes) values by offset: each quarter's +0xC .. +0x14
/// the first value, -1, 0; then the other four values at their places.
pub fn record_values([quarter, a, b, c, d]: [u32; 5]) -> Vec<(u32, u32)> {
    let mut v: Vec<(u32, u32)> = (0..4)
        .flat_map(|q| {
            [
                (0x20 * q + 0xC, quarter),
                (0x20 * q + 0x10, u32::MAX),
                (0x20 * q + 0x14, 0),
            ]
        })
        .collect();
    v.extend([
        (0x18, a),
        (0x38, c),
        (0x58, c),
        (0x78, a),
        (0x1C, b),
        (0x3C, b),
        (0x5C, d),
        (0x7C, d),
    ]);
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curve() {
        let w = walk(0, |v| v * 8, |v| v - v / 10);
        assert_eq!(w.curve[0], 0);
        // the first step: up 2 (no carry), value 2 damped to 2
        assert_eq!(w.curve[1], 2 << 4 << 3);
        assert_eq!(w.seed, (0..CURVE_LEN).fold(CURVE_SEED, |s, _| next_seed(s)));
        assert!(w.step >= LEAST_STEP && w.step <= MOST_STEP);
        let mut c: Vec<i32> = (0..CURVE_LEN as i32).collect();
        let mut copy = vec![0; CURVE_LEN];
        c[5] = 100;
        smooth(&mut c, &mut copy, |a, b, d| a + b + d);
        assert_eq!(
            (c[0], c[4], c[5], c[0xFFE], c[0xFFF]),
            (0, 3 + 100 + 4, 4 + 100 + 6, 0xFFE, 0xFFF)
        );
        assert!(!divisor_fits(0) && !divisor_fits(-1) && divisor_fits(2) && divisor_fits(i32::MIN));
        let t = random_tables(1, [0xC00, -7]);
        let s = next_seed(1);
        assert_eq!(t.randoms[1][RANDOMS - 1], s.rotate_right(10) & 0xF_FFFF);
        assert_eq!(t.threes[RANDOMS - 1], (s.rotate_right(30) & 7) as i32);
        let s = (0..RANDOMS + 1).fold(1, |s, _| next_seed(s));
        assert_eq!(t.remainders[0][RANDOMS - 1], (s as i64 % 0xC00) as u32);
        assert_eq!(t.seed, (0..3 * RANDOMS).fold(1, |s, _| next_seed(s)));
        let r = record_values([1, 2, 3, 4, 5]);
        assert_eq!(
            (r.len(), r[0], r[1], r[13], r[19]),
            (20, (0xC, 1), (0x10, u32::MAX), (0x38, 4), (0x7C, 5))
        );
    }

    #[test]
    fn tables() {
        let (t, seed) = seed_noise(0);
        assert_eq!(t[0][0xFF], 0x47);
        assert_eq!(t[1][0xFF], 0);
        assert_eq!(t[2][0xFF], (0x47u32.rotate_right(20)) & 0x3FFFF);
        let mut s = 0;
        for _ in 0..0x100 {
            s = next_seed(s);
        }
        assert_eq!((seed, values(s)), (s, [t[0][0], t[1][0], t[2][0]]));
    }
}
