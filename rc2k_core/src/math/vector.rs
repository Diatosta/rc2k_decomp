//! Three-float vectors and 3 x 3 (or 4 x 3) float matrices: rows of three floats;
//! `rotate_point`'s rows have four, the fourth the translation. As in `geometry`, x87 stack
//! values are `f64`, rounded to `f32` where stored.

/// `x x' + (y y' + z z')`, as an `f64`.
pub fn dot(a: [f32; 3], b: [f32; 3]) -> f64 {
    let (a, b) = (a.map(f64::from), b.map(f64::from));
    a[0] * b[0] + (a[1] * b[1] + a[2] * b[2])
}

/// The length, `sqrt(x x + (y y + z z))`, as an `f64`.
pub fn length(v: [f32; 3]) -> f64 {
    dot(v, v).sqrt()
}

/// Each float times one over the length.
pub fn normalise(v: [f32; 3]) -> [f32; 3] {
    let reciprocal = 1.0 / length(v);
    v.map(|x| (x as f64 * reciprocal) as f32)
}

/// Each float times `factor`.
pub fn scale(v: [f32; 3], factor: f64) -> [f32; 3] {
    v.map(|x| (x as f64 * factor) as f32)
}

/// Each float times its partner.
pub fn multiply_parts(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2]]
}

/// A row times a column (`b`'s three floats a row apart).
pub fn row_times_column(row: [f32; 3], column: [f32; 3]) -> f32 {
    dot(row, column) as f32
}

/// A point turned by the matrix `m` and moved by its translation, as `f64`s.
/// `m` is the game's 4 x 4 layout, four floats a row (the last float of each row is not
/// read): the point times the rows 0 .. 2, the translation the fourth row (12 .. 14).
pub fn rotate_point(m: &[f32; 15], p: [f64; 3]) -> [f64; 3] {
    let m = m.map(f64::from);
    let [x, y, z] = p;
    [
        x * m[0] + (y * m[4] + z * m[8]) + m[12],
        x * m[1] + (y * m[5] + z * m[9]) + m[13],
        x * m[2] + (z * m[10] + y * m[6]) + m[14],
    ]
}

/// A point of three whole numbers turned by the 3 x 3 matrix `m`, rounded to the nearest.
pub fn turn_int_point(p: [i32; 3], m: &[f32; 9]) -> [i32; 3] {
    use crate::math::geometry::fistp64;
    let m = m.map(f64::from);
    let [x, y, z] = p.map(f64::from);
    [
        fistp64(x * m[0] + (y * m[1] + z * m[2])),
        fistp64(x * m[3] + (y * m[4] + z * m[5])),
        fistp64(y * m[7] + (x * m[6] + z * m[8])),
    ]
}

/// The axis a turn is about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

/// The x87's "indefinite" (an invalid operation's NaN) as a float.
const INDEFINITE: u32 = 0xFFC0_0000;

/// The 3 x 3 matrix of a turn by `angle` about `axis`.
/// A NaN angle gives itself as both; an infinite one the indefinite.
pub fn turn_matrix(axis: Axis, angle: f64) -> [f32; 9] {
    let (s, c) = if angle.is_nan() {
        (angle as f32, angle as f32)
    } else if angle.is_infinite() {
        (f32::from_bits(INDEFINITE), f32::from_bits(INDEFINITE))
    } else {
        let (s, c) = super::portable::sin_cos(angle);
        (s as f32, c as f32)
    };
    match axis {
        Axis::X => [1.0, 0.0, 0.0, 0.0, c, s, 0.0, -s, c],
        Axis::Y => [c, 0.0, s, 0.0, 1.0, 0.0, -s, 0.0, c],
        Axis::Z => [c, s, 0.0, -s, c, 0.0, 0.0, 0.0, 1.0],
    }
}

/// The identity.
pub const IDENTITY: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

/// The constants of `search_angle`: where the search starts (pi / 2), its first step
/// (pi / 4) and the step's factor (0.5).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Search {
    pub start: f32,
    pub step: f32,
    pub half: f32,
}

#[cfg(test)]
/// The angle whose cosine is `cosine` (the two vectors' dot product, normalised first),
/// found by 16 halvings, and the last compare's status (0x100 for less, 0x500 for
/// unordered, else 0).
pub(crate) fn search_angle(cosine: f32, k: Search) -> (f32, u32) {
    let d = cosine as f64;
    let (mut angle, mut step, mut status) = (k.start, k.step, 0);
    for _ in 0..16 {
        let c = super::portable::cos(angle as f64);
        status = if c.is_nan() || d.is_nan() {
            0x500
        } else if c < d {
            0x100
        } else {
            0
        };
        match status {
            0x400 => break,
            0x100 => angle -= step,
            _ => angle += step,
        }
        step *= k.half;
    }
    (angle, status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vectors() {
        assert_eq!(dot([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]), 32.0);
        assert_eq!(length([3.0, 4.0, 0.0]), 5.0);
        assert_eq!(normalise([0.0, 3.0, 4.0]), [0.0, 0.6, 0.8]);
        // A zero vector: 0 times infinity, the indefinite.
        assert!(normalise([0.0; 3]).iter().all(|x| x.is_nan()));
        let m = turn_matrix(Axis::Z, std::f64::consts::FRAC_PI_2);
        assert_eq!(m[1], 1.0);
        assert_eq!(m[3], -1.0);
        let k = Search {
            start: std::f32::consts::FRAC_PI_2,
            step: std::f32::consts::FRAC_PI_4,
            half: 0.5,
        };
        let cosine = dot(normalise([1.0, 0.0, 0.0]), normalise([1.0, 1.0, 0.0])) as f32;
        let (angle, _) = search_angle(cosine, k);
        assert!((angle - std::f32::consts::FRAC_PI_4).abs() < 1e-4);
        let mut m = [0.0f32; 15];
        m[0] = 1.0;
        m[5] = 1.0;
        m[10] = 1.0;
        m[12] = 10.0;
        assert_eq!(rotate_point(&m, [1.0, 2.0, 3.0])[0], 11.0);
        assert_eq!(turn_int_point([1, 2, 3], &IDENTITY), [1, 2, 3]);
    }
}
