//! More x87 helpers (see `math/x87.rs`), kept apart so that work on the race's loading does not
//! edit the shared file.

#[allow(unused_imports)] // used as helpers are added
use crate::math::x87::x87;

/// `fild a; fimul b; fdiv c; fistp`: a whole number times a whole number over a float,
/// rounded once at the end as the x87 is set to round.
pub fn int_times_int_over_to_int(a: i32, b: i32, c: u32) -> i32 {
    let (v, mut out) = ([a as u32, b as u32, c], 0i32);
    x87!(
        "fild dword ptr [{v}]",
        "fimul dword ptr [{v} + 4]",
        "fdiv dword ptr [{v} + 8]",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out
    );
    out
}

/// `fld a; fsin; fmul k; fadd one; fstp`: 1 + k sin(a), rounded once.
pub fn sine_times_plus(a: u32, k: u32, one: u32) -> u32 {
    let (v, mut out) = ([a, k, one], 0u32);
    x87!(
        "fld dword ptr [{v}]",
        "fsin",
        "fmul dword ptr [{v} + 4]",
        "fadd dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out
    );
    out
}

/// `fild a; fdiv b; fdiv c; fadd st, st; fstp`: twice a whole number over two floats.
pub fn int_over_two_doubled(a: i32, b: u32, c: u32) -> u32 {
    let (v, mut out) = ([a as u32, b, c], 0u32);
    x87!(
        "fild dword ptr [{v}]",
        "fdiv dword ptr [{v} + 4]",
        "fdiv dword ptr [{v} + 8]",
        "fadd st, st",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out
    );
    out
}

/// `fild a; fmul b; fmul c; fstp`: a whole number times two floats.
pub fn int_times_two(a: i32, b: u32, c: u32) -> u32 {
    let (v, mut out) = ([a as u32, b, c], 0u32);
    x87!(
        "fild dword ptr [{v}]",
        "fmul dword ptr [{v} + 4]",
        "fmul dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out
    );
    out
}

/// `fild a; fldpi; fmulp st(1), st; fmul b; fdiv c; fstp`: a whole number times pi times
/// b over c.
pub fn int_pi_times_over(a: i32, b: u32, c: u32) -> u32 {
    let (v, mut out) = ([a as u32, b, c], 0u32);
    x87!(
        "fild dword ptr [{v}]",
        "fldpi",
        "fmulp st(1), st",
        "fmul dword ptr [{v} + 4]",
        "fdiv dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out
    );
    out
}

/// `fld a; fabs; fistp`: the size of a float as a whole number.
pub fn abs_to_int(a: u32) -> i32 {
    let (v, mut out) = ([a], 0i32);
    x87!(
        "fld dword ptr [{v}]",
        "fabs",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out
    );
    out
}

/// `fld a; fadd b; fistp`: the sum of two floats as a whole number.
pub fn add_to_int(a: u32, b: u32) -> i32 {
    let (v, mut out) = ([a, b], 0i32);
    x87!(
        "fld dword ptr [{v}]",
        "fadd dword ptr [{v} + 4]",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out
    );
    out
}

/// `fld a; fmul st, st; fld b; fmul st, st; faddp; fsqrt; fstp`: the length of two floats
/// at right angles.
pub fn float_hypot(a: u32, b: u32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!(
        "fld dword ptr [{v}]",
        "fmul st, st",
        "fld dword ptr [{v} + 4]",
        "fmul st, st",
        "faddp st(1), st",
        "fsqrt",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out
    );
    out
}

/// `fld a; fcomp b; fnstsw ax`, C0: a below b, or one is no number (`shr ah, 1; jb`).
pub fn below_or_unordered(a: u32, b: u32) -> bool {
    let v = [a, b];
    let status: u16;
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{v}]",
            "fcomp dword ptr [{v} + 4]",
            "fnstsw ax",
            v = in(reg) v.as_ptr(),
            out("ax") status,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
    status & 0x0100 != 0
}

/// `fld a; fmul h; fabs; fsub k; fcomp b; fnstsw ax`, C0: |a h| - k below b (kept in the
/// x87's precision), or one is no number.
pub fn scaled_abs_less_below(a: u32, h: u32, k: u32, b: u32) -> bool {
    let v = [a, h, k, b];
    let status: u16;
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{v}]",
            "fmul dword ptr [{v} + 4]",
            "fabs",
            "fsub dword ptr [{v} + 8]",
            "fcomp dword ptr [{v} + 12]",
            "fnstsw ax",
            v = in(reg) v.as_ptr(),
            out("ax") status,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
    status & 0x0100 != 0
}

/// `scene::project_point::rotate` with its third store a `fistp`: the point (three whole
/// numbers) through `matrix`, x and y stored as floats at `out[0]`, `out[1]`, z rounded
/// into `out[2]` from the x87's precision (`sub_4573FA`).
pub unsafe fn rotate_rounding_z(point: *const i32, matrix: *const u32, out: *mut u32) {
    x87!(
        "fild dword ptr [{p} + 8]", "fild dword ptr [{p} + 4]", "fild dword ptr [{p}]",
        "fld dword ptr [{m}]", "fmul st(0), st(1)",
        "fld dword ptr [{m} + 0x10]", "fmul st(0), st(3)",
        "fld dword ptr [{m} + 0x20]", "fmul st(0), st(5)",
        "faddp st(1), st(0)", "faddp st(1), st(0)",
        "fadd dword ptr [{m} + 0x30]", "fxch st(3)",
        "fld dword ptr [{m} + 4]", "fmul st(0), st(2)",
        "fld dword ptr [{m} + 0x14]", "fmul st(0), st(4)",
        "fld dword ptr [{m} + 0x24]", "fmul st(0), st(3)",
        "faddp st(1), st(0)", "faddp st(1), st(0)",
        "fadd dword ptr [{m} + 0x34]", "fxch st(3)",
        "fld dword ptr [{m} + 8]", "fmulp st(3), st(0)",
        "fld dword ptr [{m} + 0x18]", "fmulp st(1), st(0)",
        "fld dword ptr [{m} + 0x28]", "fmulp st(2), st(0)",
        "faddp st(1), st(0)", "faddp st(1), st(0)",
        "fadd dword ptr [{m} + 0x38]", "fxch st(2)",
        "fstp dword ptr [{o}]", "fstp dword ptr [{o} + 4]",
        "fistp dword ptr [{o} + 8]";
        p = point, m = matrix, o = out
    );
}

/// `fild a; fmul st, st; fild b; fmul st, st; faddp; fstp`: the sum of two whole numbers'
/// squares, as a float.
pub fn int_square_sum(a: i32, b: i32) -> u32 {
    let (v, mut out) = ([a as u32, b as u32], 0u32);
    x87!(
        "fild dword ptr [{v}]",
        "fmul st, st",
        "fild dword ptr [{v} + 4]",
        "fmul st, st",
        "faddp st(1), st",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out
    );
    out
}

/// `fld a; fmul st, st; fld b; fmul st, st; faddp; fstp`: the sum of two floats' squares.
pub fn float_square_sum(a: u32, b: u32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!(
        "fld dword ptr [{v}]",
        "fmul st, st",
        "fld dword ptr [{v} + 4]",
        "fmul st, st",
        "faddp st(1), st",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out
    );
    out
}

/// `fld a; fmul b; fmul c; fstp`: three floats' product, rounded once.
pub fn product3(a: u32, b: u32, c: u32) -> u32 {
    let (v, mut out) = ([a, b, c], 0u32);
    x87!(
        "fld dword ptr [{v}]",
        "fmul dword ptr [{v} + 4]",
        "fmul dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out
    );
    out
}

/// `fild a; fdiv b; fstp`: a whole number over a float.
pub fn int_over_float(a: i32, b: u32) -> u32 {
    let (v, mut out) = ([a as u32, b], 0u32);
    x87!(
        "fild dword ptr [{v}]",
        "fdiv dword ptr [{v} + 4]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out
    );
    out
}

/// `fild a; fadd b; fmul c; fstp`: (a whole number plus a float) times a float.
pub fn int_plus_times(a: i32, b: u32, c: u32) -> u32 {
    let (v, mut out) = ([a as u32, b, c], 0u32);
    x87!(
        "fild dword ptr [{v}]",
        "fadd dword ptr [{v} + 4]",
        "fmul dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out
    );
    out
}

/// `fld a; fabs; fcomp k; fnstsw ax`, C0: |a| below k, or one is no number.
pub fn abs_below_or_unordered(a: u32, k: u32) -> bool {
    let v = [a, k];
    let status: u16;
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{v}]",
            "fabs",
            "fcomp dword ptr [{v} + 4]",
            "fnstsw ax",
            v = in(reg) v.as_ptr(),
            out("ax") status,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
    status & 0x0100 != 0
}

/// `scene::project_point::rotate` with all three stores `fistp`: the point (three whole
/// numbers) through `matrix`, each part rounded from the x87's precision into `out`.
pub unsafe fn rotate_rounding(point: *const i32, matrix: *const u32, out: *mut u32) {
    x87!(
        "fild dword ptr [{p} + 8]", "fild dword ptr [{p} + 4]", "fild dword ptr [{p}]",
        "fld dword ptr [{m}]", "fmul st(0), st(1)",
        "fld dword ptr [{m} + 0x10]", "fmul st(0), st(3)",
        "fld dword ptr [{m} + 0x20]", "fmul st(0), st(5)",
        "faddp st(1), st(0)", "faddp st(1), st(0)",
        "fadd dword ptr [{m} + 0x30]", "fxch st(3)",
        "fld dword ptr [{m} + 4]", "fmul st(0), st(2)",
        "fld dword ptr [{m} + 0x14]", "fmul st(0), st(4)",
        "fld dword ptr [{m} + 0x24]", "fmul st(0), st(3)",
        "faddp st(1), st(0)", "faddp st(1), st(0)",
        "fadd dword ptr [{m} + 0x34]", "fxch st(3)",
        "fld dword ptr [{m} + 8]", "fmulp st(3), st(0)",
        "fld dword ptr [{m} + 0x18]", "fmulp st(1), st(0)",
        "fld dword ptr [{m} + 0x28]", "fmulp st(2), st(0)",
        "faddp st(1), st(0)", "faddp st(1), st(0)",
        "fadd dword ptr [{m} + 0x38]", "fxch st(2)",
        "fistp dword ptr [{o}]", "fistp dword ptr [{o} + 4]",
        "fistp dword ptr [{o} + 8]";
        p = point, m = matrix, o = out
    );
}

/// `fild a; fmul st, st; fild b; fmul st, st; faddp; fmul k` (`times` times) `; fistp`.
pub fn int_square_sum_scaled_to_int(a: i32, b: i32, k: u32, times: u32) -> u32 {
    let (v, mut out) = ([a as u32, b as u32, k], 0u32);
    if times == 2 {
        x87!(
            "fild dword ptr [{v}]", "fmul st, st", "fild dword ptr [{v} + 4]", "fmul st, st",
            "faddp st(1), st", "fmul dword ptr [{v} + 8]", "fmul dword ptr [{v} + 8]",
            "fistp dword ptr [{o}]";
            v = v.as_ptr(), o = &raw mut out
        );
    } else {
        x87!(
            "fild dword ptr [{v}]", "fmul st, st", "fild dword ptr [{v} + 4]", "fmul st, st",
            "faddp st(1), st", "fmul dword ptr [{v} + 8]",
            "fistp dword ptr [{o}]";
            v = v.as_ptr(), o = &raw mut out
        );
    }
    out
}

/// `fild i; fdiv a; fdiv a; fdiv b; fmul v; fmul c; fsub m; fmul h; fadd m; fstp`: the
/// camera's roll eased towards a target (`sub_4386A4`), rounded once.
pub fn eased_roll(i: i32, a: u32, b: u32, v: u32, c: u32, m: u32, h: u32) -> u32 {
    let (x, mut out) = ([i as u32, a, b, v, c, m, h], 0u32);
    x87!(
        "fild dword ptr [{x}]",
        "fdiv dword ptr [{x} + 4]",
        "fdiv dword ptr [{x} + 4]",
        "fdiv dword ptr [{x} + 8]",
        "fmul dword ptr [{x} + 12]",
        "fmul dword ptr [{x} + 16]",
        "fsub dword ptr [{x} + 20]",
        "fmul dword ptr [{x} + 24]",
        "fadd dword ptr [{x} + 20]",
        "fstp dword ptr [{o}]";
        x = x.as_ptr(), o = &raw mut out
    );
    out
}

/// `fld a; fmul b; fmul c; fmul d; fistp`: four floats' product as a whole number.
pub fn product4_to_int(a: u32, b: u32, c: u32, d: u32) -> u32 {
    let (x, mut out) = ([a, b, c, d], 0u32);
    x87!(
        "fld dword ptr [{x}]",
        "fmul dword ptr [{x} + 4]",
        "fmul dword ptr [{x} + 8]",
        "fmul dword ptr [{x} + 12]",
        "fistp dword ptr [{o}]";
        x = x.as_ptr(), o = &raw mut out
    );
    out
}

/// The sun's direction in `sub_455540`: `fild a; fchs; fadd k; fmul s; fsincos; fild b;
/// fmul s; fsincos; fmul st(2), st; fmulp st(3), st; fxch st(1)`, then each of the three
/// times `m` and stored: cos a cos b, sin b, sin a cos b (each x `m`).
pub fn sun_direction(a: i32, k: u32, s: u32, b: i32, m: u32) -> [u32; 3] {
    let (v, mut out) = ([a as u32, k, s, b as u32, m], [0u32; 3]);
    x87!(
        "fild dword ptr [{v}]", "fchs", "fadd dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fsincos",
        "fild dword ptr [{v} + 12]", "fmul dword ptr [{v} + 8]", "fsincos",
        "fmul st(2), st(0)", "fmulp st(3), st(0)", "fxch st(1)",
        "fmul dword ptr [{v} + 16]", "fstp dword ptr [{o}]",
        "fmul dword ptr [{v} + 16]", "fstp dword ptr [{o} + 4]",
        "fmul dword ptr [{v} + 16]", "fstp dword ptr [{o} + 8]";
        v = v.as_ptr(), o = out.as_mut_ptr()
    );
    out
}

/// A float as the x87's 80 bits (`fld dword; fstp tbyte`).
pub fn to_tbyte(a: u32) -> [u8; 10] {
    let (v, mut out) = ([a], [0u8; 10]);
    x87!("fld dword ptr [{v}]", "fstp tbyte ptr [{o}]"; v = v.as_ptr(), o = out.as_mut_ptr());
    out
}

/// 80 bits rounded to a whole number as the x87 is set to round (`fld tbyte; fistp`).
pub fn tbyte_to_int(a: [u8; 10]) -> u32 {
    let mut out = 0u32;
    x87!("fld tbyte ptr [{v}]", "fistp dword ptr [{o}]"; v = a.as_ptr(), o = &raw mut out);
    out
}

/// A point of whole numbers onto the screen as in `sub_455540` (the original's
/// instructions; `.byte 0xDE, 0xF1` its `fdivrp st(1), st`): `v` = x, y, z, the flat
/// depth, the scale, the centre's y and x. Flat: z dropped (into `scratch`) for the flat
/// depth + 1. Gives x as a float and rounded, y as a float and rounded.
pub unsafe fn sun_on_screen(v: &[u32; 7], flat: bool, scratch: *mut u32) -> [u32; 4] {
    let mut out = [0u32; 4];
    if flat {
        x87!(
            "fild dword ptr [{v}]", "fild dword ptr [{v} + 4]", "fild dword ptr [{v} + 8]",
            "fstp dword ptr [{w}]", "fild dword ptr [{v} + 12]", "fld1", "faddp st(1), st(0)",
            "fld dword ptr [{v} + 16]", ".byte 0xDE, 0xF1",
            "fmul st(1), st(0)", "fmulp st(2), st(0)", "fchs",
            "fadd dword ptr [{v} + 20]", "fxch st(1)", "fadd dword ptr [{v} + 24]",
            "fst dword ptr [{o}]", "fistp dword ptr [{o} + 4]",
            "fst dword ptr [{o} + 8]", "fistp dword ptr [{o} + 12]";
            v = v.as_ptr(), w = scratch, o = out.as_mut_ptr()
        );
    } else {
        x87!(
            "fild dword ptr [{v}]", "fild dword ptr [{v} + 4]", "fild dword ptr [{v} + 8]",
            "fld dword ptr [{v} + 16]", ".byte 0xDE, 0xF1",
            "fmul st(1), st(0)", "fmulp st(2), st(0)", "fchs",
            "fadd dword ptr [{v} + 20]", "fxch st(1)", "fadd dword ptr [{v} + 24]",
            "fst dword ptr [{o}]", "fistp dword ptr [{o} + 4]",
            "fst dword ptr [{o} + 8]", "fistp dword ptr [{o} + 12]";
            v = v.as_ptr(), o = out.as_mut_ptr()
        );
    }
    out
}

/// `fld x; fsub cx; fmul st, st; fld y; fsub cy; fmul st, st; faddp; fsqrt; fmul h; fsub k;
/// fchs; fistp`: k less half the distance from the centre, rounded.
pub fn sun_size(x: u32, cx: u32, y: u32, cy: u32, h: u32, k: u32) -> u32 {
    let (v, mut out) = ([x, cx, y, cy, h, k], 0u32);
    x87!(
        "fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fmul st(0), st(0)",
        "fld dword ptr [{v} + 8]", "fsub dword ptr [{v} + 12]", "fmul st(0), st(0)",
        "faddp st(1), st(0)", "fsqrt", "fmul dword ptr [{v} + 16]",
        "fsub dword ptr [{v} + 20]", "fchs", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out
    );
    out
}

/// `fld a; fmul k; fcomp b; fnstsw ax`, C0: a times k (in the x87's precision) below b, or
/// one is no number.
pub fn scaled_below_or_unordered(a: u32, k: u32, b: u32) -> bool {
    let v = [a, k, b];
    let status: u16;
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{v}]",
            "fmul dword ptr [{v} + 4]",
            "fcomp dword ptr [{v} + 8]",
            "fnstsw ax",
            v = in(reg) v.as_ptr(),
            out("ax") status,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
    status & 0x0100 != 0
}
