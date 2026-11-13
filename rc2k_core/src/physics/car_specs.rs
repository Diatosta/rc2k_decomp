//! The work car's handling figures from its kind's record (0xB0 bytes, kinds up to 0x21).
//!
//! Floats: what the x87 keeps on its stack is an `f64`, rounded to `f32` where it is stored.
//! The three whole figures are rounded from a product and a quotient; at extended precision
//! one may land one off at a rounding edge.

use super::x87::{fistp64, hold_low, loaded};
use crate::Invalid;

/// The highest kind with a record.
pub const LAST_KIND: u32 = 0x21;
/// A record's size in bytes.
pub const RECORD_LEN: u32 = 0xB0;
/// The kinds below which the whole figures come from the constants.
pub const OWN_FIGURES_FROM: u32 = 0x17;
/// The number of drive kinds.
pub const DRIVE_KINDS: u32 = 4;

/// The record's index for a kind (kinds past the last use the last).
pub fn record_of(kind: u32) -> u32 {
    kind.min(LAST_KIND)
}

/// The record's figures that are read.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SpecRecord {
    /// The drive kind.
    pub drive: u32,
    /// Four whole figures.
    pub figures: [i32; 4],
    /// The engine's top, a whole number.
    pub top: i32,
    /// A whole figure over the top.
    pub over_top: i32,
    /// The shift points.
    pub shifts: [f32; 3],
    /// Three whole figures (x 65536).
    pub whole: [i32; 3],
    /// Two figures times gravity.
    pub weights: [f32; 2],
}

/// The constants the figures are made with.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SpecConstants {
    pub f613f10: f32,
    /// The last shift point's factor.
    pub f613e88: f32,
    /// Added to the second and third shift points.
    pub f5c5ddc: f32,
    pub f613eb8: f32,
    pub f613fe4: f32,
    /// The top's floor and the whole figures' divisor.
    pub f5c5d18: f32,
    /// The four figures' floor (8.0).
    pub f5c5d50: f32,
    /// 65536.
    pub f5c5d98: f32,
    /// Gravity.
    pub gravity: f32,
    /// 100.
    pub hundred: f32,
    /// The whole figures of kinds below `OWN_FIGURES_FROM`.
    pub own: [i32; 3],
}

/// The figures stored from floats.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Specs {
    /// The four figures (at least the floor).
    pub figures: [f32; 4],
    /// The engine's top (at least `f5c5d18`).
    pub top: f32,
    /// `f613f10` as loaded.
    pub w1e0: f32,
    /// The shift points (the last the first times `f613e88`).
    pub shifts: [f32; 4],
    pub f5cb968: f32,
    pub f614104: f32,
    /// The whole figures x 65536 / `f5c5d18`, rounded.
    pub whole: [i32; 3],
    pub weights: [f32; 2],
    /// The drive kind's figure; the second it + 100.
    pub drive: [f32; 2],
}

/// The drive kind, checked against the number of kinds.
pub fn check_drive(drive: u32) -> Result<u32, Invalid> {
    if drive >= DRIVE_KINDS {
        return Err(Invalid("a drive kind past 3"));
    }
    Ok(drive)
}

/// The gear count held to 3 ..= 7 (by sign).
pub fn gear_count(count: u32) -> u32 {
    let mut c = count;
    if (c.wrapping_sub(3) as i32) < 0 {
        c = 3;
    }
    if (c.wrapping_sub(7) as i32) >= 0 {
        c = 7;
    }
    c
}

/// The figures from a record, its kind, the constants and the drive kind's figure.
pub fn specs(r: &SpecRecord, kind: u32, k: &SpecConstants, drive_figure: f32) -> Specs {
    let f = |x: f32| x as f64;
    let mut s = Specs {
        figures: r.figures.map(|v| v as f32),
        top: r.top as f32,
        w1e0: loaded(k.f613f10),
        ..Specs::default()
    };
    s.shifts[0] = loaded(r.shifts[0]);
    s.shifts[3] = (f(r.shifts[0]) * f(k.f613e88)) as f32;
    s.shifts[1] = (f(r.shifts[1]) + f(k.f5c5ddc)) as f32;
    s.shifts[2] = (f(r.shifts[2]) + f(k.f5c5ddc)) as f32;
    s.f5cb968 = (r.over_top as f64 * f(k.f613eb8) / f(s.top)) as f32;
    s.f614104 = (f(s.top) * f(k.f613fe4)) as f32;
    s.top = hold_low(s.top, k.f5c5d18);
    s.figures = s.figures.map(|v| hold_low(v, k.f5c5d50));
    let whole = if kind < OWN_FIGURES_FROM {
        k.own
    } else {
        r.whole
    };
    s.whole = whole.map(|v| fistp64(v as f64 * f(k.f5c5d98) / f(k.f5c5d18)));
    s.weights = r.weights.map(|w| (f(w) * f(k.gravity)) as f32);
    s.drive = [
        loaded(drive_figure),
        (f(drive_figure) + f(k.hundred)) as f32,
    ];
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn figures() {
        let r = SpecRecord {
            drive: 1,
            figures: [4, 9, 10, 2],
            top: 7000,
            over_top: 350,
            shifts: [0.9, 0.5, 0.6],
            whole: [3, 5, 7],
            weights: [2.0, 0.5],
        };
        let k = SpecConstants {
            f613f10: 1.5,
            f613e88: 0.5,
            f5c5ddc: 0.25,
            f613eb8: 2.0,
            f613fe4: 0.5,
            f5c5d18: 1000.0,
            f5c5d50: 8.0,
            f5c5d98: 65536.0,
            gravity: 10.0,
            hundred: 100.0,
            own: [1, 2, 3],
        };
        let s = specs(&r, 0x17, &k, 3.0);
        assert_eq!(s.figures, [8.0, 9.0, 10.0, 8.0]);
        assert_eq!(s.top, 7000.0);
        assert_eq!(s.shifts, [0.9, 0.75, 0.85, 0.45]);
        assert_eq!(s.f5cb968, 0.1);
        assert_eq!(s.f614104, 3500.0);
        assert_eq!(s.whole, [197, 328, 459]);
        assert_eq!(s.weights, [20.0, 5.0]);
        assert_eq!(s.drive, [3.0, 103.0]);
        assert_eq!(specs(&r, 0x16, &k, 3.0).whole, [66, 131, 197]);
        let low = SpecRecord { top: 10, ..r };
        assert_eq!(specs(&low, 0x17, &k, 3.0).top, 1000.0);
        assert_eq!(gear_count(0), 3);
        assert_eq!(gear_count(9), 7);
        assert_eq!(gear_count(u32::MAX), 3);
        assert!(check_drive(4).is_err());
        assert_eq!(record_of(0x40), LAST_KIND);
    }
}
