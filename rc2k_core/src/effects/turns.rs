//! Small float steps of the masks' making (converted 2026-10-21 from `effects::set_line_turn`
//! 0x4BF432, `set_line_turn_back` 0x4BF466, `turn_pair` 0x4BF409, `scale_rows3` 0x4BF498,
//! `smooth_steps` 0x4BE928, `adjugate3` 0x4BCED6 and `clamp_pair` 0x4BEF22): the 2 x 2
//! turn a line gives, a pair turned by it, rows scaled to set lengths, two cosine-eased
//! steps, a 3 x 3 adjugate, a pair held to whole numbers.
//!
//! As in `math::vector`, what the x87 keeps on its stack is an `f64`, rounded to `f32`
//! where the original stores it (the same bits at the game's double precision, but for
//! the cosines: `scene::trig`).

use crate::math::vector;
use crate::scene::places::Places;
use crate::scene::trig;

/// The 2 x 2 turn a line gives: its four floats in their order in memory (m0 .. m3).
pub type LineTurn = [f32; 4];

/// The order the original writes the turn's floats in (dx, dy, -dy, dx).
pub const LINE_TURN_ORDER: [usize; 4] = [0, 2, 1, 3];

/// `effects::set_line_turn` (`back` false) and `set_line_turn_back` (true): from the line's
/// two ends, m0 and m3 dx, m2 minus dy (dy when `back`), m1 its negation.
pub fn line_turn(from: [f32; 2], to: [f32; 2], back: bool) -> LineTurn {
    let dx = (to[0] as f64 - from[0] as f64) as f32;
    let dy = to[1] as f64 - from[1] as f64;
    let dy = if back { dy as f32 } else { -dy as f32 };
    [dx, -dy, dy, dx]
}

/// `effects::turn_pair`: the pair (st(0) a, st(1) b) turned by the line's turn: (a m3 + b m2,
/// b m0 + a m1), unrounded.
pub fn turn_pair(m: LineTurn, a: f64, b: f64) -> (f64, f64) {
    let m = m.map(f64::from);
    (a * m[3] + b * m[2], b * m[0] + a * m[1])
}

/// One row of three floats made `length` long: `length` over the row's length (the last
/// two squares added first), each times that, stored.
pub fn scale_to_length(row: [f32; 3], length: f32) -> [f32; 3] {
    let factor = length as f64 / vector::length(row);
    row.map(|x| (factor * x as f64) as f32)
}

/// `effects::scale_rows3`: the three rows each made the length of their own (a row of no
/// length gives NaNs, as there).
pub fn scale_rows(rows: [[f32; 3]; 3], lengths: [f32; 3]) -> [[f32; 3]; 3] {
    std::array::from_fn(|n| scale_to_length(rows[n], lengths[n]))
}

/// `fldpi; fidiv c; fimul m; fcos; fchs; fmul t; fadd a; fadd t`, stored: a + t - t
/// cos(pi m / c). (`fldpi` has 64 bits of pi; the `f64`'s 53 bits may make the angle's
/// last bit differ, far under what the float keeps.)
pub fn cosine_step(c: i32, m: i32, t: f32, a: f32) -> f32 {
    let angle = std::f64::consts::PI / c as f64 * m as f64;
    let (t, a) = (t as f64, a as f64);
    (-trig::cosine(angle) * t + a + t) as f32
}

/// `effects::smooth_steps`' two results (ecx, ebx) from step `n`, a (ebx) and b (ecx): t
/// (b - a) times `ease` (stored), then a + t - t cos(pi (n + 1) / steps[0]) and the same
/// with n over steps[1].
pub fn smooth_pair(n: u32, a: f32, b: f32, ease: f32, steps: [i32; 2]) -> (f32, f32) {
    let t = ((b as f64 - a as f64) * ease as f64) as f32;
    (
        cosine_step(steps[0], n.wrapping_add(1) as i32, t, a),
        cosine_step(steps[1], n as i32, t, a),
    )
}

/// `effects::adjugate3`: the adjugate of the 3 x 3 matrix at `s` (rows 12 bytes apart) into
/// `d`: each entry a difference of two products, stored before the next is worked out.
pub fn adjugate(p: &mut impl Places, s: u32, d: u32) {
    // (a, b, c, e): the entry is s[a] s[b] - s[c] s[e].
    const TERMS: [[u32; 4]; 9] = [
        [4, 8, 5, 7],
        [2, 7, 1, 8],
        [1, 5, 2, 4],
        [5, 6, 3, 8],
        [0, 8, 2, 6],
        [2, 3, 0, 5],
        [3, 7, 4, 6],
        [1, 6, 0, 7],
        [0, 4, 1, 3],
    ];
    for (n, k) in TERMS.into_iter().enumerate() {
        let f = k.map(|i| p.float(s.wrapping_add(4 * i)) as f64);
        p.set_float(
            d.wrapping_add(4 * n as u32),
            (f[0] * f[1] - f[2] * f[3]) as f32,
        );
    }
}

/// `effects::clamp_pair`: st(0) times `k[0]` rounded (`fistp`) and held to 0 .. 0x7FFF
/// (ebx), st(1) times `k[1]` held to 0 .. 0xFFFF (eax); then each over `k[3]` (st(0))
/// and `k[2]` (st(1)), kept on the stack: (ebx, eax, [st(0), st(1)]).
pub fn clamp_pair(st0: f64, st1: f64, k: [f32; 4]) -> (u32, u32, [f64; 2]) {
    use crate::math::geometry::fistp64;
    let b = fistp64(st0 * k[0] as f64).clamp(0, 0x7FFF);
    let a = fistp64(st1 * k[1] as f64).clamp(0, 0xFFFF);
    (
        b as u32,
        a as u32,
        [b as f64 / k[3] as f64, a as f64 / k[2] as f64],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::places::TestPlaces;

    #[test]
    fn line_and_pair() {
        let m = line_turn([1.0, 2.0], [4.0, 6.0], false);
        assert_eq!(m, [3.0, 4.0, -4.0, 3.0]);
        assert_eq!(
            line_turn([1.0, 2.0], [4.0, 6.0], true),
            [3.0, -4.0, 4.0, 3.0]
        );
        // (1, 0) turned: (m2, m0).
        assert_eq!(turn_pair(m, 0.0, 1.0), (-4.0, 3.0));
        assert_eq!(turn_pair(m, 1.0, 0.0), (3.0, 4.0));
    }

    #[test]
    fn rows_and_steps() {
        let r = scale_rows(
            [[3.0, 0.0, 4.0], [0.0, 2.0, 0.0], [0.0; 3]],
            [10.0, 1.0, 1.0],
        );
        assert_eq!(r[0], [6.0, 0.0, 8.0]);
        assert_eq!(r[1], [0.0, 1.0, 0.0]);
        assert!(r[2].iter().all(|x| x.is_nan()));
        // Step 0 of 4: a + t - t cos(pi / 4) and a + t - t cos(0) = a.
        let (c, b) = smooth_pair(0, 1.0, 3.0, 0.5, [4, 4]);
        assert_eq!(b, 1.0);
        let t = 1.0f64;
        assert_eq!(
            c,
            (-crate::math::portable::cos(std::f64::consts::FRAC_PI_4) * t + 1.0 + t) as f32
        );
        // The last step reaches a + 2t.
        assert_eq!(smooth_pair(3, 1.0, 3.0, 0.5, [4, 4]).0, 3.0);
    }

    #[test]
    fn adjugate_in_place() {
        let mut p = TestPlaces(vec![0; 9]);
        p.set_floats(0, &[2.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 4.0]);
        adjugate(&mut p, 0, 0);
        // Over the input: entry 0 is 12; entry 4 reads entry 0 written: 12 x 4 - 0.
        assert_eq!(p.floats::<9>(0)[..5], [12.0, 0.0, 0.0, 0.0, 48.0]);
    }

    #[test]
    fn pairs() {
        let k = [2.0, 4.0, 8.0, 16.0];
        assert_eq!(clamp_pair(10.25, 3.0, k), (20, 12, [1.25, 1.5]));
        assert_eq!(
            clamp_pair(-1.0, 1e5, k),
            (0, 0xFFFF, [0.0, 0xFFFF as f64 / 8.0])
        );
        // Past an i32 the integer indefinite, held to 0.
        assert_eq!(clamp_pair(0.0, 1e9, k), (0, 0, [0.0, 0.0]));
        assert_eq!(clamp_pair(f64::NAN, 1e5, k).0, 0);
    }
}
