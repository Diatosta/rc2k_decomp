//! The marks' scaled settings (converted 2026-10-21 from `marks::scale_axes` 0x4BFD73 and
//! `marks::scale_setting` 0x4C08BB). As in `math::vector`, what the x87 keeps on its stack
//! is an `f64`, rounded to `f32` where the original stores it: the same bits at the game's
//! double precision.

/// `fild n; fmul k; fstp`: a setting's whole number scaled.
pub fn scaled_setting(n: i32, factor: f32) -> f32 {
    (n as f64 * factor as f64) as f32
}

/// What `scale_axes` reads: the two axes (`flt_89CF1A`, `flt_89CF26`), the factor of
/// both (`flt_89CECA`), one factor each (`flt_89CEEE`, `flt_89CEF2`) and two sets of
/// per-part factors (`flt_8BC1DE` .., `flt_8BC1F6` ..).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Axes {
    pub axes: [[f32; 3]; 2],
    pub factor: f32,
    pub own: [f32; 2],
    pub parts: [[f32; 3]; 2],
    pub second_parts: [[f32; 3]; 2],
}

/// What it writes, in its order: both axes by the factor (`flt_89CF3E` ..), each by its
/// own factor and its parts' (`flt_89CF6E` ..), and the first results by the second
/// parts (`flt_89CF56` ..).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScaledAxes {
    pub scaled: [[f32; 3]; 2],
    pub weighted: [[f32; 3]; 2],
    pub second: [[f32; 3]; 2],
}

/// `marks::scale_axes`.
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
}
