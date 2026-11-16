//! The four wheels' springs from their heights: each wheel's squeeze, its force and damping, and
//! the wheel's place for the drawing. Floats: the x87's stack values are `f64`, rounded to `f32`
//! where stored.

use super::x87::{below, equal, fistp64, hold_low, loaded};

/// The springs' factors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringFactors {
    /// 65536.
    pub scale: f32,
    /// The spring's range.
    pub range: f32,
    /// 0: the squeeze's low end.
    pub zero: f32,
    /// 1: the squeeze's high end.
    pub one: f32,
    /// The force's factors.
    pub stiffness: f32,
    pub load: f32,
    /// The damping's; `rebound` its share when below 0.
    pub damping: f32,
    pub rebound: f32,
}

/// One wheel's spring.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    /// The squeeze, 0 ..= 1.
    pub squeeze: f32,
    /// The force, the damping added unless the squeeze is 0.
    pub force: f32,
    /// The damping.
    pub damping: f32,
}

/// One wheel's spring from its height (a whole number) and its last squeeze.
pub fn spring(height: u32, last: f32, f: &SpringFactors) -> Spring {
    let depth = height.wrapping_neg() as i32 as f64;
    let mut squeeze = hold_low((depth / f.scale as f64 / f.range as f64) as f32, f.zero);
    if !below(squeeze as f64, f.one as f64) {
        squeeze = loaded(f.one);
    }
    let q = squeeze as f64;
    let mut force = (-(q * f.stiffness as f64 * f.load as f64)) as f32;
    let mut damping = ((q - loaded(last) as f64) * f.damping as f64) as f32;
    if below(damping as f64, f.zero as f64) {
        damping = (damping as f64 * f.rebound as f64) as f32;
    }
    if !equal(q, f.zero as f64) {
        force = (damping as f64 + force as f64) as f32;
    }
    Spring {
        squeeze,
        force,
        damping,
    }
}

/// The wheel's place: ((a - b + c) + (squeeze - d) x range) x scale, rounded.
pub fn wheel_place(abcd: [f32; 4], squeeze: f32, range: f32, scale: f32) -> i32 {
    let [a, b, c, d] = abcd.map(f64::from);
    fistp64(((a - b + c) + (squeeze as f64 - d) * range as f64) * scale as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    const F: SpringFactors = SpringFactors {
        scale: 65536.0,
        range: 1.0,
        zero: 0.0,
        one: 1.0,
        stiffness: 2.0,
        load: 3.0,
        damping: 4.0,
        rebound: 0.5,
    };

    #[test]
    fn springs() {
        // Height -0x8000: half squeezed; from 0.25 the damping (0.25 x 4) is added.
        let s = spring(0xFFFF_8000, 0.25, &F);
        assert_eq!(
            s,
            Spring {
                squeeze: 0.5,
                force: -2.0,
                damping: 1.0
            }
        );
        // Back down: the damping below 0 halved.
        let s = spring(0xFFFF_C000, 0.5, &F);
        assert_eq!((s.squeeze, s.damping), (0.25, -0.5));
        // Off the ground: squeeze 0, the damping not added.
        let s = spring(0x1000, 0.5, &F);
        assert_eq!(
            s,
            Spring {
                squeeze: 0.0,
                force: -0.0,
                damping: -1.0
            }
        );
        // Past the range: held at 1.
        assert_eq!(spring(0xFFFE_0000, 1.0, &F).squeeze, 1.0);
        assert_eq!(wheel_place([1.0, 0.5, 0.25, 0.5], 1.0, 2.0, 4.0), 7);
    }
}
