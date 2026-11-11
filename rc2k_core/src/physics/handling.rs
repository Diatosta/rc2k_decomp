//! The car's damage made into its working grades and failures, and its handling figures from its
//! setup (from `physics::rate_damage` 0x4AF1EC and `physics::set_handling` 0x43EB5D). What each
//! figure is, is a reading at best; the variables are named by where they are: the work copy
//! (`g_car`) +offset, the grades' block (0x75B340) +offset, the handling block (0x613E00) +offset,
//! and the setup's scale (`k256`, 256.0). The functions read and write them through `Figures`, in
//! the original's order, each only where the original does.
//!
//! Floats: what the x87 keeps on its stack is an `f64`, rounded to `f32` where the original stores
//! (`physics::x87`): the same bits at the game's double precision. The chances' whole numbers are
//! `fistp`ed from the product of two floats (exact in an `f64`, as on the x87), so the failures
//! follow the original's to the bit.

use super::x87::{below, equal, fistp64, loaded};
use crate::Invalid;

/// Where a variable is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum At {
    /// The work copy (`g_car`, 0x719FD8) +offset.
    Car(u32),
    /// The grades' block (0x75B340) +offset.
    Grades(u32),
    /// The handling block (0x613E00) +offset.
    Handling(u32),
    /// The setup's scale (`k256`, 256.0).
    Scale,
}

/// The variables, read and written as dwords.
pub trait Figures {
    fn get(&self, at: At) -> u32;
    fn put(&mut self, at: At, v: u32);

    /// The dword at `at` as a float.
    fn float(&self, at: At) -> f32 {
        f32::from_bits(self.get(at))
    }

    /// A float stored at `at`.
    fn put_float(&mut self, at: At, v: f32) {
        self.put(at, v.to_bits());
    }
}

use At::{Car, Grades, Handling};

/// The products' factor.
const K: At = Grades(0x48);
/// The grades' floor and ceiling.
const LOW: At = Grades(0x10);
const HIGH: At = Grades(0x08);
/// Zero, as the original compares with it.
const ZERO: At = Grades(0x0C);
/// The chances' factor.
const CHANCE: At = Grades(0);
/// The random the chances are drawn against.
const RANDOM: At = Grades(0x160);
/// Each chance's whole number.
const SCRATCH: At = Grades(0x2C);
/// `engine_power_factor`: the first grade (+0x3F4).
const FIRST: At = Car(0x3F4);
/// `g_carGearBits` (+0x434).
pub const GEAR_BITS: At = Car(0x434);
/// The last ramp table (0x75B568), where the original's table pointer is left.
pub const LAST_TABLE: At = Grades(0x228);
/// The gears counted into the bits (the original's counter is left at it).
pub const GEARS: u32 = 9;

/// The ramps in order: (destination, figure, table).
const RAMPS_A: [(At, At, At); 2] = [
    (Car(0x400), Car(0x2A4), Grades(0x1A8)),
    (Car(0x404), Car(0x2A8), Grades(0x1A8)),
];
const RAMPS_C: [(At, At, At); 18] = [
    (Grades(0x150), Car(0x2C8), Grades(0x1D8)),
    (Grades(0x154), Car(0x2F4), Grades(0x1E8)),
    (Grades(0x158), Car(0x2F4), Grades(0x1F8)),
    (Grades(0x15C), Car(0x350), Grades(0x208)),
    (Grades(0xC4), Car(0x310), Grades(0x218)),
    (Grades(0xC8), Car(0x314), Grades(0x218)),
    (Grades(0xCC), Car(0x318), Grades(0x218)),
    (Grades(0xD0), Car(0x31C), Grades(0x218)),
    (Grades(0xD4), Car(0x320), Grades(0x218)),
    (Grades(0xD8), Car(0x324), Grades(0x218)),
    (Grades(0xDC), Car(0x328), Grades(0x218)),
    (Grades(0xE0), Car(0x310), Grades(0x218)),
    (Grades(0xE4), Car(0x314), Grades(0x228)),
    (Grades(0xE8), Car(0x318), Grades(0x228)),
    (Grades(0xEC), Car(0x31C), Grades(0x228)),
    (Grades(0xF0), Car(0x320), Grades(0x228)),
    (Grades(0xF4), Car(0x324), Grades(0x228)),
    (Grades(0xF8), Car(0x328), Grades(0x228)),
];
/// The failures that come and go: (flag, figure while set, figure while clear, turn).
const FAILURES: [(At, At, At, u32); 7] = [
    (Car(0x40C), Grades(0xE0), Grades(0xC4), 4),
    (Car(0x410), Grades(0xE4), Grades(0xC8), 5),
    (Car(0x414), Grades(0xE8), Grades(0xCC), 6),
    (Car(0x418), Grades(0xEC), Grades(0xD0), 7),
    (Car(0x41C), Grades(0xF0), Grades(0xD4), 8),
    (Car(0x420), Grades(0xF4), Grades(0xD8), 9),
    (Car(0x424), Grades(0xF8), Grades(0xDC), 0xA),
];

/// A ramp through (t0, t1) .. (t2, t3) as the original reads it (`math::x87::ramp`):
/// below t0 (or no number) t1; not below t2, t3; t2 - t0 equal to `zero` (or no number),
/// t1; else (x - t0) / (t2 - t0) x (t3 - t1) + t1, rounded to a float.
pub fn ramp(x: f32, t: [f32; 4], zero: f32) -> f32 {
    let [t0, t1, t2, t3] = t.map(f64::from);
    if below(x as f64, t0) {
        return loaded(t[1]);
    }
    if !below(x as f64, t2) {
        return loaded(t[3]);
    }
    let span = t2 - t0;
    if equal(span, zero as f64) {
        return loaded(t[1]);
    }
    ((x as f64 - t0) / span * (t3 - t1) + t1) as f32
}

/// 1 less the products a b c of `terms`, in order (kept on the stack: unrounded).
pub fn one_less_products(terms: &[[f32; 3]]) -> f64 {
    terms
        .iter()
        .fold(1.0, |acc, t| acc - t[0] as f64 * t[1] as f64 * t[2] as f64)
}

/// lo + i / d (hi - lo) (`math::x87::int_lerp`), rounded to a float.
pub fn int_lerp(i: i32, d: f32, hi: f32, lo: f32) -> f32 {
    (i as f64 / d as f64 * (hi as f64 - lo as f64) + lo as f64) as f32
}

/// Held to the grades' floor and ceiling, each compared after the last store.
fn hold(m: &mut impl Figures, v: At) {
    if below(m.float(v) as f64, m.float(LOW) as f64) {
        let low = loaded(m.float(LOW));
        m.put_float(v, low);
    }
    if !below(m.float(v) as f64, m.float(HIGH) as f64) {
        let high = loaded(m.float(HIGH));
        m.put_float(v, high);
    }
}

/// The figure at `x` along the table at `t` into `out`.
fn ramp_into(m: &mut impl Figures, out: At, x: At, t: At) {
    let Grades(o) = t else { unreachable!() };
    let table = [0, 4, 8, 12].map(|k| m.float(Grades(o + k)));
    let v = ramp(m.float(x), table, m.float(LOW));
    m.put_float(out, v);
}

/// A failure's chance: the figure times the chances' factor as a whole number (kept),
/// against the random's low word turned right by `turn`: whether the random is below.
fn chance(m: &mut impl Figures, figure: At, turn: u32) -> bool {
    let t = fistp64(m.float(figure) as f64 * m.float(CHANCE) as f64) as u32;
    m.put(SCRATCH, t);
    let r = m.get(RANDOM).rotate_right(turn) & 0xFFFF;
    (r.wrapping_sub(t) as i32) < 0
}

/// `physics::rate_damage`: the damage figures made into the grades and failures; returns the gear
/// bits.
pub fn rate_damage(m: &mut impl Figures) -> u32 {
    let k = m.float(K);
    let mut e = one_less_products(&[
        [m.float(Car(0x2B8)), k, m.float(Grades(0x354))],
        [m.float(Car(0x2BC)), k, m.float(Grades(0x358))],
    ]);
    if m.get(Grades(0xA4)) != 0 && !below(m.float(Car(0x2C0)) as f64, m.float(ZERO) as f64) {
        e *= m.float(Grades(0x260)) as f64;
    }
    m.put_float(FIRST, e as f32);
    hold(m, FIRST);
    for (out, a, b) in [
        (Car(0x3F8), Car(0x2AC), Car(0x348)),
        (Car(0x3FC), Car(0x2B0), Car(0x34C)),
    ] {
        let e = one_less_products(&[
            [m.float(a), k, m.float(Grades(0x360))],
            [m.float(b), k, m.float(Grades(0x35C))],
            [m.float(Car(0x2A0)), k, m.float(Grades(0x364))],
            [m.float(Grades(0xB0)), k, m.float(Grades(0x368))],
        ]);
        m.put_float(out, e as f32);
        hold(m, out);
    }
    for (out, x, t) in RAMPS_A {
        ramp_into(m, out, x, t);
    }
    for (out, x) in [(Car(0x400), Grades(0x100)), (Car(0x404), Grades(0x104))] {
        ramp_into(m, Grades(0xFC), x, Grades(0x1B8));
        let v = m.float(Grades(0xFC)) as f64 * k as f64 * k as f64 * m.float(out) as f64;
        m.put_float(out, v as f32);
    }
    ramp_into(m, Car(0x408), Car(0x330), Grades(0x1C8));
    let v = m.float(Car(0x408)) as f64;
    m.put_float(Car(0x408), (v * v) as f32);
    for (out, x, t) in RAMPS_C {
        ramp_into(m, out, x, t);
    }
    if m.get(Car(0x428)) == 0 && chance(m, Grades(0x15C), 0xD) {
        m.put(Car(0x428), 1);
    }
    for (flag, set, clear, turn) in FAILURES {
        if m.get(flag) != 0 {
            if chance(m, set, turn) {
                m.put(flag, 0);
            }
        } else if chance(m, clear, turn) {
            m.put(flag, 1);
        }
    }
    m.put(Car(0x42C), 0);
    if chance(m, Grades(0x150), 0xB) {
        m.put(Car(0x42C), 1);
    }
    if chance(m, Grades(0x154), 0x19) {
        m.put(Car(0x42C), 1);
    }
    m.put(Car(0x430), 0);
    if chance(m, Grades(0x158), 0x12) {
        m.put(Car(0x430), 1);
    }
    m.put(Car(0x2D4), 0);
    let mut bits = 0u32;
    for n in 0..GEARS {
        if equal(m.float(Car(0x2D0 + 4 * n)) as f64, m.float(ZERO) as f64) {
            bits |= 1 << n;
        }
    }
    m.put(GEAR_BITS, bits);
    let all_below = [Car(0x2B8), Car(0x2C8), Car(0x2CC), Car(0x2F4), Car(0x310)]
        .iter()
        .all(|&v| below(m.float(v) as f64, m.float(ZERO) as f64));
    if all_below {
        m.put(Car(0x29C), 0);
    } else {
        m.put(Car(0x29C), 1);
        if !below(m.float(Car(0x310)) as f64, m.float(ZERO) as f64) {
            m.put(FIRST, 0);
        }
    }
    bits
}

/// `g_carDriveKind` (+0x40): 0 or 1 (the pair tables hold two).
pub const DRIVE_KIND: At = Car(0x40);

/// `physics::set_handling`'s interpolations: (out, setting, hi, lo), in the original's
/// order but for the drive kind's.
const BEFORE: [(At, At, At, At); 4] = [
    (Handling(0xE0C), Car(0x3C), Handling(0x128), Handling(0x124)),
    (Handling(0xE08), Car(0x3C), Handling(0x130), Handling(0x12C)),
    (Handling(0xDF0), Car(0x20), Handling(0x120), Handling(0x11C)),
    (Handling(0xDF4), Car(0x30), Handling(0x148), Handling(0x144)),
];
const AFTER: [(At, At, At, At); 4] = [
    (Handling(0xDEC), Car(0x34), Handling(0x188), Handling(0x184)),
    (Handling(0xDF8), Car(0x34), Handling(0x170), Handling(0x16C)),
    (Handling(0xE00), Car(0x34), Handling(0x178), Handling(0x174)),
    (Handling(0xE04), Car(0x34), Handling(0x180), Handling(0x17C)),
];

/// `physics::set_handling`: a drive kind above 1 refused before anything; then `grade`
/// (`physics::grade_or_reset`, its result passed back) and the nine figures, each lo + setting /
/// 256 (hi - lo), the drive kind's then times the drive-kind factor. Also returns the drive kind.
pub fn set_handling<M: Figures, T>(
    m: &mut M,
    grade: impl FnOnce(&mut M) -> T,
) -> Result<(T, u32), Invalid> {
    let kind = m.get(DRIVE_KIND);
    if kind > 1 {
        return Err(Invalid("a drive kind above 1 (the pair tables hold two)"));
    }
    let graded = grade(m);
    let lerp = |m: &mut M, (out, n, hi, lo): (At, At, At, At)| {
        let v = int_lerp(
            m.get(n) as i32,
            m.float(At::Scale),
            m.float(hi),
            m.float(lo),
        );
        m.put_float(out, v);
    };
    for v in BEFORE {
        lerp(m, v);
    }
    let drive = Handling(0xDFC);
    lerp(
        m,
        (
            drive,
            Car(0x30),
            Handling(0x154 + 4 * kind),
            Handling(0x14C + 4 * kind),
        ),
    );
    let v = m.float(drive) as f64 * m.float(Handling(0xE18)) as f64;
    m.put_float(drive, v as f32);
    for v in AFTER {
        lerp(m, v);
    }
    Ok((graded, kind))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    struct Map(HashMap<At, u32>);

    impl Figures for Map {
        fn get(&self, at: At) -> u32 {
            *self.0.get(&at).unwrap_or(&0)
        }
        fn put(&mut self, at: At, v: u32) {
            self.0.insert(at, v);
        }
    }

    #[test]
    fn ramps() {
        let t = [1.0, 10.0, 3.0, 20.0];
        assert_eq!(ramp(0.5, t, 0.0), 10.0);
        assert_eq!(ramp(f32::NAN, t, 0.0), 10.0);
        assert_eq!(ramp(3.0, t, 0.0), 20.0);
        assert_eq!(ramp(2.0, t, 0.0), 15.0);
        // A flat table: t1.
        assert_eq!(ramp(1.0, [1.0, 7.0, 1.0, 9.0], 0.0), 9.0);
        assert_eq!(
            one_less_products(&[[0.5, 2.0, 0.25], [1.0, 1.0, 0.5]]),
            0.25
        );
        assert_eq!(int_lerp(128, 256.0, 3.0, 1.0), 2.0);
    }

    #[test]
    fn rates() {
        let mut m = Map::default();
        m.put_float(K, 1.0);
        m.put_float(LOW, 0.0);
        m.put_float(HIGH, 1.0);
        m.put_float(Car(0x2B8), 0.25);
        m.put_float(Grades(0x354), 1.0);
        m.put_float(CHANCE, 100.0);
        // A chance figure of 1: 100 against the random's low word 0x50 turned: below.
        m.put_float(Grades(0x1D8), 0.0);
        m.put_float(Grades(0x1E0), 0.0);
        m.put_float(Grades(0x1DC), 1.0);
        m.put_float(Grades(0x1E4), 1.0);
        m.put_float(Car(0x2C8), 5.0);
        m.put(RANDOM, 0x50 << 0xB);
        let bits = rate_damage(&mut m);
        assert_eq!(
            m.float(FIRST),
            0.0,
            "the first grade cleared: +0x310 not below 0"
        );
        assert_eq!(m.float(Car(0x3F8)), 1.0, "1 held to the ceiling");
        assert_eq!(m.get(SCRATCH), 0, "the last chance's figure");
        assert_eq!(m.get(Car(0x42C)), 1);
        assert_eq!(m.get(Car(0x29C)), 1);
        assert_eq!(bits, 0x1FF, "every gear's figure is 0");
        assert_eq!(m.get(GEAR_BITS), bits);
    }

    #[test]
    fn handling() {
        let mut m = Map::default();
        m.put(DRIVE_KIND, 2);
        assert!(set_handling(&mut m, |_| ()).is_err());
        assert!(m.0.len() == 1, "nothing written");
        m.put(DRIVE_KIND, 1);
        m.put_float(At::Scale, 256.0);
        m.put(Car(0x30), 64);
        m.put_float(Handling(0x158), 5.0);
        m.put_float(Handling(0x150), 1.0);
        m.put_float(Handling(0xE18), 2.0);
        let (graded, kind) = set_handling(&mut m, |m| m.get(DRIVE_KIND) + 7).unwrap();
        assert_eq!((graded, kind), (8, 1));
        assert_eq!(m.float(Handling(0xDFC)), 4.0);
    }
}
