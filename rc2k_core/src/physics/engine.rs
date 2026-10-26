//! The engine's torque from its speed (converted 2026-10-21 from `physics::engine_torque`
//! 0x43CBBB, `physics::engine_torque_or_cut` 0x43CB7D and `physics::free_rev` 0x43EA80):
//! the speed's share of its top, a curve of eight steps read by it, and the factors
//! after; the cut at the top; the engine revving free; the drive's two figures of
//! `physics::drive_force` 0x43CB14; and when `physics::share_engine_sound` 0x45D294
//! passes the engine's figures to the car's sound object. The revs heard eased toward
//! the engine's speed (converted 2026-10-21 from `physics::engine_sound` 0x43DC72; its
//! speed figures and the rounding stay the hook's x87).
//!
//! Floats: what the x87 keeps on its stack is an `f64`, rounded to `f32` where the
//! original stores (`physics::x87`): the same bits at the game's double precision. The
//! curve's entry is chosen from the stored share (a float) times the steps (a float
//! product, exact in an `f64`): the same entry at any precision.

use super::x87::{below, loaded};

/// `fld a; fmul b; fistp` with the rounding set to chop: the whole number toward 0, the
/// "integer indefinite" (0x80000000) for what does not fit.
pub fn fistp_chop(v: f64) -> i32 {
    let r = v.trunc();
    if r.is_nan() || r < i32::MIN as f64 || r >= 2147483648.0 {
        i32::MIN
    } else {
        r as i32
    }
}

/// The engine's figures `engine_torque` reads (the constants as read).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Engine {
    /// `g_carEngineSpeed` and its top (+0x1DC, `unk_71A1B4`).
    pub speed: f32,
    pub top: f32,
    /// `flt_5C5D90`, `flt_5C5D38` (0, 1): the curve's range.
    pub zero: f32,
    pub one: f32,
    /// `flt_5C5D50` (8): the curve's steps.
    pub steps: f32,
    /// `flt_613F94` (0.25).
    pub quarter: f32,
    /// +0x3F4 (`unk_71A3CC`): the torque's factor.
    pub factor: f32,
}

/// What `engine_torque` makes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Torque {
    /// The speed's share of its top (kept in `dword_62400C`).
    pub share: f32,
    /// The curve's entry (`dword_623FB8`, the original's ebx) when the share is in
    /// 0 ..= below 1.
    pub entry: Option<u32>,
    /// `flt_614744`: the share x `quarter`, from the unrounded quotient.
    pub revs: f32,
    /// `flt_614748`: the curve's line at the share (or its first value), x `factor`.
    pub torque: f32,
}

/// `physics::engine_torque`: `curve(i)` the line's two ends ((`flt_613FF0[i]`,
/// `flt_613FF0[i + 1]`), (`dword_614B7C[i]`, `dword_614B7C[i + 1]`)), asked only on the
/// curve's path; `first` (`dword_614B7C[0]`) asked only off it.
pub fn engine_torque(
    e: &Engine,
    curve: impl FnOnce(u32) -> ([f32; 2], [f32; 2]),
    first: impl FnOnce() -> f32,
) -> Torque {
    let quotient = e.speed as f64 / e.top as f64;
    let share = quotient as f32;
    let (entry, value) = if below(share as f64, e.one as f64) && !below(share as f64, e.zero as f64)
    {
        let i = fistp_chop(share as f64 * e.steps as f64) as u32;
        let ([a0, a1], [c0, c1]) = curve(i);
        let s = share as f64;
        let line = (s - a0 as f64) / (a1 as f64 - a0 as f64) * (c1 as f64 - c0 as f64) + c0 as f64;
        (Some(i), line as f32)
    } else {
        (None, loaded(first()))
    };
    Torque {
        share,
        entry,
        revs: (quotient * e.quarter as f64) as f32,
        torque: (value as f64 * e.factor as f64) as f32,
    }
}

/// `physics::engine_torque_or_cut`'s test: at or past the engine's top (an unordered
/// compare counts as below: no cut) the cut (+0x1F0, `unk_71A1C8`, set to 5); below it,
/// while the cut counts down, it goes down by one; either way the torque is 0 then
/// (Some(the cut)); else None: the torque is `engine_torque`'s.
pub fn torque_cut(speed: f32, top: f32, cut: u32) -> Option<u32> {
    if !below(speed as f64, top as f64) {
        Some(5)
    } else if cut != 0 {
        Some(cut.wrapping_sub(1))
    } else {
        None
    }
}

/// What `physics::free_rev` reads besides the torque's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FreeRev {
    /// `flt_614740`, `flt_5CB968`.
    pub drive: f32,
    pub drive_factor: f32,
    /// Gear 0's ratio (`g_gearRatios[0]`), `flt_5C5E34` (10000).
    pub ratio: f32,
    pub k10000: f32,
    /// +0x638 (`unk_71A610`, a whole number), `flt_5C5E2C` (256).
    pub gearing: i32,
    pub k256: f32,
    /// The step (`flt_61388C`).
    pub step: f32,
    /// `g_carEngineSpeed`.
    pub engine: f32,
    /// `flt_613FD8`, `flt_613FD4`: the drag's factor and base.
    pub drag_factor: f32,
    pub drag_base: f32,
}

/// What `physics::free_rev` makes: (`flt_614750`, the engine speed, the drag kept in
/// `dword_62400C`). The engine speed is brought toward 0 by the drag, not past it (at 0
/// or no number left alone).
pub fn free_rev(f: &FreeRev) -> (f32, f32, f32) {
    let push = (f.drive as f64 * f.drive_factor as f64 / f.ratio as f64 / f.k10000 as f64
        * f.gearing as f64
        / f.k256 as f64) as f32;
    let engine = (push as f64 * f.step as f64 / f.ratio as f64 + f.engine as f64) as f32;
    let drag = (engine as f64 * f.drag_factor as f64 + f.drag_base as f64) as f32;
    let eased = if engine == 0.0 || engine.is_nan() {
        engine
    } else if engine < 0.0 {
        let y = (engine as f64 + drag as f64) as f32;
        if below(y as f64, 0.0) { y } else { 0.0 }
    } else {
        let y = (engine as f64 - drag as f64) as f32;
        if below(y as f64, 0.0) { 0.0 } else { y }
    };
    (push, eased, drag)
}

/// What `physics::drive_force` reads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Drive {
    /// `flt_614748` (the torque), `flt_614744` (its base).
    pub torque: f32,
    pub base: f32,
    /// +0x638 (`unk_71A610`, a whole number), `flt_5C5E2C` (256).
    pub gearing: i32,
    pub k256: f32,
    /// `flt_5CB968`, the gear's ratio, `flt_5C5E34` (10000).
    pub factor: f32,
    pub ratio: f32,
    pub k10000: f32,
    /// +0x1D0 (`unk_71A1A8`) set.
    pub cut: bool,
}

/// `physics::drive_force`'s figures: `flt_614740` = (torque + base) x gearing / 256 -
/// base; `flt_614750` = it x factor / the gear's ratio / 10000, or 0 with the cut.
pub fn drive(d: &Drive) -> (f32, f32) {
    let b = d.base as f64;
    let engine = ((d.torque as f64 + b) * d.gearing as f64 / d.k256 as f64 - b) as f32;
    let drive = if d.cut {
        0.0
    } else {
        (engine as f64 * d.factor as f64 / d.ratio as f64 / d.k10000 as f64) as f32
    };
    (engine, drive)
}

/// `physics::share_engine_sound`'s test: the work car's figure (`flt_71A208`) above the
/// sound object's (+0xD0; `fcomp`: not equal, not below, not unordered): the four
/// figures (0x71A1FC .. 0x71A208) go to the object.
pub fn shares_figures(figure: f32, kept: f32) -> bool {
    !below(figure as f64, kept as f64) && figure != kept
}

/// The revs heard (`g_car` +0x1E4) held to these (by the sign of the difference).
pub const REVS_LOW: u32 = 2000;
pub const REVS_HIGH: u32 = 12000;

/// `physics::engine_sound`: the revs heard eased toward the engine's speed (`target`, the
/// whole number the x87 made of it, at least `flt_6140F8`) by 50/256 of the gap (`imul`,
/// `sar`), then held to `REVS_LOW` ..= `REVS_HIGH`. (the revs, the step: the original's
/// eax).
pub fn ease_revs(revs: u32, target: u32) -> (u32, u32) {
    let step = ((target.wrapping_sub(revs).wrapping_mul(50)) as i32 >> 8) as u32;
    let mut r = revs.wrapping_add(step);
    if (r.wrapping_sub(REVS_LOW) as i32) < 0 {
        r = REVS_LOW;
    }
    if (r.wrapping_sub(REVS_HIGH) as i32) >= 0 {
        r = REVS_HIGH;
    }
    (r, step)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revs() {
        assert_eq!(ease_revs(3000, 3512), (3100, 100));
        assert_eq!(ease_revs(3000, 0), (2414, (-586i32) as u32));
        assert_eq!(ease_revs(2000, 0).0, REVS_LOW);
        assert_eq!(ease_revs(11990, 20000).0, REVS_HIGH);
    }

    #[test]
    fn torque() {
        assert_eq!(fistp_chop(2.9), 2);
        assert_eq!(fistp_chop(-2.9), -2);
        assert_eq!(fistp_chop(f64::NAN), i32::MIN);
        let e = Engine {
            speed: 3000.0,
            top: 6000.0,
            zero: 0.0,
            one: 1.0,
            steps: 8.0,
            quarter: 0.25,
            factor: 2.0,
        };
        let curve = |i: u32| {
            assert_eq!(i, 4);
            ([0.5, 0.625], [100.0, 200.0])
        };
        let t = engine_torque(&e, curve, || panic!("not asked"));
        assert_eq!(
            t,
            Torque {
                share: 0.5,
                entry: Some(4),
                revs: 0.125,
                torque: 200.0
            }
        );
        let past = Engine { speed: 6000.0, ..e };
        let t = engine_torque(&past, |_| panic!("not asked"), || 7.0);
        assert_eq!((t.entry, t.torque), (None, 14.0));
        assert_eq!(torque_cut(6000.0, 6000.0, 0), Some(5));
        // No number counts as below (C0): no cut.
        assert_eq!(torque_cut(f32::NAN, 6000.0, 0), None);
        assert_eq!(torque_cut(100.0, 6000.0, 3), Some(2));
        assert_eq!(torque_cut(100.0, 6000.0, 0), None);
    }

    #[test]
    fn shares() {
        assert!(shares_figures(2.0, 1.0));
        assert!(!shares_figures(1.0, 1.0));
        assert!(!shares_figures(0.5, 1.0));
        assert!(!shares_figures(f32::NAN, 1.0));
        assert!(!shares_figures(2.0, f32::NAN));
    }

    #[test]
    fn drives() {
        let d = Drive {
            torque: 6.0,
            base: 2.0,
            gearing: 64,
            k256: 256.0,
            factor: 3.0,
            ratio: 0.5,
            k10000: 1.0,
            cut: false,
        };
        assert_eq!(drive(&d), (0.0, 0.0));
        assert_eq!(drive(&Drive { torque: 14.0, ..d }), (2.0, 12.0));
        assert_eq!(
            drive(&Drive {
                torque: 14.0,
                cut: true,
                ..d
            }),
            (2.0, 0.0)
        );
    }

    #[test]
    fn revving() {
        let f = FreeRev {
            drive: 10.0,
            drive_factor: 2.0,
            ratio: 0.5,
            k10000: 10.0,
            gearing: 3,
            k256: 2.0,
            step: 0.5,
            engine: 100.0,
            drag_factor: 0.0,
            drag_base: 1.0,
        };
        // push 10 x 2 / 0.5 / 10 x 3 / 2 = 6; engine 100 + 6 x 0.5 / 0.5 = 106; less 1.
        assert_eq!(free_rev(&f), (6.0, 105.0, 1.0));
        // -7 + 6 = -1, brought up by 2: not past 0.
        let back = FreeRev {
            engine: -7.0,
            drag_base: 2.0,
            ..f
        };
        assert_eq!(free_rev(&back), (6.0, 0.0, 2.0));
        // -9 + 6 = -3, up by 2; -6 + 6 = 0 is left alone.
        let down = FreeRev {
            engine: -9.0,
            drag_base: 2.0,
            ..f
        };
        assert_eq!(free_rev(&down).1, -1.0);
        let still = FreeRev {
            engine: -6.0,
            drag_base: 2.0,
            ..f
        };
        assert_eq!(free_rev(&still).1, 0.0);
    }
}
