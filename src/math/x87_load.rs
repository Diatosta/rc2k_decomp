//! More x87 helpers (see `math/x87.rs`), kept apart so that work on the race's loading does not
//! edit the shared file.
//!
//! Plain Rust since phase 6 (2026-10-24): what the x87 holds on its stack is an `X` (an
//! `f64` that rounds as the control word says, `x87_float.rs`), what it stores as a dword
//! float `st`, `fild` is `int`, `fistp` / `fist` `to_int`. The order of the operations is
//! the original's. The assembly versions are the reference the tests compare with
//! (`x87_reference_c.rs`).

use crate::math::x87_float::{X, compare, f80, from_f80, int, ld, st, to_int};

/// `(a * b) / c`, rounded once at the end as the x87 is set to round
/// (`fild a; fimul b; fdiv c; fistp`): a whole number times a whole number over a float.
pub fn int_times_int_over_to_int(a: i32, b: i32, c: u32) -> i32 {
    to_int(int(a) * int(b) / ld(c))
}

/// `|a|` as a whole number (`fld a; fabs; fistp`): the size of a float.
pub fn abs_to_int(a: u32) -> i32 {
    to_int(ld(a).abs())
}

/// `a + b` as a whole number (`fld a; fadd b; fistp`): the sum of two floats.
pub fn add_to_int(a: u32, b: u32) -> i32 {
    to_int(ld(a) + ld(b))
}

/// `sqrt(a a + b b)` as a float (`fld a; fmul st, st; fld b; fmul st, st; faddp; fsqrt;
/// fstp`): the length of two floats at right angles.
pub fn float_hypot(a: u32, b: u32) -> u32 {
    let a2 = ld(a) * ld(a);
    let b2 = ld(b) * ld(b);
    st((a2 + b2).sqrt())
}

/// C0 of `fld a; fcomp b; fnstsw ax` (`shr ah, 1; jb`): a below b, or one is no number.
pub fn below_or_unordered(a: u32, b: u32) -> bool {
    compare(ld(a), ld(b)) & 0x0100 != 0
}

/// C0 of `fld a; fmul h; fabs; fsub k; fcomp b`: |a h| - k below b (kept at the x87's
/// precision), or one is no number.
pub fn scaled_abs_less_below(a: u32, h: u32, k: u32, b: u32) -> bool {
    let x = (ld(a) * ld(h)).abs() - ld(k);
    compare(x, ld(b)) & 0x0100 != 0
}

/// The `T` at `p`, checked to lie in one region (the program ends otherwise).
fn get<T: crate::pointers::Plain, P>(p: *const P) -> T {
    crate::pointers::read_or_end(crate::pointers::game_address(p), "a math helper's operand")
}

/// `value` written at `p`, checked as `get`.
fn put<T: crate::pointers::Plain, P>(p: *mut P, value: T) {
    crate::pointers::write_or_end(
        crate::pointers::game_address(p.cast_const()),
        value,
        "a math helper's operand",
    );
}

/// The point (three whole numbers) through `matrix` (16 floats; the 12 used are the
/// columns' at 0, 4, 8 / 0x10, 0x14, 0x18 / 0x20, 0x24, 0x28 and the translation at 0x30,
/// 0x34, 0x38), as the original adds them: m0 x + (m4 y + m8 z) and so on, then the
/// translation. Left at double precision.
fn rotated(point: *const i32, matrix: *const u32) -> [X; 3] {
    let [x, y, z]: [i32; 3] = get(point);
    let (x, y, z) = (int(x), int(y), int(z));
    let matrix: [u32; 15] = get(matrix);
    let m = |i: usize| ld(matrix[i]);
    let s0 = m(0) * x + (m(4) * y + m(8) * z) + m(12);
    let s1 = m(1) * x + (m(5) * y + m(9) * z) + m(13);
    let s2 = x * m(2) + (y * m(6) + z * m(10)) + m(14);
    [s0, s1, s2]
}

/// `scene::project_point::rotate` with its third store a `fistp`: the point (three whole
/// numbers) through `matrix`, x and y stored as floats at `out[0]`, `out[1]`, z rounded
/// into `out[2]` from the x87's precision (`sub_4573FA`).
pub fn rotate_rounding_z(point: *const i32, matrix: *const u32, out: *mut u32) {
    let s = rotated(point, matrix);
    put(out, [st(s[0]), st(s[1]), to_int(s[2]) as u32]);
}

/// `a a + b b` of two whole numbers, as a float (`fild a; fmul st, st; fild b; fmul st,
/// st; faddp; fstp`).
pub fn int_square_sum(a: i32, b: i32) -> u32 {
    let (a, b) = (int(a), int(b));
    st(a * a + b * b)
}

/// `a a + b b` of two floats (`fld a; fmul st, st; fld b; fmul st, st; faddp; fstp`).
pub fn float_square_sum(a: u32, b: u32) -> u32 {
    let a2 = ld(a) * ld(a);
    let b2 = ld(b) * ld(b);
    st(a2 + b2)
}

/// `a b c`, rounded once (`fld a; fmul b; fmul c; fstp`).
pub fn product3(a: u32, b: u32, c: u32) -> u32 {
    st(ld(a) * ld(b) * ld(c))
}

/// A whole number over a float (`fild a; fdiv b; fstp`).
pub fn int_over_float(a: i32, b: u32) -> u32 {
    st(int(a) / ld(b))
}

/// `(a + b) c`, a whole number plus a float, times a float (`fild a; fadd b; fmul c;
/// fstp`).
pub fn int_plus_times(a: i32, b: u32, c: u32) -> u32 {
    st((int(a) + ld(b)) * ld(c))
}

/// C0 of `fld a; fabs; fcomp k`: |a| below k, or one is no number.
pub fn abs_below_or_unordered(a: u32, k: u32) -> bool {
    compare(ld(a).abs(), ld(k)) & 0x0100 != 0
}

/// `scene::project_point::rotate` with all three stores `fistp`: the point (three whole
/// numbers) through `matrix`, each part rounded from the x87's precision into `out`.
pub fn rotate_rounding(point: *const i32, matrix: *const u32, out: *mut u32) {
    let s = rotated(point, matrix);
    put(out, s.map(|part| to_int(part) as u32));
}

/// `(a a + b b) k` (`times` = 1) or `(a a + b b) k k` (`times` = 2) as a whole number
/// (`fild a; fmul st, st; fild b; fmul st, st; faddp; fmul k; [fmul k;] fistp`).
pub fn int_square_sum_scaled_to_int(a: i32, b: i32, k: u32, times: u32) -> u32 {
    let (a, b) = (int(a), int(b));
    let mut x = (a * a + b * b) * ld(k);
    if times == 2 {
        x *= ld(k);
    }
    to_int(x) as u32
}

/// The camera's roll eased towards a target (`sub_4386A4`), rounded once at the end:
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

/// `a b c d` as a whole number (`fld a; fmul b; fmul c; fmul d; fistp`).
pub fn product4_to_int(a: u32, b: u32, c: u32, d: u32) -> u32 {
    to_int(ld(a) * ld(b) * ld(c) * ld(d)) as u32
}

/// The sun's direction in `sub_455540`: with `p = (k - a) s` and `q = b s`, cos p cos q,
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

/// A float as the x87's 80 bits (`fld dword; fstp tbyte`).
pub fn to_tbyte(a: u32) -> [u8; 10] {
    f80(ld(a))
}

/// 80 bits rounded to a whole number as the x87 is set to round (`fld tbyte; fistp`).
pub fn tbyte_to_int(a: [u8; 10]) -> u32 {
    to_int(from_f80(a)) as u32
}

/// A point of whole numbers onto the screen as in `sub_455540`: `v` = x, y, z, the flat
/// depth, the scale, the centre's y and x. Flat: z dropped (into `scratch`, as a float)
/// and the scale over the flat depth + 1; else the scale over z. Gives x as a float and
/// rounded, y as a float and rounded (each `x r + cx` and `cy - y r`, r the ratio).
pub fn sun_on_screen(v: &[u32; 7], flat: bool, scratch: &impl crate::session::Cell32) -> [u32; 4] {
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
