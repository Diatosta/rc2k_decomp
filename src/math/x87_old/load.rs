//! More x87 helpers (see `math/x87.rs`), kept apart so that work on the race's loading does
//! not edit the shared file.
//!
//! What the x87 holds on its stack is an `X` (an `f64` that rounds as the control word
//! says, `x87_float.rs`), what it stores as a dword float `st`, `fild` is `int`, `fistp` /
//! `fist` `to_int`. Their comparison with the x87's instructions is frozen in
//! `x87_frozen_c.rs`.

use crate::math::x87_float::{X, compare, f80, from_f80, int, ld, st, to_int};

/// `sqrt(a a + b b)` as a float: the length of two floats at right angles.
pub fn float_hypot(a: u32, b: u32) -> u32 {
    let a2 = ld(a) * ld(a);
    let b2 = ld(b) * ld(b);
    st((a2 + b2).sqrt())
}

/// C0 of the compare of `a` with `b`: a below b, or one is no number.
pub fn below_or_unordered(a: u32, b: u32) -> bool {
    compare(ld(a), ld(b)) & 0x0100 != 0
}

/// C0 of the compare of `|a h| - k` (kept at the x87's precision) with `b`: below, or one
/// is no number.
pub fn scaled_abs_less_below(a: u32, h: u32, k: u32, b: u32) -> bool {
    let x = (ld(a) * ld(h)).abs() - ld(k);
    compare(x, ld(b)) & 0x0100 != 0
}

/// The point (three whole numbers) through `matrix` (16 floats; the 12 used are the
/// columns' at 0, 4, 8 / 0x10, 0x14, 0x18 / 0x20, 0x24, 0x28 and the translation at 0x30,
/// 0x34, 0x38): m0 x + (m4 y + m8 z) and so on, then the translation. Left at double
/// precision.
fn rotated(point: [i32; 3], matrix: &[u32]) -> [X; 3] {
    let [x, y, z] = point.map(int);
    let m = |i: usize| ld(matrix[i]);
    let s0 = m(0) * x + (m(4) * y + m(8) * z) + m(12);
    let s1 = m(1) * x + (m(5) * y + m(9) * z) + m(13);
    let s2 = x * m(2) + (y * m(6) + z * m(10)) + m(14);
    [s0, s1, s2]
}

/// `scene::project_point::rotate` with its third store a `fistp`: the point (three whole
/// numbers) through `matrix`, x and y stored as floats at `out[0]`, `out[1]`, z rounded
/// into the third from the x87's precision.
pub fn rotate_rounding_z(point: [i32; 3], matrix: &[u32]) -> [u32; 3] {
    let s = rotated(point, matrix);
    [st(s[0]), st(s[1]), to_int(s[2]) as u32]
}

/// `a a + b b` of two whole numbers, as a float.
pub fn int_square_sum(a: i32, b: i32) -> u32 {
    let (a, b) = (int(a), int(b));
    st(a * a + b * b)
}

/// `a a + b b` of two floats.
pub fn float_square_sum(a: u32, b: u32) -> u32 {
    let a2 = ld(a) * ld(a);
    let b2 = ld(b) * ld(b);
    st(a2 + b2)
}

/// `(a + b) c`, a whole number plus a float, times a float.
pub fn int_plus_times(a: i32, b: u32, c: u32) -> u32 {
    st((int(a) + ld(b)) * ld(c))
}

/// C0 of the compare of `|a|` with `k`: |a| below k, or one is no number.
pub fn abs_below_or_unordered(a: u32, k: u32) -> bool {
    compare(ld(a).abs(), ld(k)) & 0x0100 != 0
}

/// `scene::project_point::rotate` with all three stores `fistp`: the point (three whole
/// numbers) through `matrix`, each part rounded from the x87's precision.
pub fn rotate_rounding(point: [i32; 3], matrix: &[u32]) -> [u32; 3] {
    rotated(point, matrix).map(|part| to_int(part) as u32)
}

/// `(a a + b b) k` (`times` = 1) or `(a a + b b) k k` (`times` = 2) as a whole number.
pub fn int_square_sum_scaled_to_int(a: i32, b: i32, k: u32, times: u32) -> u32 {
    let (a, b) = (int(a), int(b));
    let mut x = (a * a + b * b) * ld(k);
    if times == 2 {
        x *= ld(k);
    }
    to_int(x) as u32
}

/// The camera's roll eased towards a target (`race::view_matrices`), rounded once at the end:
/// `((((i / a / a / b) v c) - m) h) + m`.
pub fn eased_roll(i: i32, a: u32, b: u32, v: u32, c: u32, m: u32, h: u32) -> u32 {
    let mut x = int(i);
    x /= ld(a);
    x /= ld(a);
    x /= ld(b);
    x *= ld(v);
    x *= ld(c);
    x -= ld(m);
    x *= ld(h);
    x += ld(m);
    st(x)
}

/// The sun's direction in `race::draw_sky`: with `p = (k - a) s` and `q = b s`, cos p cos q,
/// sin q and sin p cos q, each times `m` and stored as a float.
pub fn sun_direction(a: i32, k: u32, s: u32, b: i32, m: u32) -> [u32; 3] {
    let p = (-int(a) + ld(k)) * ld(s);
    let q = int(b) * ld(s);
    let (sin_p, cos_p) = p.sin_cos();
    let (sin_q, cos_q) = q.sin_cos();
    let cos_cos = cos_p * cos_q;
    let sin_cos = sin_p * cos_q;
    [st(cos_cos * ld(m)), st(sin_q * ld(m)), st(sin_cos * ld(m))]
}

/// A float as the x87's 80 bits.
pub fn to_tbyte(a: u32) -> [u8; 10] {
    f80(ld(a))
}

/// 80 bits rounded to a whole number as the x87 is set to round.
pub fn tbyte_to_int(a: [u8; 10]) -> u32 {
    to_int(from_f80(a)) as u32
}

/// A point of whole numbers onto the screen as in `race::draw_sky`: `v` = x, y, z, the flat
/// depth, the scale, the centre's y and x. Flat: z dropped (into `scratch`, as a float)
/// and the scale over the flat depth + 1; else the scale over z. Gives x as a float and
/// rounded, y as a float and rounded (each `x r + cx` and `cy - y r`, r the ratio).
pub fn sun_on_screen(
    v: &[u32; 7],
    flat: bool,
    scratch: &impl crate::memory::session::Cell32,
) -> [u32; 4] {
    let (x, y, z) = (int(v[0] as i32), int(v[1] as i32), int(v[2] as i32));
    let ratio = if flat {
        scratch.set_bits(st(z));
        ld(v[4]) / (int(v[3] as i32) + X(1.0))
    } else {
        ld(v[4]) / z
    };
    let sx = x * ratio + ld(v[6]);
    let sy = -(y * ratio) + ld(v[5]);
    [st(sx), to_int(sx) as u32, st(sy), to_int(sy) as u32]
}

/// `k` less half the distance from the centre, rounded, as a whole number: `-(sqrt((x -
/// cx)^2 + (y - cy)^2) h - k)` (`fistp`).
pub fn sun_size(x: u32, cx: u32, y: u32, cy: u32, h: u32, k: u32) -> u32 {
    let dx = ld(x) - ld(cx);
    let dx2 = dx * dx;
    let dy = ld(y) - ld(cy);
    let dy2 = dy * dy;
    let d = (dx2 + dy2).sqrt() * ld(h) - ld(k);
    to_int(-d) as u32
}
