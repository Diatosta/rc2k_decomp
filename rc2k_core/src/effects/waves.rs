//! The marks' wave tables, made once: the 0x11 wave levels, whole numbers, made floats, and a table
//! of 0x140 sines. As in `math::vector`, what the x87 keeps on its stack is an `f64`, rounded to
//! `f32` where it is stored; the sines are `fsin`'s (`scene::trig`): the last bit may differ
//! at a rounding edge.

use crate::scene::trig::sine;

/// The levels and the sines.
pub const LEVELS: usize = 0x11;
pub const SINES: usize = 0x140;

/// A level as a float: `n / scale - less` in `f64`, stored as `f32`.
pub fn level(n: i32, scale: f32, less: f32) -> f32 {
    (n as f64 / scale as f64 - less as f64) as f32
}

/// The sines of 0, step, 2 step ..: the angle summed in `f64`, never rounded to
/// a float.
pub fn sines(step: f32) -> [f32; SINES] {
    let mut x = 0.0f64;
    std::array::from_fn(|_| {
        let s = sine(x) as f32;
        x += step as f64;
        s
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waves() {
        assert_eq!(level(512, 256.0, 1.0), 1.0);
        assert_eq!(level(-256, 256.0, 1.0), -2.0);
        let s = sines(std::f32::consts::FRAC_PI_2);
        assert_eq!((s[0], s[1]), (0.0, 1.0));
        assert!(s[2].abs() < 1e-6);
    }
}
