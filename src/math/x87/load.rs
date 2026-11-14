//! More x87 helpers (see `math/x87.rs`), kept apart so that work on the race's loading does
//! not edit the shared file.
//!
//! What the x87 holds on its stack is an `X` (an `f64` that rounds as the control word
//! says, `x87_float.rs`), what it stores as a dword float `st`, `fild` is `int`, `fistp` /
//! `fist` `to_int`. Their comparison with the x87's instructions is frozen in
//! `x87_frozen_c.rs`.

use super::{Flt, out};
use crate::math::x87_float::{Rounds, X, at_rounding, compare, f80, from_f80, int, ld, st, to_int};

/// `(a * b) / c`, rounded once at the end as the x87 is set to round: a whole number times
/// a whole number over a float.
#[cfg(test)] // inlined at its caller; the frozen comparison keeps it
pub fn int_times_int_over_to_int(a: i32, b: i32, c: u32) -> i32 {
    to_int(int(a) * int(b) / ld(c))
}

/// `|a|` as a whole number: the size of a float.
#[cfg(test)] // inlined at its caller; the frozen comparison keeps it
pub fn abs_to_int(a: u32) -> i32 {
    to_int(ld(a).abs())
}

/// `sqrt(a a + b b)` as a float: the length of two floats at right angles.
pub fn float_hypot(a: impl Flt, b: impl Flt) -> f32 {
    at_rounding!(|r| float_hypot_with(r, a, b))
}

/// `float_hypot` at the rounding `r`.
pub(crate) fn float_hypot_with<R: Rounds>(r: R, a: impl Flt, b: impl Flt) -> f32 {
    let a = a.value();
    let b = b.value();
    let a2 = r.ld(a) * r.ld(a);
    let b2 = r.ld(b) * r.ld(b);
    r.stf(r.sqrt(a2 + b2))
}

/// C0 of the compare of `a` with `b`: a below b, or one is no number.
pub fn below_or_unordered(a: impl Flt, b: impl Flt) -> bool {
    let a = a.value();
    let b = b.value();
    compare(ld(a), ld(b)) & 0x0100 != 0
}

/// C0 of the compare of `|a h| - k` (kept at the x87's precision) with `b`: below, or one
/// is no number.
#[cfg(test)] // the callers use the `_with` form; the tests compare this one
pub fn scaled_abs_less_below(a: impl Flt, h: impl Flt, k: impl Flt, b: impl Flt) -> bool {
    at_rounding!(|r| scaled_abs_less_below_with(r, a, h, k, b))
}

/// `scaled_abs_less_below` at the rounding `r`.
pub(crate) fn scaled_abs_less_below_with<R: Rounds>(
    r: R,
    a: impl Flt,
    h: impl Flt,
    k: impl Flt,
    b: impl Flt,
) -> bool {
    let a = a.value();
    let h = h.value();
    let k = k.value();
    let b = b.value();
    let x = r.of(r.value(r.ld(a) * r.ld(h)).abs()) - r.ld(k);
    r.compare(x, r.ld(b)) & 0x0100 != 0
}

/// The point (three whole numbers) through `matrix` (16 floats; the 12 used are the
/// columns' at 0, 4, 8 / 0x10, 0x14, 0x18 / 0x20, 0x24, 0x28 and the translation at 0x30,
/// 0x34, 0x38): m0 x + (m4 y + m8 z) and so on, then the
/// translation. Left at double precision.
fn rotated<R: Rounds>(r: R, point: [i32; 3], matrix: &[u32]) -> [R::Reg; 3] {
    let [x, y, z] = point.map(|v| r.int(v));
    let m = |i: usize| r.ld(matrix[i]);
    let s0 = m(0) * x + (m(4) * y + m(8) * z) + m(12);
    let s1 = m(1) * x + (m(5) * y + m(9) * z) + m(13);
    let s2 = x * m(2) + (y * m(6) + z * m(10)) + m(14);
    [s0, s1, s2]
}

/// `scene::project_point::rotate` with its third store a whole number: the point (three whole
/// numbers) through `matrix`, x and y stored as floats at `out[0]`, `out[1]`, z rounded
/// into the third from the x87's precision.
pub fn rotate_rounding_z(point: [i32; 3], matrix: &[u32]) -> [u32; 3] {
    at_rounding!(|r| {
        let s = rotated(r, point, matrix);
        [r.st(s[0]), r.st(s[1]), r.to_int(s[2]) as u32]
    })
}

/// `a a + b b` of two whole numbers, as a float.
pub fn int_square_sum(a: i32, b: i32) -> f32 {
    let (a, b) = (int(a), int(b));
    out(a * a + b * b)
}

/// `a a + b b` of two floats.
pub fn float_square_sum(a: impl Flt, b: impl Flt) -> f32 {
    let a = a.value();
    let b = b.value();
    let a2 = ld(a) * ld(a);
    let b2 = ld(b) * ld(b);
    out(a2 + b2)
}

/// `(a + b) c`, a whole number plus a float, times a float.
pub fn int_plus_times(a: i32, b: impl Flt, c: impl Flt) -> f32 {
    let b = b.value();
    let c = c.value();
    out((int(a) + ld(b)) * ld(c))
}

/// C0 of the compare of `|a|` with `k`: |a| below k, or one is no number.
pub fn abs_below_or_unordered(a: impl Flt, k: impl Flt) -> bool {
    let a = a.value();
    let k = k.value();
    compare(ld(a).abs(), ld(k)) & 0x0100 != 0
}

/// `scene::project_point::rotate` with all three stores whole numbers: the point (three whole
/// numbers) through `matrix`, each part rounded from the x87's precision.
pub fn rotate_rounding(point: [i32; 3], matrix: &[u32]) -> [u32; 3] {
    at_rounding!(|r| rotated(r, point, matrix).map(|part| r.to_int(part) as u32))
}

/// `(a a + b b) k` (`times` = 1) or `(a a + b b) k k` (`times` = 2) as a whole number.
pub fn int_square_sum_scaled_to_int(a: i32, b: i32, k: impl Flt, times: u32) -> u32 {
    let k = k.value();
    let (a, b) = (int(a), int(b));
    let mut x = (a * a + b * b) * ld(k);
    if times == 2 {
        x *= ld(k);
    }
    to_int(x) as u32
}

/// The camera's roll eased towards a target (`race::view_matrices`), rounded once at the end:
/// `((((i / a / a / b) v c) - m) h) + m`.
pub fn eased_roll(
    i: i32,
    a: impl Flt,
    b: impl Flt,
    v: impl Flt,
    c: impl Flt,
    m: impl Flt,
    h: impl Flt,
) -> f32 {
    let a = a.value();
    let b = b.value();
    let v = v.value();
    let c = c.value();
    let m = m.value();
    let h = h.value();
    let mut x = int(i);
    x /= ld(a);
    x /= ld(a);
    x /= ld(b);
    x *= ld(v);
    x *= ld(c);
    x -= ld(m);
    x *= ld(h);
    x += ld(m);
    out(x)
}

/// The sun's direction in `race::draw_sky`: with `p = (k - a) s` and `q = b s`, cos p cos q,
/// sin q and sin p cos q, each times `m` and stored as a float.
pub fn sun_direction(a: i32, k: impl Flt, s: impl Flt, b: i32, m: impl Flt) -> [u32; 3] {
    let k = k.value();
    let s = s.value();
    let m = m.value();
    let p = (-int(a) + ld(k)) * ld(s);
    let q = int(b) * ld(s);
    let (sin_p, cos_p) = p.sin_cos();
    let (sin_q, cos_q) = q.sin_cos();
    let cos_cos = cos_p * cos_q;
    let sin_cos = sin_p * cos_q;
    [st(cos_cos * ld(m)), st(sin_q * ld(m)), st(sin_cos * ld(m))]
}

/// A float as the x87's 80 bits.
pub fn to_tbyte(a: impl Flt) -> [u8; 10] {
    let a = a.value();
    f80(ld(a))
}

/// 80 bits rounded to a whole number as the x87 is set to round.
pub fn tbyte_to_int(a: [u8; 10]) -> u32 {
    to_int(from_f80(a)) as u32
}

/// A point of whole numbers onto the screen as in `race::draw_sky`: `point` = x, y, z and
/// the flat depth; `centre` = the centre's y and x. Flat: z dropped (into `scratch`, as a
/// float) and the scale over the flat depth + 1; else the scale over z. Gives x and y as
/// floats (the first) and rounded to whole numbers (the second), each `x r + cx` and
/// `cy - y r`, r the ratio.
pub fn sun_on_screen(
    point: [i32; 4],
    scale: impl Flt,
    centre: [impl Flt; 2],
    flat: bool,
    scratch: &impl crate::session::Cell32,
) -> ([f32; 2], [i32; 2]) {
    let scale = scale.value();
    let [cy, cx] = centre.map(|c| c.value());
    let (x, y, z) = (int(point[0]), int(point[1]), int(point[2]));
    let ratio = if flat {
        scratch.set_bits(st(z));
        ld(scale) / (int(point[3]) + X(1.0))
    } else {
        ld(scale) / z
    };
    let sx = x * ratio + ld(cx);
    let sy = -(y * ratio) + ld(cy);
    ([out(sx), out(sy)], [to_int(sx), to_int(sy)])
}

/// `k` less half the distance from the centre, rounded, as a whole number: `-(sqrt((x -
/// cx)^2 + (y - cy)^2) h - k)`.
pub fn sun_size(
    x: impl Flt,
    cx: impl Flt,
    y: impl Flt,
    cy: impl Flt,
    h: impl Flt,
    k: impl Flt,
) -> u32 {
    let x = x.value();
    let cx = cx.value();
    let y = y.value();
    let cy = cy.value();
    let h = h.value();
    let k = k.value();
    let dx = ld(x) - ld(cx);
    let dx2 = dx * dx;
    let dy = ld(y) - ld(cy);
    let dy2 = dy * dy;
    let d = (dx2 + dy2).sqrt() * ld(h) - ld(k);
    to_int(-d) as u32
}
