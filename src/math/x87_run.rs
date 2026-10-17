//! More x87 helpers (see `math/x87.rs`), kept apart so that work on the race's run does not
//! edit the shared file.

use crate::math::x87::x87;

/// `fld v[0]; fmul st, st; fld v[1]; fmul st, st; fld v[2]; fmul st, st; faddp; faddp;
/// fsqrt`: the length of three floats as an 80-bit value (not rounded to a float).
pub unsafe fn length_80(v: *const u32) -> [u8; 10] {
    let mut out = [0u8; 10];
    x87!("fld dword ptr [{v}]", ".byte 0xD8, 0xC8",
        "fld dword ptr [{v} + 4]", ".byte 0xD8, 0xC8",
        "fld dword ptr [{v} + 8]", ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fsqrt",
        "fstp tbyte ptr [{o}]";
        v = v, o = out.as_mut_ptr());
    out
}

/// With the 80-bit `s` in st(0): `fld v[k]; fmul st, st(1); fstp v[k]` for k = 0, 1, 2.
pub unsafe fn scale_by_80(v: *mut u32, s: [u8; 10]) {
    x87!("fld tbyte ptr [{s}]",
        "fld dword ptr [{v}]", ".byte 0xD8, 0xC9", "fstp dword ptr [{v}]",
        "fld dword ptr [{v} + 4]", ".byte 0xD8, 0xC9", "fstp dword ptr [{v} + 4]",
        "fld dword ptr [{v} + 8]", ".byte 0xD8, 0xC9", "fstp dword ptr [{v} + 8]",
        "fstp st(0)";
        v = v, s = s.as_ptr());
}

/// `fld a; fadd b; fmul h; fdiv d; fstp`.
pub fn sum_times_over(a: u32, b: u32, h: u32, d: u32) -> u32 {
    let (v, mut out) = ([a, b, h, d], 0u32);
    x87!("fld dword ptr [{v}]", "fadd dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fdiv dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// Runs `f` with the x87's control word set to `word`, then puts the old one back (for
/// self-tests that try several precisions).
pub fn with_control_word(word: u16, f: &mut dyn FnMut()) {
    let mut old = 0u16;
    unsafe {
        core::arch::asm!("fnstcw word ptr [{o}]", "fldcw word ptr [{w}]",
            o = in(reg) &raw mut old, w = in(reg) &raw const word, options(nostack));
    }
    f();
    unsafe {
        core::arch::asm!("fldcw word ptr [{w}]", w = in(reg) &raw const old, options(nostack));
    }
}

/// Control words for those tests: single, double and extended precision, all exceptions
/// masked, rounding to nearest.
pub const TEST_CONTROL_WORDS: [u16; 3] = [0x007F, 0x027F, 0x037F];

/// `fild a; fild c; fsubp; fmul f; fild b; fild c; fsubp; fmul g; fild c; faddp; faddp;
/// fistp`: c + (a - c) f + (b - c) g as a whole number (the sums in that order).
pub fn plane_height(a: i32, b: i32, c: i32, f: u32, g: u32) -> i32 {
    let (i, v, mut out) = ([a, b, c], [f, g], 0i32);
    x87!("fild dword ptr [{i}]", "fild dword ptr [{i} + 8]", ".byte 0xDE, 0xE9",
        "fmul dword ptr [{v}]",
        "fild dword ptr [{i} + 4]", "fild dword ptr [{i} + 8]", ".byte 0xDE, 0xE9",
        "fmul dword ptr [{v} + 4]",
        "fild dword ptr [{i} + 8]", ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1",
        "fistp dword ptr [{o}]";
        i = i.as_ptr(), v = v.as_ptr(), o = &raw mut out);
    out
}

/// `physics::face_shares`' floating point part, the original's instructions in its order.
/// `i`: the differences d738, d73C, d740, d750, d754, d758 (whole numbers) and the scale
/// `flt_5C5D98` (bits) at [6]. `o` gets f764, f75C, f760, f810, i800, f814, i804, and with
/// `sse` f768 (`rcpss` of f764) at [7].
pub unsafe fn face_shares(i: &[u32; 7], o: &mut [u32; 8], sse: bool) {
    let (i, o) = (i.as_ptr(), o.as_mut_ptr());
    unsafe {
        if !sse {
            core::arch::asm!(
                "fild dword ptr [{i} + 4]", "fild dword ptr [{i} + 20]", ".byte 0xDE, 0xC9",
                "fild dword ptr [{i} + 8]", "fild dword ptr [{i} + 16]", ".byte 0xDE, 0xC9",
                ".byte 0xDE, 0xE9", "fstp dword ptr [{o}]",
                "fild dword ptr [{i}]", "fild dword ptr [{i} + 20]", ".byte 0xDE, 0xC9",
                "fild dword ptr [{i} + 8]", "fild dword ptr [{i} + 12]", ".byte 0xDE, 0xC9",
                ".byte 0xDE, 0xE9", "fstp dword ptr [{o} + 4]",
                "fild dword ptr [{i} + 4]", "fild dword ptr [{i} + 12]", ".byte 0xDE, 0xC9",
                "fild dword ptr [{i}]", "fild dword ptr [{i} + 16]", ".byte 0xDE, 0xC9",
                ".byte 0xDE, 0xE9", "fstp dword ptr [{o} + 8]",
                "fld dword ptr [{o} + 4]", "fdiv dword ptr [{o}]", "fst dword ptr [{o} + 12]",
                "fmul dword ptr [{i} + 24]", "fistp dword ptr [{o} + 16]",
                "fld dword ptr [{o} + 8]", "fdiv dword ptr [{o}]", "fst dword ptr [{o} + 20]",
                "fmul dword ptr [{i} + 24]", "fistp dword ptr [{o} + 24]",
                i = in(reg) i, o = in(reg) o,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        } else {
            core::arch::asm!(
                "fild dword ptr [{i} + 4]", "fild dword ptr [{i} + 20]", ".byte 0xDE, 0xC9",
                "fild dword ptr [{i} + 8]", "fild dword ptr [{i} + 16]", ".byte 0xDE, 0xC9",
                ".byte 0xDE, 0xE9", "fstp dword ptr [{o}]",
                "rcpss xmm0, dword ptr [{o}]", "movss dword ptr [{o} + 28], xmm0",
                "fild dword ptr [{i}]", "fild dword ptr [{i} + 20]", ".byte 0xDE, 0xC9",
                "fild dword ptr [{i} + 8]", "fild dword ptr [{i} + 12]", ".byte 0xDE, 0xC9",
                ".byte 0xDE, 0xE9", "fst dword ptr [{o} + 4]", "fmul dword ptr [{o} + 28]",
                "fst dword ptr [{o} + 12]", "fmul dword ptr [{i} + 24]",
                "fistp dword ptr [{o} + 16]",
                "fild dword ptr [{i} + 4]", "fild dword ptr [{i} + 12]", ".byte 0xDE, 0xC9",
                "fild dword ptr [{i}]", "fild dword ptr [{i} + 16]", ".byte 0xDE, 0xC9",
                ".byte 0xDE, 0xE9", "fst dword ptr [{o} + 8]", "fmul dword ptr [{o} + 28]",
                "fst dword ptr [{o} + 20]", "fmul dword ptr [{i} + 24]",
                "fistp dword ptr [{o} + 24]",
                i = in(reg) i, o = in(reg) o,
                out("xmm0") _,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }
}

/// `fld a; fsub b; [fadd c;] fld d; fmul e; fsubp st(1), st`: (a - b [+ c]) - d e, left
/// unrounded (80 bits).
pub fn less_plus_less_product_80(a: u32, b: u32, c: Option<u32>, d: u32, e: u32) -> [u8; 10] {
    let mut out = [0u8; 10];
    let v = [a, b, c.unwrap_or(0), d, e];
    if c.is_some() {
        x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]",
            "fld dword ptr [{v} + 12]", "fmul dword ptr [{v} + 16]", ".byte 0xDE, 0xE9",
            "fstp tbyte ptr [{o}]";
            v = v.as_ptr(), o = out.as_mut_ptr());
    } else {
        x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]",
            "fld dword ptr [{v} + 12]", "fmul dword ptr [{v} + 16]", ".byte 0xDE, 0xE9",
            "fstp tbyte ptr [{o}]";
            v = v.as_ptr(), o = out.as_mut_ptr());
    }
    out
}

/// `fild i; fmul a; fmul b; fdiv c; fistp`.
pub fn int_times_times_over_to_int(i: i32, a: u32, b: u32, c: u32) -> i32 {
    let (v, mut out) = ([i as u32, a, b, c], 0i32);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fdiv dword ptr [{v} + 12]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul st, st; fmul k; fadd c; fmul g; fst; fchs; fstp`: (a a k + c) g and its
/// negative, each rounded to a float.
pub fn square_times_plus_times(a: u32, k: u32, c: u32, g: u32) -> (u32, u32) {
    let (v, mut out) = ([a, k, c, g], [0u32; 2]);
    x87!("fld dword ptr [{v}]", ".byte 0xD8, 0xC8", "fmul dword ptr [{v} + 4]",
        "fadd dword ptr [{v} + 8]", "fmul dword ptr [{v} + 12]", "fst dword ptr [{o}]",
        "fchs", "fstp dword ptr [{o} + 4]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    (out[0], out[1])
}

/// `fld1; fchs; fstp`: -1.0 as the x87 stores it.
pub fn minus_one() -> u32 {
    let mut out = 0u32;
    x87!("fld1", "fchs", "fstp dword ptr [{o}]"; o = &raw mut out);
    out
}

/// `fild i; fdiv d; fmul m; fstp`.
pub fn int_over_times(i: i32, d: u32, m: u32) -> u32 {
    let (v, mut out) = ([i as u32, d, m], 0u32);
    x87!("fild dword ptr [{v}]", "fdiv dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fcomp b; fnstsw`: the whole status word's C0 .. C3 (0x4700).
pub fn compare_full(a: u32, b: u32) -> u16 {
    let (v, mut status) = ([a, b], 0u16);
    x87!("fld dword ptr [{v}]", "fcomp dword ptr [{v} + 4]", "fnstsw word ptr [{s}]";
        v = v.as_ptr(), s = &raw mut status);
    status & 0x4700
}

/// `fld1`, then for each (a, b, c) `fld a; fmul b; fmul c; fsubp st(1), st`: 1 less the
/// products, unrounded (the 80 bits carried between the terms through memory, exactly).
pub fn one_less_products(terms: &[[u32; 3]]) -> [u8; 10] {
    let mut acc = [0u8; 10];
    x87!("fld1", "fstp tbyte ptr [{o}]"; o = acc.as_mut_ptr());
    for t in terms {
        x87!("fld tbyte ptr [{o}]", "fld dword ptr [{t}]", "fmul dword ptr [{t} + 4]",
            "fmul dword ptr [{t} + 8]", ".byte 0xDE, 0xE9", "fstp tbyte ptr [{o}]";
            o = acc.as_mut_ptr(), t = t.as_ptr());
    }
    acc
}

/// `fld e (80 bits); fmul k`, unrounded.
pub fn ext_times(e: [u8; 10], k: u32) -> [u8; 10] {
    let mut out = [0u8; 10];
    x87!("fld tbyte ptr [{e}]", "fmul dword ptr [{k}]", "fstp tbyte ptr [{o}]";
        e = e.as_ptr(), k = &raw const k, o = out.as_mut_ptr());
    out
}

/// A ramp through (t0, t1) .. (t2, t3) as the original reads it: below t0 (or no number)
/// t1; not below t2, t3; t2 - t0 equal to `zero` (or no number), t1; else
/// `fld x; fsub t0; fld t2; fsub t0; fdivp; fld t3; fsub t1; fmulp; fadd t1`. Rounded to
/// a float (the `fstp`).
pub fn ramp(x: u32, t: [u32; 4], zero: u32) -> u32 {
    if compare_full(x, t[0]) & 0x100 != 0 {
        return crate::math::x87::loaded(t[1]);
    }
    if compare_full(x, t[2]) & 0x100 == 0 {
        return crate::math::x87::loaded(t[3]);
    }
    let mut status = 0u16;
    x87!("fld dword ptr [{t} + 8]", "fsub dword ptr [{t}]", "fcomp dword ptr [{z}]",
        "fnstsw word ptr [{s}]";
        t = t.as_ptr(), z = &raw const zero, s = &raw mut status);
    if status & 0x4000 != 0 {
        return crate::math::x87::loaded(t[1]);
    }
    let mut out = 0u32;
    x87!("fld dword ptr [{x}]", "fsub dword ptr [{t}]", "fld dword ptr [{t} + 8]",
        "fsub dword ptr [{t}]", ".byte 0xDE, 0xF9", "fld dword ptr [{t} + 12]",
        "fsub dword ptr [{t} + 4]", ".byte 0xDE, 0xC9", "fadd dword ptr [{t} + 4]",
        "fstp dword ptr [{o}]";
        x = &raw const x, t = t.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul b; fmul c; fmul d; fstp`.
pub fn mul3(a: u32, b: u32, c: u32, d: u32) -> u32 {
    let (v, mut out) = ([a, b, c, d], 0u32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fmul dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul st, st; fstp`.
pub fn square(a: u32) -> u32 {
    let mut out = 0u32;
    x87!("fld dword ptr [{a}]", ".byte 0xD8, 0xC8", "fstp dword ptr [{o}]";
        a = &raw const a, o = &raw mut out);
    out
}

/// `fild i; fdiv d; fld hi; fsub lo; fmulp st(1), st; fadd lo; fstp`: lo + i / d (hi -
/// lo), the products in that order.
pub fn int_lerp(i: i32, d: u32, hi: u32, lo: u32) -> u32 {
    let (v, mut out) = ([i as u32, d, hi, lo], 0u32);
    x87!("fild dword ptr [{v}]", "fdiv dword ptr [{v} + 4]", "fld dword ptr [{v} + 8]",
        "fsub dword ptr [{v} + 12]", ".byte 0xDE, 0xC9", "fadd dword ptr [{v} + 12]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fsub b; fdiv c; fistp`.
pub fn sub_div_to_int(a: u32, b: u32, c: u32) -> i32 {
    let (v, mut out) = ([a, b, c], 0i32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fdiv dword ptr [{v} + 8]",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a` as an 80-bit value; with `negate`, `fchs` after.
pub fn int_80(a: i32, negate: bool) -> [u8; 10] {
    let mut out = [0u8; 10];
    if negate {
        x87!("fild dword ptr [{a}]", "fchs", "fstp tbyte ptr [{o}]";
            a = &raw const a, o = out.as_mut_ptr());
    } else {
        x87!("fild dword ptr [{a}]", "fstp tbyte ptr [{o}]";
            a = &raw const a, o = out.as_mut_ptr());
    }
    out
}

/// `fld e (80 bits); fistp`.
pub fn ext_to_int(e: [u8; 10]) -> i32 {
    let mut out = 0i32;
    x87!("fld tbyte ptr [{e}]", "fistp dword ptr [{o}]"; e = e.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fild b; fdivp st(1), st; fadd c; fstp`: a / b + c.
pub fn int_over_int_plus(a: i32, b: i32, c: u32) -> u32 {
    let (v, mut out) = ([a as u32, b as u32, c], 0u32);
    x87!("fild dword ptr [{v}]", "fild dword ptr [{v} + 4]", ".byte 0xDE, 0xF9",
        "fadd dword ptr [{v} + 8]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fsincos; fstp c; fstp s`: (sine, cosine) rounded to floats.
pub fn sine_cosine_of(a: u32) -> (u32, u32) {
    let mut out = [0u32; 2];
    x87!("fld dword ptr [{a}]", "fsincos", "fstp dword ptr [{o} + 4]", "fstp dword ptr [{o}]";
        a = &raw const a, o = out.as_mut_ptr());
    (out[0], out[1])
}

/// `fld1; fsub a; fstp`.
pub fn one_less(a: u32) -> u32 {
    let mut out = 0u32;
    x87!("fld1", "fsub dword ptr [{a}]", "fstp dword ptr [{o}]";
        a = &raw const a, o = &raw mut out);
    out
}

/// `fld x; fabs; fsqrt; fmul t; fadd c; fstp`.
pub fn abs_sqrt_times_plus(x: u32, t: u32, c: u32) -> u32 {
    let (v, mut out) = ([x, t, c], 0u32);
    x87!("fld dword ptr [{v}]", "fabs", "fsqrt", "fmul dword ptr [{v} + 4]",
        "fadd dword ptr [{v} + 8]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fadd b; fadd c; fstp`.
pub fn add3(a: u32, b: u32, c: u32) -> u32 {
    let (v, mut out) = ([a, b, c], 0u32);
    x87!("fld dword ptr [{v}]", "fadd dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul b; fmul c; fistp`.
pub fn mul2_to_int(a: u32, b: u32, c: u32) -> i32 {
    let (v, mut out) = ([a, b, c], 0i32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul b; fchs; fadd c; fstp`: c less a b.
pub fn neg_product_plus(a: u32, b: u32, c: u32) -> u32 {
    let (v, mut out) = ([a, b, c], 0u32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fchs", "fadd dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul b; fchs; fstp`.
pub fn neg_mul(a: u32, b: u32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fchs", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fabs; fcomp b; fnstsw`: C0 .. C3 (0x4700).
pub fn abs_compare(a: u32, b: u32) -> u16 {
    let (v, mut status) = ([a, b], 0u16);
    x87!("fld dword ptr [{v}]", "fabs", "fcomp dword ptr [{v} + 4]", "fnstsw word ptr [{s}]";
        v = v.as_ptr(), s = &raw mut status);
    status & 0x4700
}

/// `fld a; fabs; fstp`.
pub fn abs_of(a: u32) -> u32 {
    let mut out = 0u32;
    x87!("fld dword ptr [{a}]", "fabs", "fstp dword ptr [{o}]"; a = &raw const a, o = &raw mut out);
    out
}

/// `fld a; fmul st, st; fld b; fmul st, st; faddp; fsqrt; fistp`: the length of two
/// floats at right angles, as a whole number.
pub fn float_hypot_to_int(a: u32, b: u32) -> i32 {
    let (v, mut out) = ([a, b], 0i32);
    x87!("fld dword ptr [{v}]", ".byte 0xD8, 0xC8", "fld dword ptr [{v} + 4]", ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1", "fsqrt", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild i; fsub a; fmul k; fadd a; fistp`: a + (i - a) k as a whole number.
pub fn int_toward_to_int(i: i32, a: u32, k: u32) -> i32 {
    let (v, mut out) = ([i as u32, a, k], 0i32);
    x87!("fild dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fadd dword ptr [{v} + 4]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul k; fist; fcomp z; fnstsw`: the product as a whole number, and the
/// compare's C0 .. C3 (0x4700) of the product (unrounded) against z.
pub fn times_fist_compare(a: u32, k: u32, z: u32) -> (i32, u16) {
    let (v, mut out, mut status) = ([a, k, z], 0i32, 0u16);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fist dword ptr [{o}]",
        "fcomp dword ptr [{v} + 8]", "fnstsw word ptr [{s}]";
        v = v.as_ptr(), o = &raw mut out, s = &raw mut status);
    (out, status & 0x4700)
}

/// `fild n; fild m; fsub one; fsubr st(1), st; fdivp st(1), st; fld hi; fsub lo; fmulp;
/// fadd lo; fstp`: lo + ((m - 1) - n) / (m - 1) (hi - lo).
pub fn start_share(n: i32, m: i32, one: u32, hi: u32, lo: u32) -> u32 {
    let (v, mut out) = ([n as u32, m as u32, one, hi, lo], 0u32);
    x87!("fild dword ptr [{v}]", "fild dword ptr [{v} + 4]", "fsub dword ptr [{v} + 8]",
        ".byte 0xDC, 0xE1", ".byte 0xDE, 0xF9", "fld dword ptr [{v} + 12]",
        "fsub dword ptr [{v} + 16]", ".byte 0xDE, 0xC9", "fadd dword ptr [{v} + 16]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fdiv b; fmul c; fmul d; fmul e; fistp`.
pub fn div_mul3_to_int(a: u32, b: u32, c: u32, d: u32, e: u32) -> i32 {
    let (v, mut out) = ([a, b, c, d, e], 0i32);
    x87!("fld dword ptr [{v}]", "fdiv dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fmul dword ptr [{v} + 12]", "fmul dword ptr [{v} + 16]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld e (80 bits); fdiv a; fdiv b; fstp`.
pub fn ext_div2(e: [u8; 10], a: u32, b: u32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fld tbyte ptr [{e}]", "fdiv dword ptr [{v}]", "fdiv dword ptr [{v} + 4]",
        "fstp dword ptr [{o}]";
        e = e.as_ptr(), v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld t0; fadd t1; ..; fdiv d0; fdiv d1; ..; fstp`: the terms' sum over the divisors,
/// the 80 bits carried between the steps through memory (exactly).
pub fn sum_over(terms: &[u32], divisors: &[u32]) -> u32 {
    let mut acc = [0u8; 10];
    x87!("fld dword ptr [{a}]", "fstp tbyte ptr [{o}]"; a = &raw const terms[0], o = acc.as_mut_ptr());
    for t in &terms[1..] {
        x87!("fld tbyte ptr [{o}]", "fadd dword ptr [{t}]", "fstp tbyte ptr [{o}]";
            o = acc.as_mut_ptr(), t = t as *const u32);
    }
    for d in divisors {
        x87!("fld tbyte ptr [{o}]", "fdiv dword ptr [{d}]", "fstp tbyte ptr [{o}]";
            o = acc.as_mut_ptr(), d = d as *const u32);
    }
    crate::scene::matrix3::rounded(acc)
}

/// `fld x; fmul st, st; fmul k; fdiv d; fstp`.
pub fn square_times_over(x: u32, k: u32, d: u32) -> u32 {
    let (v, mut out) = ([x, k, d], 0u32);
    x87!("fld dword ptr [{v}]", ".byte 0xD8, 0xC8", "fmul dword ptr [{v} + 4]",
        "fdiv dword ptr [{v} + 8]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld1; fsub a; fmul b; fadd a; fstp`: a + (1 - a) b.
pub fn toward_one(a: u32, b: u32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fld1", "fsub dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fadd dword ptr [{v}]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fsub b; fmul k; fimul i; fadd b; fdiv d; fdiv w; fstp`.
pub fn lerp_int_over2(a: u32, b: u32, k: u32, i: i32, d: u32, w: u32) -> u32 {
    let (v, mut out) = ([a, b, k, i as u32, d, w], 0u32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fimul dword ptr [{v} + 12]", "fadd dword ptr [{v} + 4]", "fdiv dword ptr [{v} + 16]",
        "fdiv dword ptr [{v} + 20]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld top; fsub low; fild n; fsub one; fdivp st(1), st`: the step between gears,
/// unrounded.
pub fn gear_step_80(top: u32, low: u32, n: i32, one: u32) -> [u8; 10] {
    let (v, mut out) = ([top, low, n as u32, one], [0u8; 10]);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fild dword ptr [{v} + 8]",
        "fsub dword ptr [{v} + 12]", ".byte 0xDE, 0xF9", "fstp tbyte ptr [{o}]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    out
}

/// `fadd st, st(1)` of two 80-bit values (a in st(0), b in st(1)), unrounded.
pub fn ext_add(a: [u8; 10], b: [u8; 10]) -> [u8; 10] {
    let mut out = [0u8; 10];
    x87!("fld tbyte ptr [{b}]", "fld tbyte ptr [{a}]", ".byte 0xD8, 0xC1", "fstp tbyte ptr [{o}]",
        "fstp st(0)";
        a = a.as_ptr(), b = b.as_ptr(), o = out.as_mut_ptr());
    out
}

/// `fld v; fsub one; fmul a; fadd one; fmul g; fmul m; fchs; fmul a; fstp`.
pub fn weight_force(v: u32, one: u32, a: u32, g: u32, m: u32) -> u32 {
    let (x, mut out) = ([v, one, a, g, m], 0u32);
    x87!("fld dword ptr [{x}]", "fsub dword ptr [{x} + 4]", "fmul dword ptr [{x} + 8]",
        "fadd dword ptr [{x} + 4]", "fmul dword ptr [{x} + 12]", "fmul dword ptr [{x} + 16]",
        "fchs", "fmul dword ptr [{x} + 8]", "fstp dword ptr [{o}]";
        x = x.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul b; fmul c; fchs; fadd d; fstp`: d less a b c.
pub fn neg_product2_plus(a: u32, b: u32, c: u32, d: u32) -> u32 {
    let (v, mut out) = ([a, b, c, d], 0u32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fchs", "fadd dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}
