//! The marks' wave tables, made once (converted 2026-10-21 from `effects::make_wave_tables`
//! 0x4C4E2F): the 0x11 wave levels, whole numbers, made floats, and a table of 0x140
//! sines. As in `math::vector`, what the x87 keeps on its stack is an `f64`, rounded to
//! `f32` where the original stores it; the sines are `fsin`'s (`scene::trig`): the last bit
//! may differ at a rounding edge.

use crate::scene::trig::sine;

/// The levels and the sines.
pub const LEVELS: usize = 0x11;
pub const SINES: usize = 0x140;

/// Whether the tables are to be made, by the mark (`g_marksWaveDone`): 0 not yet; where
/// the original goes on instead (`nullsub_23`) made before; anything else it would jump to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    ToMake,
    Made,
    Elsewhere,
}

pub fn mark(value: u32, made: u32) -> Mark {
    if value == made {
        Mark::Made
    } else if value == 0 {
        Mark::ToMake
    } else {
        Mark::Elsewhere
    }
}

/// `fild n; fdiv scale; fsub less`: a level as a float.
pub fn level(n: i32, scale: f32, less: f32) -> f32 {
    (n as f64 / scale as f64 - less as f64) as f32
}

/// The sines of 0, step, 2 step ..: the angle summed on the x87's stack, never rounded to
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
        assert_eq!(mark(5, 5), Mark::Made);
        assert_eq!(mark(0, 5), Mark::ToMake);
        assert_eq!(mark(7, 5), Mark::Elsewhere);
        assert_eq!(level(512, 256.0, 1.0), 1.0);
        assert_eq!(level(-256, 256.0, 1.0), -2.0);
        let s = sines(std::f32::consts::FRAC_PI_2);
        assert_eq!((s[0], s[1]), (0.0, 1.0));
        assert!(s[2].abs() < 1e-6);
    }
}
