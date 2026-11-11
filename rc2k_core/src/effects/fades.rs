//! The marks' fade colours (from `effects::make_fade_colours`
//! 0x4C37EF): 0x11 colours from one toward white, each component's inverse (0xFF less it)
//! times a step growing from 0, rounded, made a pixel (alpha 0xFF) and added to the colour
//! as one dword (carries cross components). As in `math::geometry`, what the x87 keeps on
//! its stack is an `f64`: the same at the game's double precision.

use crate::math::geometry::fistp64;

/// The colours made.
pub const FADES: usize = 0x11;

/// A colour's three inverses (red, green, blue).
pub fn inverses(colour: u32) -> [u32; 3] {
    [16, 8, 0].map(|s| ((colour >> s) & 0xFF) ^ 0xFF)
}

/// The fades: what is left after them (the step, the last three scaled) and the colours.
#[derive(Clone, Debug, PartialEq)]
pub struct Fades {
    pub colours: [u32; FADES],
    pub step: f32,
    pub scaled: [i32; 3],
}

/// The fades of `colour` with the step growing by `growth` (`fld step; fadd growth;
/// fstp`), each inverse times the step (`fild; fmul; fistp`).
pub fn fades(colour: u32, growth: f32) -> Fades {
    let inv = inverses(colour);
    let mut step = 0.0f32;
    let mut scaled = [0i32; 3];
    let colours = std::array::from_fn(|_| {
        scaled = inv.map(|v| fistp64(v as i32 as f64 * step as f64));
        let [r, g, b] = scaled.map(|v| v as u32 & 0xFF);
        step = (step as f64 + growth as f64) as f32;
        (0xFF00_0000 | r << 16 | g << 8 | b).wrapping_add(colour)
    });
    Fades {
        colours,
        step,
        scaled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fade() {
        assert_eq!(inverses(0x0080_4020), [0x7F, 0xBF, 0xDF]);
        let f = fades(0, 0.0625);
        assert_eq!(f.colours[0], 0xFF00_0000);
        assert_eq!(f.colours[1], 0xFF10_1010);
        assert_eq!(f.colours[16], 0xFFFF_FFFF);
        assert_eq!((f.step, f.scaled), (1.0625, [255; 3]));
        assert_eq!(fades(0xFF, 0.0625).colours[1], 0xFF10_10FF);
        // Past 1 the sum carries into the next component.
        assert_eq!(fades(0x80, 2.0).colours[1], 0xFFFE_FF7E);
    }
}
