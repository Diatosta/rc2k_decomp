//! The marks' scaled settings. As in `math::vector`, what the x87 keeps on its stack is an `f64`,
//! rounded to `f32` where it is stored: the same bits at the game's double precision.

/// A setting's whole number times `factor` in `f64`, stored as `f32`.
pub fn scaled_setting(n: i32, factor: f32) -> f32 {
    (n as f64 * factor as f64) as f32
}

/// What `scale_axes` reads: the two axes, the factor of both, one factor each and two sets of
/// per-part factors (`WindscreenFlakeState.axis_factors[0]` .., `WindscreenFlakeState.halves[0]`
/// ..).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Axes {
    pub axes: [[f32; 3]; 2],
    pub factor: f32,
    pub own: [f32; 2],
    pub parts: [[f32; 3]; 2],
    pub second_parts: [[f32; 3]; 2],
}

/// What it writes, in its order: both axes by the factor
/// (`WindscreenFlakeTables.scaled_by_radius[0][0]` ..), each by its own factor and its parts'
/// (`WindscreenFlakeTables.scaled_axes[0][0]` ..), and the first results by the second parts
/// (`WindscreenFlakeTables.half_axes[0][0]` ..).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScaledAxes {
    pub scaled: [[f32; 3]; 2],
    pub weighted: [[f32; 3]; 2],
    pub second: [[f32; 3]; 2],
}

/// `effects::scale_axes`.
pub fn scale_axes(a: &Axes) -> ScaledAxes {
    let times = |x: f32, k: f32| (x as f64 * k as f64) as f32;
    let scaled = a.axes.map(|v| v.map(|x| times(x, a.factor)));
    let weighted = std::array::from_fn(|n| {
        std::array::from_fn(|k| {
            (a.axes[n][k] as f64 * a.own[n] as f64 * a.parts[n][k] as f64) as f32
        })
    });
    let second =
        std::array::from_fn(|n| std::array::from_fn(|k| times(scaled[n][k], a.second_parts[n][k])));
    ScaledAxes {
        scaled,
        weighted,
        second,
    }
}

/// The corner records (0x100 of 0x80 bytes, words) and the word pairs scaled, in the order they
/// are made: the words at 0, 4, 0xC, 8 of the record.
pub const CORNERS: u32 = 0x100;
pub const CORNER_PAIRS: [u32; 4] = [0, 4, 0xC, 8];

/// A corner record scaled (`scale_corners`): the four pairs' words times the scale
/// (each word as an unsigned 16-bit number), the two sides (the sixteen-bit differences of
/// the first and second pairs' words) and the slope, the second side over the first.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Corner {
    pub values: [f32; 8],
    pub sides: [u32; 2],
    pub slope: f32,
}

#[cfg(test)]
pub(crate) fn scale_corner(word: impl Fn(u32) -> u16, scale: f32) -> Corner {
    let mut values = [0.0f32; 8];
    for (k, &o) in CORNER_PAIRS.iter().enumerate() {
        for h in 0..2 {
            values[2 * k + h] = (word(o + 2 * h as u32) as f64 * scale as f64) as f32;
        }
    }
    let side = |a: u32, b: u32| {
        let d = word(a).wrapping_sub(word(b));
        (if (d as i16) < 0 { d.wrapping_neg() } else { d }) as u32
    };
    let sides = [side(4, 0), side(0xA, 2)];
    Corner {
        values,
        sides,
        slope: (sides[1] as i32 as f64 / sides[0] as i32 as f64) as f32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scales() {
        assert_eq!(scaled_setting(-7, 3.25), -22.75);
        assert_eq!(scaled_setting(i32::MIN, -2.0), 4294967296.0);
        let a = Axes {
            axes: [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]],
            factor: 2.0,
            own: [0.5, 3.0],
            parts: [[1.0, 2.0, 3.0], [1.0, 1.0, 0.5]],
            second_parts: [[1.0, 0.5, 0.25], [0.0; 3]],
        };
        let s = scale_axes(&a);
        assert_eq!(s.scaled, [[2.0, 4.0, 6.0], [8.0, 10.0, 12.0]]);
        assert_eq!(s.weighted, [[0.5, 2.0, 4.5], [12.0, 15.0, 9.0]]);
        assert_eq!(s.second, [[2.0, 2.0, 1.5], [0.0; 3]]);
    }

    #[test]
    fn corner() {
        let words = [10u16, 20, 14, 26, 0, 0, 0, 0];
        let c = scale_corner(|o| words[(o / 2) as usize], 0.5);
        assert_eq!(c.values, [5.0, 10.0, 7.0, 13.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(c.sides, [4, 20]);
        assert_eq!(c.slope, 5.0);
        let flat = scale_corner(|o| if o == 0 { 0xFFFF } else { 0 }, 1.0);
        assert_eq!(flat.sides, [1, 0]);
        assert_eq!(flat.values[0], 65535.0);
    }
}
