//! The work car's motion held to its limits (Converted from
//! `physics::limit_motion` 0x43F7AC): a motion part that is no number or infinite zeroes
//! the motion, its car-frame copy and the spin; then the spin's parts held to ± one limit
//! and the motion's (x, z, y) to ± another; past the top speed the motion and its copy
//! scaled down to it.
//!
//! Floats: what the x87 keeps on its stack is an `f64`, rounded to `f32` where the
//! original stores (`physics::x87`): the same bits at the game's double precision.

use super::x87::{below, loaded, negated};
use crate::math::vector::{length, scale};

/// The variables it holds (the work copy's +0x184, +0x198, +0x1A4) and the two limits
/// (`spin_limit`, `velocity_limit`: negated in place for the second compare and put back).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Moving {
    pub spin: [f32; 3],
    pub motion: [f32; 3],
    pub local: [f32; 3],
    pub spin_limit: f32,
    pub motion_limit: f32,
}

/// What it leaves besides.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Limited {
    /// The last motion part's exponent bits looked at: the first all ones, or z's.
    pub exponent: u32,
    /// The motion's length after the holds (`motion_length`).
    pub length: f32,
    /// The scale past the top (`scratch`), or none.
    pub factor: Option<f32>,
}

/// `v` held to ± `l`: `fld l; fld v; fcompp` (not below: `l`), `l` negated, the same the
/// other way (below: `-l`), `l` negated back.
pub fn hold(v: &mut f32, l: &mut f32) {
    if !below(*v as f64, *l as f64) {
        *v = loaded(*l);
    }
    *l = negated(*l);
    if below(*v as f64, *l as f64) {
        *v = loaded(*l);
    }
    *l = negated(*l);
}

/// `physics::limit_motion`: `infinite` is told the motion when a part is no number or
/// infinite (the original logs it), before the three are zeroed; `top` is `top_speed`.
pub fn limit_motion(m: &mut Moving, top: f32, infinite: impl FnOnce([f32; 3])) -> Limited {
    let mut exponent = 0;
    let mut bad = false;
    for part in m.motion {
        exponent = part.to_bits() & 0x7F80_0000;
        if exponent == 0x7F80_0000 {
            bad = true;
            break;
        }
    }
    if bad {
        infinite(m.motion);
        m.motion = [0.0; 3];
        m.local = [0.0; 3];
        m.spin = [0.0; 3];
    }
    for k in 0..3 {
        hold(&mut m.spin[k], &mut m.spin_limit);
    }
    for k in [0, 2, 1] {
        hold(&mut m.motion[k], &mut m.motion_limit);
    }
    let length = length(m.motion) as f32;
    let mut factor = None;
    // `fld top; fcomp length`: on unless top is not below the length (or no number).
    if !matches!(
        top.partial_cmp(&length),
        Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)
    ) || top.is_nan()
    {
        let f = top as f64 / length as f64;
        m.motion = scale(m.motion, f);
        m.local = scale(m.local, f);
        factor = Some(f as f32);
    }
    Limited {
        exponent,
        length,
        factor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holds() {
        let (mut v, mut l) = (7.0, 5.0);
        hold(&mut v, &mut l);
        assert_eq!((v, l), (5.0, 5.0));
        let mut v = -9.0;
        hold(&mut v, &mut l);
        assert_eq!(v, -5.0);
        let mut v = f32::NAN;
        hold(&mut v, &mut l);
        assert_eq!(v, -5.0, "unordered: below both times");
    }

    #[test]
    fn limits() {
        let mut m = Moving {
            spin: [1.0, -8.0, 2.0],
            motion: [3.0, 4.0, 0.0],
            local: [6.0, 8.0, 0.0],
            spin_limit: 5.0,
            motion_limit: 10.0,
        };
        let mut told = None;
        let out = limit_motion(&mut m, 2.5, |v| told = Some(v));
        assert_eq!(told, None);
        assert_eq!(m.spin, [1.0, -5.0, 2.0]);
        assert_eq!(out.length, 5.0);
        assert_eq!(out.factor, Some(0.5));
        assert_eq!(m.motion, [1.5, 2.0, 0.0]);
        assert_eq!(m.local, [3.0, 4.0, 0.0]);
        assert_eq!(out.exponent, 0);
        m.motion[1] = f32::INFINITY;
        let out = limit_motion(&mut m, 100.0, |v| told = Some(v));
        assert_eq!(told, Some([1.5, f32::INFINITY, 0.0]));
        assert_eq!(out.exponent, 0x7F80_0000);
        assert_eq!((m.motion, m.spin, out.factor), ([0.0; 3], [0.0; 3], None));
    }
}
