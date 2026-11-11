//! The x87 assembly versions of the helpers in `math/x87.rs` from its "part B" line, kept as the reference the
//! plain Rust versions are tested against (phase 6, 2026-10-24).

use crate::math::x87::x87;

/// `fld a; fsub b; [fchs;] fcomp c; fstsw`: the status word's C0 .. C2.
pub(super) fn sub_compare(a: u32, b: u32, negate: bool, c: u32) -> u16 {
    let mut status: u16 = 0;
    if negate {
        x87!(
            "fld dword ptr [{a}]", "fsub dword ptr [{b}]", "fchs", "fcomp dword ptr [{c}]",
            "fstsw word ptr [{s}]";
            a = &raw const a, b = &raw const b, c = &raw const c, s = &raw mut status
        );
    } else {
        x87!(
            "fld dword ptr [{a}]", "fsub dword ptr [{b}]", "fcomp dword ptr [{c}]",
            "fstsw word ptr [{s}]";
            a = &raw const a, b = &raw const b, c = &raw const c, s = &raw mut status
        );
    }
    status & 0x700
}

/// `fild r; fmul m; fadd c; fld a; fsub b; fmul s; fadd e; fmul k; fmulp; fstp` with `v`
/// [m, c, a, b, s, e, k]: (r m + c) ((a - b) s + e) k, the factors unrounded.
pub(super) fn random_step(r: i32, v: [u32; 7]) -> u32 {
    let mut out = 0u32;
    x87!(
        "fild dword ptr [{r}]", "fmul dword ptr [{v}]", "fadd dword ptr [{v} + 4]",
        "fld dword ptr [{v} + 8]", "fsub dword ptr [{v} + 12]", "fmul dword ptr [{v} + 16]",
        "fadd dword ptr [{v} + 20]", "fmul dword ptr [{v} + 24]", ".byte 0xDE, 0xC9",
        "fstp dword ptr [{o}]";
        r = &raw const r, v = v.as_ptr(), o = &raw mut out
    );
    out
}

/// `fild y; fadd a; fsub b; fmul st, st; fstp`.
pub(super) fn int_plus_less_squared(y: i32, a: u32, b: u32) -> u32 {
    let mut out = 0u32;
    x87!(
        "fild dword ptr [{y}]", "fadd dword ptr [{a}]", "fsub dword ptr [{b}]",
        ".byte 0xD8, 0xC8", "fstp dword ptr [{o}]";
        y = &raw const y, a = &raw const a, b = &raw const b, o = &raw mut out
    );
    out
}

/// One of the marks' kind records (0x30 bytes, 12 floats): +0xC and +0x24 negated in
/// place, then each of the twelve times a / b (kept unrounded) rounded to a whole number in
/// place (`fmul st, st(1)`; the last `fmulp`).
pub(super) fn scale_record(r: *mut u32, a: u32, b: u32) {
    x87!(
        "fld dword ptr [{r} + 0xC]", "fchs", "fstp dword ptr [{r} + 0xC]",
        "fld dword ptr [{r} + 0x24]", "fchs", "fstp dword ptr [{r} + 0x24]",
        "fld dword ptr [{a}]", "fdiv dword ptr [{b}]",
        "fld dword ptr [{r}]", ".byte 0xD8, 0xC9", "fistp dword ptr [{r}]",
        "fld dword ptr [{r} + 4]", ".byte 0xD8, 0xC9", "fistp dword ptr [{r} + 4]",
        "fld dword ptr [{r} + 8]", ".byte 0xD8, 0xC9", "fistp dword ptr [{r} + 8]",
        "fld dword ptr [{r} + 0xC]", ".byte 0xD8, 0xC9", "fistp dword ptr [{r} + 0xC]",
        "fld dword ptr [{r} + 0x10]", ".byte 0xD8, 0xC9", "fistp dword ptr [{r} + 0x10]",
        "fld dword ptr [{r} + 0x14]", ".byte 0xD8, 0xC9", "fistp dword ptr [{r} + 0x14]",
        "fld dword ptr [{r} + 0x18]", ".byte 0xD8, 0xC9", "fistp dword ptr [{r} + 0x18]",
        "fld dword ptr [{r} + 0x1C]", ".byte 0xD8, 0xC9", "fistp dword ptr [{r} + 0x1C]",
        "fld dword ptr [{r} + 0x20]", ".byte 0xD8, 0xC9", "fistp dword ptr [{r} + 0x20]",
        "fld dword ptr [{r} + 0x24]", ".byte 0xD8, 0xC9", "fistp dword ptr [{r} + 0x24]",
        "fld dword ptr [{r} + 0x28]", ".byte 0xD8, 0xC9", "fistp dword ptr [{r} + 0x28]",
        "fld dword ptr [{r} + 0x2C]", ".byte 0xDE, 0xC9", "fistp dword ptr [{r} + 0x2C]";
        r = r, a = &raw const a, b = &raw const b
    );
}

/// `fild a; fmul m; fadd c; fst` (the first), then `fadd d; fsub e; fstp` (the second).
pub(super) fn int_times_plus_on(a: i32, m: u32, c: u32, d: u32, e: u32) -> (u32, u32) {
    let (v, mut out) = ([a as u32, m, c, d, e], [0u32; 2]);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]",
        "fst dword ptr [{o}]", "fadd dword ptr [{v} + 12]", "fsub dword ptr [{v} + 16]",
        "fstp dword ptr [{o} + 4]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    (out[0], out[1])
}

/// `fld a; fsub b; fsub b; fsub b; fstp`.
pub(super) fn sub_thrice(a: u32, b: u32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fsub dword ptr [{v} + 4]",
        "fsub dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fadd b; fstp`.
pub(super) fn int_plus(a: i32, b: u32) -> u32 {
    let (v, mut out) = ([a as u32, b], 0u32);
    x87!("fild dword ptr [{v}]", "fadd dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld t; fsub c; fmul k; fadd c; fstp`: c eased toward t by k.
pub(super) fn ease(t: u32, c: u32, k: u32) -> u32 {
    let (v, mut out) = ([t, c, k], 0u32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fadd dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fadd b; fistp`.
pub(super) fn int_plus_to_int(a: i32, b: u32) -> i32 {
    let (v, mut out) = ([a as u32, b], 0i32);
    x87!("fild dword ptr [{v}]", "fadd dword ptr [{v} + 4]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fldz; fisub a; fiadd b; fistp`.
pub(super) fn zero_less_int_plus_int(a: i32, b: i32) -> i32 {
    let (v, mut out) = ([a, b], 0i32);
    x87!("fldz", "fisub dword ptr [{v}]", "fiadd dword ptr [{v} + 4]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld f; fisub a; fiadd b; fistp`.
pub(super) fn less_int_plus_int(f: u32, a: i32, b: i32) -> i32 {
    let (v, mut out) = ([f, a as u32, b as u32], 0i32);
    x87!("fld dword ptr [{v}]", "fisub dword ptr [{v} + 4]", "fiadd dword ptr [{v} + 8]",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fldpi; fadd st, st; fsub a; fstp`: 2 pi less a.
pub(super) fn two_pi_less(a: u32) -> u32 {
    let mut out = 0u32;
    x87!("fldpi", ".byte 0xD8, 0xC0", "fsub dword ptr [{a}]", "fstp dword ptr [{o}]";
        a = &raw const a, o = &raw mut out);
    out
}

/// `fild a; fmul m; fadd c; fmul k; fstp`.
pub(super) fn int_times_plus_times(a: i32, m: u32, c: u32, k: u32) -> u32 {
    let (v, mut out) = ([a as u32, m, c, k], 0u32);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]",
        "fmul dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fidiv n; fstp`.
pub(super) fn over_int(a: u32, n: i32) -> u32 {
    let (v, mut out) = ([a, n as u32], 0u32);
    x87!("fld dword ptr [{v}]", "fidiv dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fldz; [fchs;] fmul b; fxch st(1); fmul c; faddp st(1), st; fmul k; fstp`:
/// (±0 b + a c) k, the zero's product kept.
pub(super) fn zero_times_plus_times(a: u32, b: u32, c: u32, k: u32, negative: bool) -> u32 {
    let (v, mut out) = ([a, b, c, k], 0u32);
    if negative {
        x87!("fld dword ptr [{v}]", "fldz", "fchs", "fmul dword ptr [{v} + 4]",
            ".byte 0xD9, 0xC9", "fmul dword ptr [{v} + 8]", ".byte 0xDE, 0xC1",
            "fmul dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
            v = v.as_ptr(), o = &raw mut out);
    } else {
        x87!("fld dword ptr [{v}]", "fldz", "fmul dword ptr [{v} + 4]",
            ".byte 0xD9, 0xC9", "fmul dword ptr [{v} + 8]", ".byte 0xDE, 0xC1",
            "fmul dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
            v = v.as_ptr(), o = &raw mut out);
    }
    out
}

/// `fld m; fild r; fprem; fadd v; fstp`: v plus the remainder of r by m.
pub(super) fn prem_plus(r: i32, m: u32, v: u32) -> u32 {
    let (w, mut out) = ([m, r as u32, v], 0u32);
    x87!("fld dword ptr [{w}]", "fild dword ptr [{w} + 4]", "fprem", "fadd dword ptr [{w} + 8]",
        "fstp dword ptr [{o}]", "fstp st(0)";
        w = w.as_ptr(), o = &raw mut out);
    out
}

/// `fild r; fmul qword k; fld v; fmul a; fmulp st(1), st; fadd v; fstp`: v grown by
/// v a r k.
pub(super) fn grown(r: i32, k: u64, v: u32, a: u32) -> u32 {
    let (w, mut out) = ([r as u32, v, a], 0u32);
    x87!("fild dword ptr [{w}]", "fmul qword ptr [{k}]", "fld dword ptr [{w} + 4]",
        "fmul dword ptr [{w} + 8]", ".byte 0xDE, 0xC9", "fadd dword ptr [{w} + 4]",
        "fstp dword ptr [{o}]";
        w = w.as_ptr(), k = &raw const k, o = &raw mut out);
    out
}

/// `fld a; fsub b; fabs; fmul k1; fmul k2; fadd k3; fdivr one; fsubr one; fmul k4;
/// fistp`: a rate by the difference of a and b.
pub(super) fn rate_to_int(a: u32, b: u32, k1: u32, k2: u32, k3: u32, one: u32, k4: u32) -> i32 {
    let (v, mut out) = ([a, b, k1, k2, k3, one, k4], 0i32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fabs",
        "fmul dword ptr [{v} + 8]", "fmul dword ptr [{v} + 12]", "fadd dword ptr [{v} + 16]",
        "fdivr dword ptr [{v} + 20]", "fsubr dword ptr [{v} + 20]", "fmul dword ptr [{v} + 24]",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fmul m1; fild b; fmul m2; faddp st(1), st; fadd d`, kept as 80 bits.
pub(super) fn pair_sum_80(a: i32, m1: u32, b: i32, m2: u32, d: u32) -> [u8; 10] {
    let (v, mut out) = ([a as u32, m1, b as u32, m2, d], [0u8; 10]);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fild dword ptr [{v} + 8]",
        "fmul dword ptr [{v} + 12]", ".byte 0xDE, 0xC1", "fadd dword ptr [{v} + 16]",
        "fstp tbyte ptr [{o}]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    out
}

/// `fld tbyte e; fsub f; fstp`.
pub(super) fn ext_less(e: [u8; 10], f: u32) -> u32 {
    let mut out = 0u32;
    x87!("fld tbyte ptr [{e}]", "fsub dword ptr [{f}]", "fstp dword ptr [{o}]";
        e = e.as_ptr(), f = &raw const f, o = &raw mut out);
    out
}

/// Three products of three, `fld; fmul; fmul` each, summed by `faddp st(1), st` in
/// order: (p0 + p1) + p2.
pub(super) fn three_products(p: [[u32; 3]; 3]) -> u32 {
    let mut out = 0u32;
    x87!("fld dword ptr [{p}]", "fmul dword ptr [{p} + 4]", "fmul dword ptr [{p} + 8]",
        "fld dword ptr [{p} + 12]", "fmul dword ptr [{p} + 16]", "fmul dword ptr [{p} + 20]",
        ".byte 0xDE, 0xC1",
        "fld dword ptr [{p} + 24]", "fmul dword ptr [{p} + 28]", "fmul dword ptr [{p} + 32]",
        ".byte 0xDE, 0xC1", "fstp dword ptr [{o}]";
        p = p.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul b; fst` (the first), `fabs; fmul c; fistp` (the second).
pub(super) fn times_kept_abs_to_int(a: u32, b: u32, c: u32) -> (u32, i32) {
    let (v, mut kept, mut out) = ([a, b, c], 0u32, 0i32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fst dword ptr [{k}]", "fabs",
        "fmul dword ptr [{v} + 8]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), k = &raw mut kept, o = &raw mut out);
    (kept, out)
}

/// `fld a; fsub b; fmul k1; fld b; fmul k2; faddp st(1), st; fst` (the first), `fabs;
/// fmul c; fistp` (the second).
pub(super) fn blend_kept_abs_to_int(a: u32, b: u32, k1: u32, k2: u32, c: u32) -> (u32, i32) {
    let (v, mut kept, mut out) = ([a, b, k1, k2, c], 0u32, 0i32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fld dword ptr [{v} + 4]", "fmul dword ptr [{v} + 12]", ".byte 0xDE, 0xC1",
        "fst dword ptr [{k}]", "fabs", "fmul dword ptr [{v} + 16]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), k = &raw mut kept, o = &raw mut out);
    (kept, out)
}

/// `fld f; fmul s; fld s; fabs; [fmul m;] fsubp st(1), st; fmul k; fstp`: (f s - |s| [m]) k.
pub(super) fn times_less_abs_times(f: u32, s: u32, m: Option<u32>, k: u32) -> u32 {
    let (v, mut out) = ([f, s, m.unwrap_or(0), k], 0u32);
    if m.is_some() {
        x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fld dword ptr [{v} + 4]",
            "fabs", "fmul dword ptr [{v} + 8]", ".byte 0xDE, 0xE9", "fmul dword ptr [{v} + 12]",
            "fstp dword ptr [{o}]";
            v = v.as_ptr(), o = &raw mut out);
    } else {
        x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fld dword ptr [{v} + 4]",
            "fabs", ".byte 0xDE, 0xE9", "fmul dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
            v = v.as_ptr(), o = &raw mut out);
    }
    out
}

/// `fld d; fsub m; fld f; fimul i; faddp st(1), st; fstp`: d less m plus f i.
pub(super) fn less_plus_int_times(d: u32, m: u32, f: u32, i: i32) -> u32 {
    let (v, mut out) = ([d, m, f, i as u32], 0u32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fld dword ptr [{v} + 8]",
        "fimul dword ptr [{v} + 12]", ".byte 0xDE, 0xC1", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul b; fimul i; [fadd c;] fmul s`, kept as 80 bits.
pub(super) fn times_int_plus_times_80(a: u32, b: u32, i: i32, c: Option<u32>, s: u32) -> [u8; 10] {
    let (v, mut out) = ([a, b, i as u32, c.unwrap_or(0), s], [0u8; 10]);
    if c.is_some() {
        x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fimul dword ptr [{v} + 8]",
            "fadd dword ptr [{v} + 12]", "fmul dword ptr [{v} + 16]", "fstp tbyte ptr [{o}]";
            v = v.as_ptr(), o = out.as_mut_ptr());
    } else {
        x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fimul dword ptr [{v} + 8]",
            "fmul dword ptr [{v} + 16]", "fstp tbyte ptr [{o}]";
            v = v.as_ptr(), o = out.as_mut_ptr());
    }
    out
}

/// `fild a; fmul m1; fild b; fmul m2; faddp st(1), st; fadd d; fsub e; fstp`.
pub(super) fn pair_sum_less(a: i32, m1: u32, b: i32, m2: u32, d: u32, e: u32) -> u32 {
    let (v, mut out) = ([a as u32, m1, b as u32, m2, d, e], 0u32);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fild dword ptr [{v} + 8]",
        "fmul dword ptr [{v} + 12]", ".byte 0xDE, 0xC1", "fadd dword ptr [{v} + 16]",
        "fsub dword ptr [{v} + 20]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul b; fmul c; faddp st(1), st` onto the float d, then `fstp`.
pub(super) fn product_plus(a: u32, b: u32, c: u32, d: u32) -> u32 {
    let (v, mut out) = ([d, a, b, c], 0u32);
    x87!("fld dword ptr [{v}]", "fld dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fmul dword ptr [{v} + 12]", ".byte 0xDE, 0xC1", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld f; fimul i; fistp`.
pub(super) fn float_times_int_to_int(f: u32, i: i32) -> i32 {
    let (v, mut out) = ([f, i as u32], 0i32);
    x87!("fld dword ptr [{v}]", "fimul dword ptr [{v} + 4]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul k1; fmul k2; fadd k3; fdivr one; fsubr one; fmul k4; fistp`: a rate by a.
pub(super) fn speed_rate_to_int(a: u32, k1: u32, k2: u32, k3: u32, one: u32, k4: u32) -> i32 {
    let (v, mut out) = ([a, k1, k2, k3, one, k4], 0i32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fadd dword ptr [{v} + 12]", "fdivr dword ptr [{v} + 16]", "fsubr dword ptr [{v} + 16]",
        "fmul dword ptr [{v} + 20]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fabs; fld b; fabs; faddp; fld a; fsub c; fabs; faddp; fadd d; fist` (the
/// first), `fstp` (the second): |a| + |b| + |a - c| + d.
pub(super) fn abs_sum_to_int_kept(a: u32, b: u32, c: u32, d: u32) -> (i32, u32) {
    let (v, mut n, mut kept) = ([a, b, c, d], 0i32, 0u32);
    x87!("fld dword ptr [{v}]", "fabs", "fld dword ptr [{v} + 4]", "fabs", ".byte 0xDE, 0xC1",
        "fld dword ptr [{v}]", "fsub dword ptr [{v} + 8]", "fabs", ".byte 0xDE, 0xC1",
        "fadd dword ptr [{v} + 12]", "fist dword ptr [{n}]", "fstp dword ptr [{k}]";
        v = v.as_ptr(), n = &raw mut n, k = &raw mut kept);
    (n, kept)
}

/// `fld a; fabs; fmul k1; fld b; fsub c; fabs; faddp; fmul k2; fst` (the first), `fcomp
/// k3; fstsw ax` (the second, the status word).
pub(super) fn abs_blend_compare(a: u32, k1: u32, b: u32, c: u32, k2: u32, k3: u32) -> (u32, u16) {
    let (v, mut kept, mut status) = ([a, k1, b, c, k2, k3], 0u32, 0u16);
    x87!("fld dword ptr [{v}]", "fabs", "fmul dword ptr [{v} + 4]", "fld dword ptr [{v} + 8]",
        "fsub dword ptr [{v} + 12]", "fabs", ".byte 0xDE, 0xC1", "fmul dword ptr [{v} + 16]",
        "fst dword ptr [{k}]", "fcomp dword ptr [{v} + 20]", "fnstsw word ptr [{s}]";
        v = v.as_ptr(), k = &raw mut kept, s = &raw mut status);
    (kept, status)
}

/// `fld v; fmul a; fadd b; fistp`.
pub(super) fn times_plus_to_int(v: u32, a: u32, b: u32) -> i32 {
    let (x, mut out) = ([v, a, b], 0i32);
    x87!("fld dword ptr [{x}]", "fmul dword ptr [{x} + 4]", "fadd dword ptr [{x} + 8]",
        "fistp dword ptr [{o}]";
        x = x.as_ptr(), o = &raw mut out);
    out
}

/// `fld d; fld f; fimul i; faddp st(1), st`, kept as 80 bits.
pub(super) fn plus_int_times_80(d: u32, f: u32, i: i32) -> [u8; 10] {
    let (v, mut out) = ([d, f, i as u32], [0u8; 10]);
    x87!("fld dword ptr [{v}]", "fld dword ptr [{v} + 4]", "fimul dword ptr [{v} + 8]",
        ".byte 0xDE, 0xC1", "fstp tbyte ptr [{o}]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    out
}

/// `three_products` kept as 80 bits.
pub(super) fn three_products_80(p: [[u32; 3]; 3]) -> [u8; 10] {
    let mut out = [0u8; 10];
    x87!("fld dword ptr [{p}]", "fmul dword ptr [{p} + 4]", "fmul dword ptr [{p} + 8]",
        "fld dword ptr [{p} + 12]", "fmul dword ptr [{p} + 16]", "fmul dword ptr [{p} + 20]",
        ".byte 0xDE, 0xC1",
        "fld dword ptr [{p} + 24]", "fmul dword ptr [{p} + 28]", "fmul dword ptr [{p} + 32]",
        ".byte 0xDE, 0xC1", "fstp tbyte ptr [{o}]";
        p = p.as_ptr(), o = out.as_mut_ptr());
    out
}

/// `fld a; fmul k; fcom c; fstsw ax` (then dropped): the status word.
pub(super) fn times_compare(a: u32, k: u32, c: u32) -> u16 {
    let (v, mut status) = ([a, k, c], 0u16);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fcom dword ptr [{v} + 8]",
        "fnstsw word ptr [{s}]", "fstp st(0)";
        v = v.as_ptr(), s = &raw mut status);
    status
}

/// `fld a; fmul k; fimul n; fistp`.
pub(super) fn times_int_to_int(a: u32, k: u32, n: i32) -> i32 {
    let (v, mut out) = ([a, k, n as u32], 0i32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fimul dword ptr [{v} + 8]",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fmul b; fadd c; fst` (the first), `fchs; fst` (the second).
pub(super) fn int_times_plus_and_negated(a: i32, b: u32, c: u32) -> (u32, u32) {
    let (v, mut kept, mut negated) = ([a as u32, b, c], 0u32, 0u32);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]",
        "fst dword ptr [{k}]", "fchs", "fstp dword ptr [{n}]";
        v = v.as_ptr(), k = &raw mut kept, n = &raw mut negated);
    (kept, negated)
}

/// `fild a; fmul b; fsub c; fsub d; fstp`.
pub(super) fn int_times_less_less(a: i32, b: u32, c: u32, d: u32) -> u32 {
    let (v, mut out) = ([a as u32, b, c, d], 0u32);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fsub dword ptr [{v} + 8]",
        "fsub dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fabs; fld b; fabs; faddp; fmul k; fistp`.
pub(super) fn abs_sum_times_to_int(a: u32, b: u32, k: u32) -> i32 {
    let (v, mut out) = ([a, b, k], 0i32);
    x87!("fld dword ptr [{v}]", "fabs", "fld dword ptr [{v} + 4]", "fabs", ".byte 0xDE, 0xC1",
        "fmul dword ptr [{v} + 8]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `effects::emit_exhaust_smoke`'s point: (-t0 k, (t1 - d) k, t2 k) turned by the matrix at `m` (its
/// rows 0x10 apart), placed by +0x30 .. and the place taken off again, all on the x87 stack as
/// the original does (`fxch`, register forms as bytes); then x, y, y less `lift`, z.
pub(super) fn turned_place_less(t: [u32; 3], k: u32, d: u32, lift: u32, m: u32) -> [u32; 4] {
    let (v, mut out) = ([t[0], t[1], t[2], k, d, lift], [0u32; 4]);
    x87!(
        "fld dword ptr [{v}]", "fchs", "fmul dword ptr [{v} + 12]",
        "fld dword ptr [{v} + 4]", "fsub dword ptr [{v} + 16]", "fmul dword ptr [{v} + 12]",
        "fld dword ptr [{v} + 8]", "fmul dword ptr [{v} + 12]",
        "fld dword ptr [{m}]", ".byte 0xD8, 0xC9",
        "fld dword ptr [{m} + 0x10]", ".byte 0xD8, 0xCB",
        "fld dword ptr [{m} + 0x20]", ".byte 0xD8, 0xCD",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fadd dword ptr [{m} + 0x30]",
        ".byte 0xD9, 0xCB",
        "fld dword ptr [{m} + 4]", ".byte 0xD8, 0xCA",
        "fld dword ptr [{m} + 0x14]", ".byte 0xD8, 0xCC",
        "fld dword ptr [{m} + 0x24]", ".byte 0xD8, 0xCB",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fadd dword ptr [{m} + 0x34]",
        ".byte 0xD9, 0xCB",
        "fld dword ptr [{m} + 8]", ".byte 0xDE, 0xCB",
        "fld dword ptr [{m} + 0x18]", ".byte 0xDE, 0xC9",
        "fld dword ptr [{m} + 0x28]", ".byte 0xDE, 0xCA",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fadd dword ptr [{m} + 0x38]",
        ".byte 0xD9, 0xCA",
        "fsub dword ptr [{m} + 0x30]", "fstp dword ptr [{o}]",
        "fsub dword ptr [{m} + 0x34]", "fst dword ptr [{o} + 4]",
        "fsub dword ptr [{v} + 20]", "fstp dword ptr [{o} + 8]",
        "fsub dword ptr [{m} + 0x38]", "fstp dword ptr [{o} + 12]";
        v = v.as_ptr(), m = m, o = out.as_mut_ptr()
    );
    out
}

/// `fild r; fmul k` kept, then for each of three: `fld p; fmul st, st(1); fimul ticks;
/// fadd at; fstp` (the last `fmulp`).
pub(super) fn drifted(r: i32, k: u32, p: [u32; 3], ticks: i32, at: [u32; 3]) -> [u32; 3] {
    let v = [
        r as u32,
        k,
        p[0],
        p[1],
        p[2],
        ticks as u32,
        at[0],
        at[1],
        at[2],
    ];
    let mut out = [0u32; 3];
    x87!(
        "fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]",
        "fld dword ptr [{v} + 8]", ".byte 0xD8, 0xC9", "fimul dword ptr [{v} + 20]",
        "fadd dword ptr [{v} + 24]", "fstp dword ptr [{o}]",
        "fld dword ptr [{v} + 12]", ".byte 0xD8, 0xC9", "fimul dword ptr [{v} + 20]",
        "fadd dword ptr [{v} + 28]", "fstp dword ptr [{o} + 4]",
        "fld dword ptr [{v} + 16]", ".byte 0xDE, 0xC9", "fimul dword ptr [{v} + 20]",
        "fadd dword ptr [{v} + 32]", "fstp dword ptr [{o} + 8]";
        v = v.as_ptr(), o = out.as_mut_ptr()
    );
    out
}

/// `fld a; fmul k; fadd one; fdivr one; fsubr one; fstp`: 1 - 1 / (a k + 1).
pub(super) fn one_less_inverse(a: u32, k: u32, one: u32) -> u32 {
    let (v, mut out) = ([a, k, one], 0u32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]",
        "fdivr dword ptr [{v} + 8]", "fsubr dword ptr [{v} + 8]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul k; fidiv n; fstp`.
pub(super) fn times_over_int(a: u32, k: u32, n: i32) -> u32 {
    let (v, mut out) = ([a, k, n as u32], 0u32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fidiv dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul b`, kept as 80 bits.
pub(super) fn mul_80(a: u32, b: u32) -> [u8; 10] {
    let (v, mut out) = ([a, b], [0u8; 10]);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fstp tbyte ptr [{o}]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    out
}

/// The point (st(0) p0, st(1) p1, st(2) p2) turned and placed by the matrix at `m` on the
/// x87 stack as `effects::emit_twin_particle_first` does (`fxch`, register forms as bytes); then x, y, y
/// less
/// `lift`, z.
pub(super) fn turned_place(p: [u32; 3], lift: u32, m: u32) -> [u32; 4] {
    let (v, mut out) = ([p[0], p[1], p[2], lift], [0u32; 4]);
    x87!(
        "fld dword ptr [{v} + 8]", "fld dword ptr [{v} + 4]", "fld dword ptr [{v}]",
        "fld dword ptr [{m}]", ".byte 0xD8, 0xC9",
        "fld dword ptr [{m} + 0x10]", ".byte 0xD8, 0xCB",
        "fld dword ptr [{m} + 0x20]", ".byte 0xD8, 0xCD",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fadd dword ptr [{m} + 0x30]",
        ".byte 0xD9, 0xCB",
        "fld dword ptr [{m} + 4]", ".byte 0xD8, 0xCA",
        "fld dword ptr [{m} + 0x14]", ".byte 0xD8, 0xCC",
        "fld dword ptr [{m} + 0x24]", ".byte 0xD8, 0xCB",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fadd dword ptr [{m} + 0x34]",
        ".byte 0xD9, 0xCB",
        "fld dword ptr [{m} + 8]", ".byte 0xDE, 0xCB",
        "fld dword ptr [{m} + 0x18]", ".byte 0xDE, 0xC9",
        "fld dword ptr [{m} + 0x28]", ".byte 0xDE, 0xCA",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fadd dword ptr [{m} + 0x38]",
        ".byte 0xD9, 0xCA",
        "fstp dword ptr [{o}]", "fst dword ptr [{o} + 4]",
        "fsub dword ptr [{v} + 12]", "fstp dword ptr [{o} + 8]",
        "fstp dword ptr [{o} + 12]";
        v = v.as_ptr(), m = m, o = out.as_mut_ptr()
    );
    out
}

/// `fild a; fimul b; fistp`.
pub(super) fn int_times_int_to_int(a: i32, b: i32) -> i32 {
    let (v, mut out) = ([a as u32, b as u32], 0i32);
    x87!("fild dword ptr [{v}]", "fimul dword ptr [{v} + 4]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fadd b; fadd c; fadd d; fmul k; fstp`.
pub(super) fn sum4_times(a: u32, b: u32, c: u32, d: u32, k: u32) -> u32 {
    let (v, mut out) = ([a, b, c, d, k], 0u32);
    x87!("fld dword ptr [{v}]", "fadd dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]",
        "fadd dword ptr [{v} + 12]", "fmul dword ptr [{v} + 16]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// The length of (a, b, c) (`fmul st, st` each, `faddp` twice, `fsqrt`) `fist` (the
/// first); then `fmul k1; fadd k2; fdivr one; fsubr one; fmul k3; fistp` (the second).
pub(super) fn length_and_rate(p: [u32; 3], k1: u32, k2: u32, one: u32, k3: u32) -> (i32, i32) {
    let (v, mut length, mut rate) = ([p[0], p[1], p[2], k1, k2, one, k3], 0i32, 0i32);
    x87!(
        "fld dword ptr [{v}]", ".byte 0xD8, 0xC8", "fld dword ptr [{v} + 4]", ".byte 0xD8, 0xC8",
        "fld dword ptr [{v} + 8]", ".byte 0xD8, 0xC8", ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1",
        "fsqrt", "fist dword ptr [{l}]", "fmul dword ptr [{v} + 12]", "fadd dword ptr [{v} + 16]",
        "fdivr dword ptr [{v} + 20]", "fsubr dword ptr [{v} + 20]", "fmul dword ptr [{v} + 24]",
        "fistp dword ptr [{r}]";
        v = v.as_ptr(), l = &raw mut length, r = &raw mut rate
    );
    (length, rate)
}

/// (a, b, c) by one over its length, the factor kept unrounded (`fld st; fmul; fstp`
/// twice, the last `fmul; fstp`).
pub(super) fn normalised3(p: [u32; 3], one: u32) -> [u32; 3] {
    let (v, mut out) = ([p[0], p[1], p[2], one], [0u32; 3]);
    x87!(
        "fld dword ptr [{v}]", ".byte 0xD8, 0xC8", "fld dword ptr [{v} + 4]", ".byte 0xD8, 0xC8",
        "fld dword ptr [{v} + 8]", ".byte 0xD8, 0xC8", ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1",
        "fsqrt", "fdivr dword ptr [{v} + 12]",
        ".byte 0xD9, 0xC0", "fmul dword ptr [{v}]", "fstp dword ptr [{o}]",
        ".byte 0xD9, 0xC0", "fmul dword ptr [{v} + 4]", "fstp dword ptr [{o} + 4]",
        "fmul dword ptr [{v} + 8]", "fstp dword ptr [{o} + 8]";
        v = v.as_ptr(), o = out.as_mut_ptr()
    );
    out
}

/// `effects::cars_particles`'s third side: the cross product of (p0, p1, p2) and the vector at `w`
/// (unrounded) turned by the transposed matrix at `m` (rows 0x10 apart), as the original
/// does it on the x87 stack (`fxch`, register forms as bytes).
pub(super) fn crossed_turned(p: [u32; 3], w: u32, m: u32) -> [u32; 3] {
    let mut out = [0u32; 3];
    x87!(
        "fld dword ptr [{p}]", "fmul dword ptr [{w} + 4]",
        "fld dword ptr [{p} + 4]", "fmul dword ptr [{w}]", ".byte 0xDE, 0xE9",
        "fld dword ptr [{p} + 8]", "fmul dword ptr [{w}]",
        "fld dword ptr [{p}]", "fmul dword ptr [{w} + 8]", ".byte 0xDE, 0xE9",
        "fld dword ptr [{p} + 4]", "fmul dword ptr [{w} + 8]",
        "fld dword ptr [{p} + 8]", "fmul dword ptr [{w} + 4]", ".byte 0xDE, 0xE9",
        "fld dword ptr [{m}]", ".byte 0xD8, 0xC9",
        "fld dword ptr [{m} + 0x10]", ".byte 0xD8, 0xCB",
        "fld dword ptr [{m} + 0x20]", ".byte 0xD8, 0xCD",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1",
        ".byte 0xD9, 0xCB",
        "fld dword ptr [{m} + 4]", ".byte 0xD8, 0xCA",
        "fld dword ptr [{m} + 0x14]", ".byte 0xD8, 0xCC",
        "fld dword ptr [{m} + 0x24]", ".byte 0xD8, 0xCB",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1",
        ".byte 0xD9, 0xCB",
        "fld dword ptr [{m} + 8]", ".byte 0xDE, 0xCB",
        "fld dword ptr [{m} + 0x18]", ".byte 0xDE, 0xC9",
        "fld dword ptr [{m} + 0x28]", ".byte 0xDE, 0xCA",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1",
        ".byte 0xD9, 0xCA",
        "fstp dword ptr [{o}]", "fstp dword ptr [{o} + 4]", "fstp dword ptr [{o} + 8]";
        p = p.as_ptr(), w = w, m = m, o = out.as_mut_ptr()
    );
    out
}

/// The vector at `p` made of length 1 as `effects::aim_vector` does it: the length (`fmul st, st`
/// each, `faddp` twice, `fsqrt`) `fst` (the second), `fld1; fdivrp`, each by it, stored z
/// first, the factor dropped (`ffree st; fincstp`).
pub(super) fn unit_and_length(p: [u32; 3]) -> ([u32; 3], u32) {
    let (mut out, mut length) = ([0u32; 3], 0u32);
    x87!(
        "fld dword ptr [{p}]", ".byte 0xD8, 0xC8", "fld dword ptr [{p} + 4]", ".byte 0xD8, 0xC8",
        "fld dword ptr [{p} + 8]", ".byte 0xD8, 0xC8", ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1",
        "fsqrt", "fst dword ptr [{l}]", "fld1", ".byte 0xDE, 0xF1",
        "fld dword ptr [{p}]", ".byte 0xD8, 0xC9", "fld dword ptr [{p} + 4]", ".byte 0xD8, 0xCA",
        "fld dword ptr [{p} + 8]", ".byte 0xD8, 0xCB",
        "fstp dword ptr [{o} + 8]", "fstp dword ptr [{o} + 4]", "fstp dword ptr [{o}]",
        ".byte 0xDD, 0xC0", ".byte 0xD9, 0xF7";
        p = p.as_ptr(), l = &raw mut length, o = out.as_mut_ptr()
    );
    (out, length)
}

/// `fld a; fchs; fsub b; fadd c; fadd d; fstp`.
pub(super) fn negated_less_plus_plus(a: u32, b: u32, c: u32, d: u32) -> u32 {
    let (v, mut out) = ([a, b, c, d], 0u32);
    x87!("fld dword ptr [{v}]", "fchs", "fsub dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]",
        "fadd dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld t; fsin; fld t; fcos; fstp` (the cosine), `fstp` (the sine).
pub(super) fn sine_and_cosine(t: u32) -> (u32, u32) {
    let (mut s, mut c) = (0u32, 0u32);
    x87!("fld dword ptr [{t}]", "fsin", "fld dword ptr [{t}]", "fcos",
        "fstp dword ptr [{c}]", "fstp dword ptr [{s}]";
        t = &raw const t, c = &raw mut c, s = &raw mut s);
    (s, c)
}

/// `effects::tilt_shape`'s turn of (y, z) by the stored sine and cosine: z c - y s and z s + y c
/// (`fsubp`, `faddp` on the stack), returned as (new y, new z).
pub(super) fn turn_yz(y: u32, z: u32, s: u32, c: u32) -> (u32, u32) {
    let (v, mut out) = ([y, z, s, c], [0u32; 2]);
    x87!(
        "fld dword ptr [{v} + 4]", "fmul dword ptr [{v} + 12]",
        "fld dword ptr [{v}]", "fmul dword ptr [{v} + 8]", ".byte 0xDE, 0xE9",
        "fld dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fld dword ptr [{v}]", "fmul dword ptr [{v} + 12]", ".byte 0xDE, 0xC1",
        "fstp dword ptr [{o}]", "fstp dword ptr [{o} + 4]";
        v = v.as_ptr(), o = out.as_mut_ptr()
    );
    (out[0], out[1])
}

/// `fild a; fmul k; fiadd b; fistp`.
pub(super) fn int_times_plus_int_to_int(a: i32, k: u32, b: i32) -> i32 {
    let (v, mut out) = ([a as u32, k, b as u32], 0i32);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fiadd dword ptr [{v} + 8]",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fmul ka; fild b; fmul kb; fild c; fmul kc; faddp; faddp`, kept as 80 bits:
/// a ka + (b kb + c kc).
pub(super) fn blend3_80(a: i32, b: i32, c: i32, k: [u32; 3]) -> [u8; 10] {
    let (v, mut out) = ([a as u32, k[0], b as u32, k[1], c as u32, k[2]], [0u8; 10]);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fild dword ptr [{v} + 8]",
        "fmul dword ptr [{v} + 12]", "fild dword ptr [{v} + 16]", "fmul dword ptr [{v} + 20]",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fstp tbyte ptr [{o}]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    out
}

/// `fld a; fild v; fdiv kd; fmul km; faddp; fst` (the first), `fsin; fmul ks; fstp` (the
/// second).
pub(super) fn angle_and_wave(a: u32, v: i32, kd: u32, km: u32, ks: u32) -> (u32, u32) {
    let (x, mut angle, mut wave) = ([a, v as u32, kd, km, ks], 0u32, 0u32);
    x87!("fld dword ptr [{x}]", "fild dword ptr [{x} + 4]", "fdiv dword ptr [{x} + 8]",
        "fmul dword ptr [{x} + 12]", ".byte 0xDE, 0xC1", "fst dword ptr [{a}]", "fsin",
        "fmul dword ptr [{x} + 16]", "fstp dword ptr [{w}]";
        x = x.as_ptr(), a = &raw mut angle, w = &raw mut wave);
    (angle, wave)
}

/// `fild a; fisub b; fmul f; fiadd b; fistp`: b + (a - b) f, rounded.
pub(super) fn int_lerp_to_int(a: i32, b: i32, f: u32) -> i32 {
    let (v, mut out) = ([a as u32, b as u32, f], 0i32);
    x87!("fild dword ptr [{v}]", "fisub dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fiadd dword ptr [{v} + 4]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fidiv b; fmul k; fstp`.
pub(super) fn int_over_int_times(a: i32, b: i32, k: u32) -> u32 {
    let (v, mut out) = ([a as u32, b as u32, k], 0u32);
    x87!("fild dword ptr [{v}]", "fidiv dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `effects::clip_quad`'s crossing: t = (m - fc) / (fs - fc) (`fld m; fsub fc; fld fs; fsub fc;
/// fdivp st(1), st`), then for each (s, c) pair (s - c) t + c (`fld s; fsub c; fmul st,
/// st(1)`, the last `fmulp`; `fadd c; fstp`), t kept unrounded.
pub(super) fn clip_lerp3(m: u32, fc: u32, fs: u32, pairs: [[u32; 2]; 3]) -> [u32; 3] {
    let v = [
        m,
        fc,
        fs,
        pairs[0][0],
        pairs[0][1],
        pairs[1][0],
        pairs[1][1],
        pairs[2][0],
        pairs[2][1],
    ];
    let mut out = [0u32; 3];
    x87!(
        "fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fld dword ptr [{v} + 8]",
        "fsub dword ptr [{v} + 4]", ".byte 0xDE, 0xF9",
        "fld dword ptr [{v} + 12]", "fsub dword ptr [{v} + 16]", ".byte 0xD8, 0xC9",
        "fadd dword ptr [{v} + 16]", "fstp dword ptr [{o}]",
        "fld dword ptr [{v} + 20]", "fsub dword ptr [{v} + 24]", ".byte 0xD8, 0xC9",
        "fadd dword ptr [{v} + 24]", "fstp dword ptr [{o} + 4]",
        "fld dword ptr [{v} + 28]", "fsub dword ptr [{v} + 32]", ".byte 0xDE, 0xC9",
        "fadd dword ptr [{v} + 32]", "fstp dword ptr [{o} + 8]";
        v = v.as_ptr(), o = out.as_mut_ptr()
    );
    out
}

/// `effects::draw_mark_slots`'s projection: q = focal / z (`fld; fdiv`), kept; x' = q x + cx (`fld
/// st; fmul; fadd; fstp`), y' = -(q y) + cy (`fld st; fmul; fchs; fadd; fstp`), s = q k
/// (`fmul; fstp`).
pub(super) fn slot_project(
    focal: u32,
    z: u32,
    x: u32,
    cx: u32,
    y: u32,
    cy: u32,
    k: u32,
) -> [u32; 3] {
    let v = [focal, z, x, cx, y, cy, k];
    let mut out = [0u32; 3];
    x87!(
        "fld dword ptr [{v}]", "fdiv dword ptr [{v} + 4]",
        ".byte 0xD9, 0xC0", "fmul dword ptr [{v} + 8]", "fadd dword ptr [{v} + 12]",
        "fstp dword ptr [{o}]",
        ".byte 0xD9, 0xC0", "fmul dword ptr [{v} + 16]", "fchs", "fadd dword ptr [{v} + 20]",
        "fstp dword ptr [{o} + 4]",
        "fmul dword ptr [{v} + 24]", "fstp dword ptr [{o} + 8]";
        v = v.as_ptr(), o = out.as_mut_ptr()
    );
    out
}

/// `fild i; fmul a; fmul b; fstp`.
pub(super) fn int_times_times(i: i32, a: u32, b: u32) -> u32 {
    let (v, mut out) = ([i as u32, a, b], 0u32);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fld b; fsub a; fmul r; faddp; fstp`: a + (b - a) r.
pub(super) fn toward(a: u32, b: u32, r: u32) -> u32 {
    let (v, mut out) = ([a, b, r], 0u32);
    x87!("fld dword ptr [{v}]", "fld dword ptr [{v} + 4]", "fsub dword ptr [{v}]",
        "fmul dword ptr [{v} + 8]", ".byte 0xDE, 0xC1", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld b; fsub a; fmul k; fld c; fmul m; faddp; fstp`: (b - a) k + c m.
pub(super) fn diff_times_plus_times(b: u32, a: u32, k: u32, c: u32, m: u32) -> u32 {
    let (v, mut out) = ([b, a, k, c, m], 0u32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fld dword ptr [{v} + 12]", "fmul dword ptr [{v} + 16]", ".byte 0xDE, 0xC1",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `effects::draw_particle`'s turn: the point p (st(0) x, st(1) y, st(2) z, 80 bits) by the matrix
/// at m read by columns, each (m[c] x + (m[c+4] y + m[c+8] z)) + m[c+12], as the original
/// interleaves them; the three rounded.
pub(super) fn column_turn(p: &[[u8; 10]; 3], m: u32) -> [u32; 3] {
    let mut out = [0u32; 3];
    x87!(
        "fld tbyte ptr [{p} + 20]", "fld tbyte ptr [{p} + 10]", "fld tbyte ptr [{p}]",
        "fld dword ptr [{m}]", ".byte 0xD8, 0xC9",
        "fld dword ptr [{m} + 0x10]", ".byte 0xD8, 0xCB",
        "fld dword ptr [{m} + 0x20]", ".byte 0xD8, 0xCD",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fadd dword ptr [{m} + 0x30]",
        ".byte 0xD9, 0xCB",
        "fld dword ptr [{m} + 4]", ".byte 0xD8, 0xCA",
        "fld dword ptr [{m} + 0x14]", ".byte 0xD8, 0xCC",
        "fld dword ptr [{m} + 0x24]", ".byte 0xD8, 0xCB",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fadd dword ptr [{m} + 0x34]",
        ".byte 0xD9, 0xCB",
        "fld dword ptr [{m} + 8]", ".byte 0xDE, 0xCB",
        "fld dword ptr [{m} + 0x18]", ".byte 0xDE, 0xC9",
        "fld dword ptr [{m} + 0x28]", ".byte 0xDE, 0xCA",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fadd dword ptr [{m} + 0x38]",
        ".byte 0xD9, 0xCA",
        "fstp dword ptr [{o}]", "fstp dword ptr [{o} + 4]", "fstp dword ptr [{o} + 8]";
        p = p.as_ptr(), m = m, o = out.as_mut_ptr()
    );
    out
}

/// `effects::draw_particle`'s square: q = focal / c, kept; z' = c zk; s = size q (`fild; fmul st,
/// st(1)`); Y = -(q b) + cy and X = q a + cx kept in 80 bits, the corners Y - s, Y - s,
/// Y + s, Y + s and X - s, X + s, X + s, X - s. Out: [s, z', y0 .. y3, x0 .. x3].
pub(super) fn particle_square(a: u32, b: u32, c: u32, k: [u32; 4], size: i32) -> [u32; 10] {
    // k: focal, cx, cy, zk
    let v = [a, b, c, k[0], k[1], k[2], k[3], size as u32];
    let mut out = [0u32; 10];
    x87!(
        "fld dword ptr [{v} + 12]", "fdiv dword ptr [{v} + 8]",
        "fld dword ptr [{v} + 8]", "fmul dword ptr [{v} + 24]", "fstp dword ptr [{o} + 4]",
        "fild dword ptr [{v} + 28]", ".byte 0xD8, 0xC9", "fstp dword ptr [{o}]",
        ".byte 0xD9, 0xC0", "fmul dword ptr [{v}]", ".byte 0xD9, 0xC9",
        "fmul dword ptr [{v} + 4]", "fchs", "fadd dword ptr [{v} + 20]",
        ".byte 0xD9, 0xC0", "fsub dword ptr [{o}]", "fstp dword ptr [{o} + 8]",
        ".byte 0xD9, 0xC0", "fsub dword ptr [{o}]", "fstp dword ptr [{o} + 12]",
        ".byte 0xD9, 0xC0", "fadd dword ptr [{o}]", "fstp dword ptr [{o} + 16]",
        "fadd dword ptr [{o}]", "fstp dword ptr [{o} + 20]",
        "fadd dword ptr [{v} + 16]",
        ".byte 0xD9, 0xC0", "fsub dword ptr [{o}]", "fstp dword ptr [{o} + 24]",
        ".byte 0xD9, 0xC0", "fadd dword ptr [{o}]", "fstp dword ptr [{o} + 28]",
        ".byte 0xD9, 0xC0", "fadd dword ptr [{o}]", "fstp dword ptr [{o} + 32]",
        "fsub dword ptr [{o}]", "fstp dword ptr [{o} + 36]";
        v = v.as_ptr(), o = out.as_mut_ptr()
    );
    out
}

/// `fld v; fimul t; fadd a; fstp`: a + v t.
pub(super) fn plus_times_int(a: u32, v: u32, t: i32) -> u32 {
    let (x, mut out) = ([v, t as u32, a], 0u32);
    x87!("fld dword ptr [{x}]", "fimul dword ptr [{x} + 4]", "fadd dword ptr [{x} + 8]",
        "fstp dword ptr [{o}]";
        x = x.as_ptr(), o = &raw mut out);
    out
}

/// `fld v; fadd w; fimul t; fadd a; fstp`: a + (v + w) t.
pub(super) fn plus_sum_times_int(a: u32, v: u32, w: u32, t: i32) -> u32 {
    let (x, mut out) = ([v, w, t as u32, a], 0u32);
    x87!("fld dword ptr [{x}]", "fadd dword ptr [{x} + 4]", "fimul dword ptr [{x} + 8]",
        "fadd dword ptr [{x} + 12]", "fstp dword ptr [{o}]";
        x = x.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul k; fsubr a; fstp`: a - a k.
pub(super) fn less_own_times(a: u32, k: u32) -> u32 {
    let (x, mut out) = ([a, k], 0u32);
    x87!("fld dword ptr [{x}]", "fmul dword ptr [{x} + 4]", "fsubr dword ptr [{x}]",
        "fstp dword ptr [{o}]";
        x = x.as_ptr(), o = &raw mut out);
    out
}

/// `fild k; fimul t; fiadd c; fistp`.
pub(super) fn int_times_int_plus_int(k: i32, t: i32, c: i32) -> i32 {
    let (x, mut out) = ([k as u32, t as u32, c as u32], 0i32);
    x87!("fild dword ptr [{x}]", "fimul dword ptr [{x} + 4]", "fiadd dword ptr [{x} + 8]",
        "fistp dword ptr [{o}]";
        x = x.as_ptr(), o = &raw mut out);
    out
}

/// `fild r; fmul k1; fmul k2; fmul k3; fsin` (or `fcos`) `; fmul m; fadd a; fstp`.
pub(super) fn int_times3_wave_plus(r: i32, k: [u32; 3], m: u32, a: u32, cosine: bool) -> u32 {
    let (x, mut out) = ([r as u32, k[0], k[1], k[2], m, a], 0u32);
    if cosine {
        x87!("fild dword ptr [{x}]", "fmul dword ptr [{x} + 4]", "fmul dword ptr [{x} + 8]",
            "fmul dword ptr [{x} + 12]", "fcos", "fmul dword ptr [{x} + 16]",
            "fadd dword ptr [{x} + 20]", "fstp dword ptr [{o}]";
            x = x.as_ptr(), o = &raw mut out);
    } else {
        x87!("fild dword ptr [{x}]", "fmul dword ptr [{x} + 4]", "fmul dword ptr [{x} + 8]",
            "fmul dword ptr [{x} + 12]", "fsin", "fmul dword ptr [{x} + 16]",
            "fadd dword ptr [{x} + 20]", "fstp dword ptr [{o}]";
            x = x.as_ptr(), o = &raw mut out);
    }
    out
}

/// `fild r; fmul k1; fadd a; fmul k2; fadd b; fstp`.
pub(super) fn int_times_plus_times_plus(r: i32, k1: u32, a: u32, k2: u32, b: u32) -> u32 {
    let (x, mut out) = ([r as u32, k1, a, k2, b], 0u32);
    x87!("fild dword ptr [{x}]", "fmul dword ptr [{x} + 4]", "fadd dword ptr [{x} + 8]",
        "fmul dword ptr [{x} + 12]", "fadd dword ptr [{x} + 16]", "fstp dword ptr [{o}]";
        x = x.as_ptr(), o = &raw mut out);
    out
}

/// `effects::draw_turned_sprite`'s square: q = focal / c, kept; z' = c zk; s = size q (`fild; fmul
/// st, st(1)`); the half sides (s, s) and (s, -s) turned by sin / cos (u' = v cos + u sin,
/// v' = u cos - v sin, as the original's `fld st(1)` pairs); Y = -(q b) + cy and X = q a +
/// cx kept in 80 bits, the corners Y - v1, Y - v2, Y + v1, Y + v2 and X - u1, X - u2,
/// X + u1, X + u2. Out: [s, z', u1, v1, u2, v2, y0 .. y3, x0 .. x3].
pub(super) fn turned_sprite(
    a: u32,
    b: u32,
    c: u32,
    k: [u32; 4],
    size: i32,
    sin: u32,
    cos: u32,
) -> [u32; 14] {
    // k: focal, cx, cy, zk
    let v = [a, b, c, k[0], k[1], k[2], k[3], size as u32, sin, cos];
    let mut out = [0u32; 14];
    x87!(
        "fld dword ptr [{v} + 12]", "fdiv dword ptr [{v} + 8]",
        "fld dword ptr [{v} + 8]", "fmul dword ptr [{v} + 24]", "fstp dword ptr [{o} + 4]",
        "fild dword ptr [{v} + 28]", ".byte 0xD8, 0xC9", "fst dword ptr [{o}]",
        "fst dword ptr [{o} + 8]", "fst dword ptr [{o} + 12]", "fst dword ptr [{o} + 16]",
        "fchs", "fstp dword ptr [{o} + 20]",
        // (u1, v1) = (s, s)
        "fld dword ptr [{o} + 8]", "fld dword ptr [{o} + 12]",
        ".byte 0xD9, 0xC1", ".byte 0xD9, 0xC1",
        "fmul dword ptr [{v} + 36]", ".byte 0xD9, 0xC9", "fmul dword ptr [{v} + 32]",
        ".byte 0xDE, 0xC1", "fstp dword ptr [{o} + 8]",
        "fchs", "fmul dword ptr [{v} + 32]", ".byte 0xD9, 0xC9", "fmul dword ptr [{v} + 36]",
        ".byte 0xDE, 0xC1", "fstp dword ptr [{o} + 12]",
        // (u2, v2) = (s, -s)
        "fld dword ptr [{o} + 16]", "fld dword ptr [{o} + 20]",
        ".byte 0xD9, 0xC1", ".byte 0xD9, 0xC1",
        "fmul dword ptr [{v} + 36]", ".byte 0xD9, 0xC9", "fmul dword ptr [{v} + 32]",
        ".byte 0xDE, 0xC1", "fstp dword ptr [{o} + 16]",
        "fchs", "fmul dword ptr [{v} + 32]", ".byte 0xD9, 0xC9", "fmul dword ptr [{v} + 36]",
        ".byte 0xDE, 0xC1", "fstp dword ptr [{o} + 20]",
        // Y, then X
        ".byte 0xD9, 0xC0", "fmul dword ptr [{v}]", ".byte 0xD9, 0xC9",
        "fmul dword ptr [{v} + 4]", "fchs", "fadd dword ptr [{v} + 20]",
        ".byte 0xD9, 0xC0", "fsub dword ptr [{o} + 12]", "fstp dword ptr [{o} + 24]",
        ".byte 0xD9, 0xC0", "fsub dword ptr [{o} + 20]", "fstp dword ptr [{o} + 28]",
        ".byte 0xD9, 0xC0", "fadd dword ptr [{o} + 12]", "fstp dword ptr [{o} + 32]",
        "fadd dword ptr [{o} + 20]", "fstp dword ptr [{o} + 36]",
        "fadd dword ptr [{v} + 16]",
        ".byte 0xD9, 0xC0", "fsub dword ptr [{o} + 8]", "fstp dword ptr [{o} + 40]",
        ".byte 0xD9, 0xC0", "fsub dword ptr [{o} + 16]", "fstp dword ptr [{o} + 44]",
        ".byte 0xD9, 0xC0", "fadd dword ptr [{o} + 8]", "fstp dword ptr [{o} + 48]",
        "fadd dword ptr [{o} + 16]", "fstp dword ptr [{o} + 52]";
        v = v.as_ptr(), o = out.as_mut_ptr()
    );
    out
}

/// `fld a; fadd b`, left in 80 bits.
pub(super) fn add_80(a: u32, b: u32) -> [u8; 10] {
    let (v, mut out) = ([a, b], [0u8; 10]);
    x87!("fld dword ptr [{v}]", "fadd dword ptr [{v} + 4]", "fstp tbyte ptr [{o}]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    out
}

/// `effects::draw_flare_part`'s turn of a corner (x, y) by sin / cos about (cx, cy): x' = (x cos +
/// y sin) + cx, y' = (-x sin + y cos) + cy (`fld y; fld x; fld st(1); fld st(1)` ...).
pub(super) fn turn_about(x: u32, y: u32, sin: u32, cos: u32, cx: u32, cy: u32) -> [u32; 2] {
    let v = [x, y, sin, cos, cx, cy];
    let mut out = [0u32; 2];
    x87!(
        "fld dword ptr [{v} + 4]", "fld dword ptr [{v}]",
        ".byte 0xD9, 0xC1", ".byte 0xD9, 0xC1",
        "fmul dword ptr [{v} + 12]", ".byte 0xD9, 0xC9", "fmul dword ptr [{v} + 8]",
        ".byte 0xDE, 0xC1", "fadd dword ptr [{v} + 16]", "fstp dword ptr [{o}]",
        "fchs", "fmul dword ptr [{v} + 8]", ".byte 0xD9, 0xC9", "fmul dword ptr [{v} + 12]",
        ".byte 0xDE, 0xC1", "fadd dword ptr [{v} + 20]", "fstp dword ptr [{o} + 4]";
        v = v.as_ptr(), o = out.as_mut_ptr()
    );
    out
}

/// `fld v; fmul k; fsubr c; fistp`: c - v k.
pub(super) fn times_subr_to_int(v: u32, k: u32, c: u32) -> i32 {
    let (x, mut out) = ([v, k, c], 0i32);
    x87!("fld dword ptr [{x}]", "fmul dword ptr [{x} + 4]", "fsubr dword ptr [{x} + 8]",
        "fistp dword ptr [{o}]";
        x = x.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; [fchs;] fmul b; fadd c`, left in 80 bits.
pub(super) fn times_plus_80(a: u32, b: u32, c: u32, negated: bool) -> [u8; 10] {
    let (x, mut out) = ([a, b, c], [0u8; 10]);
    if negated {
        x87!("fld dword ptr [{x}]", "fchs", "fmul dword ptr [{x} + 4]",
            "fadd dword ptr [{x} + 8]", "fstp tbyte ptr [{o}]";
            x = x.as_ptr(), o = out.as_mut_ptr());
    } else {
        x87!("fld dword ptr [{x}]", "fmul dword ptr [{x} + 4]", "fadd dword ptr [{x} + 8]",
            "fstp tbyte ptr [{o}]";
            x = x.as_ptr(), o = out.as_mut_ptr());
    }
    out
}

/// `fld k; fimul t; fiadd c; fistp`.
pub(super) fn times_int_plus_int_to_int(k: u32, t: i32, c: i32) -> i32 {
    let (x, mut out) = ([k, t as u32, c as u32], 0i32);
    x87!("fld dword ptr [{x}]", "fimul dword ptr [{x} + 4]", "fiadd dword ptr [{x} + 8]",
        "fistp dword ptr [{o}]";
        x = x.as_ptr(), o = &raw mut out);
    out
}

/// `fld v; fmul k; fimul t; fsubr v; fstp`: v - v k t.
pub(super) fn damp(v: u32, k: u32, t: i32) -> u32 {
    let (x, mut out) = ([v, k, t as u32], 0u32);
    x87!("fld dword ptr [{x}]", "fmul dword ptr [{x} + 4]", "fimul dword ptr [{x} + 8]",
        "fsubr dword ptr [{x}]", "fstp dword ptr [{o}]";
        x = x.as_ptr(), o = &raw mut out);
    out
}

/// `fild i; fsub v; fmul k; fimul t; fadd v; fist; fstp`: v + (i - v) k t, as a whole
/// number and as a float.
pub(super) fn approach(i: i32, v: u32, k: u32, t: i32) -> (i32, u32) {
    let (x, mut whole, mut out) = ([i as u32, v, k, t as u32], 0i32, 0u32);
    x87!("fild dword ptr [{x}]", "fsub dword ptr [{x} + 4]", "fmul dword ptr [{x} + 8]",
        "fimul dword ptr [{x} + 12]", "fadd dword ptr [{x} + 4]", "fist dword ptr [{w}]",
        "fstp dword ptr [{o}]";
        x = x.as_ptr(), w = &raw mut whole, o = &raw mut out);
    (whole, out)
}

/// `fld v; fimul t; fsubr a; fstp`: a - v t.
pub(super) fn less_times_int(a: u32, v: u32, t: i32) -> u32 {
    let (x, mut out) = ([v, t as u32, a], 0u32);
    x87!("fld dword ptr [{x}]", "fimul dword ptr [{x} + 4]", "fsubr dword ptr [{x} + 8]",
        "fstp dword ptr [{o}]";
        x = x.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fld b; fmul k; faddp; fistp`: a + b k as a whole number.
pub(super) fn plus_product_to_int(a: u32, b: u32, k: u32) -> i32 {
    let (x, mut out) = ([a, b, k], 0i32);
    x87!("fld dword ptr [{x}]", "fld dword ptr [{x} + 4]", "fmul dword ptr [{x} + 8]",
        ".byte 0xDE, 0xC1", "fistp dword ptr [{o}]";
        x = x.as_ptr(), o = &raw mut out);
    out
}

/// `fild i; fmul a; fmul b; fistp`.
pub(super) fn int_times_times_to_int(i: i32, a: u32, b: u32) -> i32 {
    let (x, mut out) = ([i as u32, a, b], 0i32);
    x87!("fild dword ptr [{x}]", "fmul dword ptr [{x} + 4]", "fmul dword ptr [{x} + 8]",
        "fistp dword ptr [{o}]";
        x = x.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul k; fadd c; fchs`, left in 80 bits.
pub(super) fn times_plus_negated_80(a: u32, k: u32, c: u32) -> [u8; 10] {
    let (x, mut out) = ([a, k, c], [0u8; 10]);
    x87!("fld dword ptr [{x}]", "fmul dword ptr [{x} + 4]", "fadd dword ptr [{x} + 8]", "fchs",
        "fstp tbyte ptr [{o}]";
        x = x.as_ptr(), o = out.as_mut_ptr());
    out
}

/// `fld a; fsub b; fmul k; fadd c; fchs`, left in 80 bits.
pub(super) fn less_times_plus_negated_80(a: u32, b: u32, k: u32, c: u32) -> [u8; 10] {
    let (x, mut out) = ([a, b, k, c], [0u8; 10]);
    x87!("fld dword ptr [{x}]", "fsub dword ptr [{x} + 4]", "fmul dword ptr [{x} + 8]",
        "fadd dword ptr [{x} + 12]", "fchs", "fstp tbyte ptr [{o}]";
        x = x.as_ptr(), o = out.as_mut_ptr());
    out
}

/// `fld v; fist i; fisub i; fmul k; fistp`: v's fraction (as the rounding is set) times k.
pub(super) fn fraction_times_to_int(v: u32, k: u32) -> i32 {
    let (x, mut i, mut out) = ([v, k], 0i32, 0i32);
    x87!("fld dword ptr [{x}]", "fist dword ptr [{i}]", "fisub dword ptr [{i}]",
        "fmul dword ptr [{x} + 4]", "fistp dword ptr [{o}]";
        x = x.as_ptr(), i = &raw mut i, o = &raw mut out);
    out
}

/// `fild a; fidiv b; fistp`.
pub(super) fn int_over_int_to_int(a: i32, b: i32) -> i32 {
    let (v, mut out) = ([a as u32, b as u32], 0i32);
    x87!("fild dword ptr [{v}]", "fidiv dword ptr [{v} + 4]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fadd b; fadd c; fmul k; fstp`.
pub(super) fn sum3_times(a: u32, b: u32, c: u32, k: u32) -> u32 {
    let (x, mut out) = ([a, b, c, k], 0u32);
    x87!("fld dword ptr [{x}]", "fadd dword ptr [{x} + 4]", "fadd dword ptr [{x} + 8]",
        "fmul dword ptr [{x} + 12]", "fstp dword ptr [{o}]";
        x = x.as_ptr(), o = &raw mut out);
    out
}

// ---- the differential tests (plain `x87.rs` against the assembly above) ----

#[cfg(test)]
mod diff {
    use crate::math::cells::At;
    use crate::math::x87 as plain;
    use crate::math::x87::{control_word, set_control_word};
    use std::collections::BTreeMap;

    /// Cases per function and control word.
    const CASES: u32 = 20_000;

    #[derive(Clone)]
    struct Rng(u64);

    const EDGES: [f32; 40] = [
        0.0,
        -0.0,
        1.0,
        -1.0,
        0.5,
        -0.5,
        1.5,
        2.5,
        -1.5,
        -2.5,
        0.25,
        3.5,
        1e-45,
        1.1754942e-38,
        -1e-45,
        3.4028235e38,
        -3.4028235e38,
        1e30,
        1e-30,
        2147483520.0,
        2147483648.0,
        -2147483648.0,
        -2147483520.0,
        4294967296.0,
        16777216.0,
        16777217.0,
        -16777216.0,
        8388607.5,
        std::f32::consts::PI,
        std::f32::consts::FRAC_PI_2,
        2.0 * std::f32::consts::PI,
        -std::f32::consts::PI,
        0.1,
        100.0,
        1000.0,
        65536.0,
        0.001,
        255.0,
        -0.001,
        7.0,
    ];

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        }
        fn below(&mut self, n: u64) -> u64 {
            self.next() % n
        }
        /// A float's bits: edge values, random finite patterns, integers, halves,
        /// moderate values, values near the i32 range, denormals.
        fn fl(&mut self) -> u32 {
            match self.below(14) {
                0 | 1 => EDGES[self.below(EDGES.len() as u64) as usize].to_bits(),
                2 => {
                    let b = self.next() as u32;
                    if (b >> 23) & 0xFF == 0xFF {
                        b & !(1 << 30)
                    } else {
                        b
                    }
                }
                3 => (self.int_small() as f32).to_bits(),
                4 => (self.int_small() as f32 + 0.5).to_bits(),
                5 => (self.int_small() as f32 * 0.5 + 0.25).to_bits(),
                6..=9 => self.fm(),
                10 => {
                    let base = if self.below(2) == 0 {
                        2147483648.0f32
                    } else {
                        -2147483648.0
                    };
                    (base + (self.int_small() % 300) as f32 * 128.0).to_bits()
                }
                11 => (self.next() as u32) & 0x807F_FFFF,
                12 => (self.int_small() as f32 / 1024.0).to_bits(),
                _ => {
                    (self.next() as u32 & 0x8000_0000)
                        | (0x2000_0000 + (self.next() as u32 & 0x1FFF_FFFF) % 0x2000_0000)
                }
            }
        }
        /// A moderate float (|x| below a few thousand), often a whole or half number.
        fn fm(&mut self) -> u32 {
            let v = (self.below(4_000_000) as f64 / 1000.0 - 2000.0) as f32;
            match self.below(5) {
                0 => v.round().to_bits(),
                1 => (v.round() + 0.5).to_bits(),
                2 => (v / 64.0).to_bits(),
                _ => v.to_bits(),
            }
        }
        /// An angle for `sin` / `cos`: up to a few thousand radians, often near a multiple
        /// of a quarter turn.
        fn ft(&mut self) -> u32 {
            match self.below(4) {
                0 => {
                    let k = (self.below(2001) as i32 - 1000) as f32;
                    let near = (k * std::f32::consts::FRAC_PI_2).to_bits();
                    near.wrapping_add(self.below(7) as u32).wrapping_sub(3)
                }
                1 => ((self.below(20_000_000) as f64 / 1000.0 - 10_000.0) as f32).to_bits(),
                2 => ((self.below(2_000_000) as f64 / 100_000.0 - 10.0) as f32).to_bits(),
                _ => {
                    EDGES[self.below(EDGES.len() as u64) as usize].to_bits() & !0x4000_0000
                        | 0x3000_0000
                }
            }
        }
        fn int_small(&mut self) -> i32 {
            match self.below(3) {
                0 => self.below(201) as i32 - 100,
                1 => self.below(200_001) as i32 - 100_000,
                _ => self.below(33_554_433) as i32 - 16_777_216,
            }
        }
        /// An integer: random, small, or at an edge.
        fn ii(&mut self) -> i32 {
            match self.below(8) {
                0 => [
                    0,
                    1,
                    -1,
                    i32::MAX,
                    i32::MIN,
                    1 << 24,
                    (1 << 24) + 1,
                    -(1 << 30),
                    1 << 30,
                    46341,
                ][self.below(10) as usize],
                1 | 2 => self.next() as i32,
                _ => self.int_small(),
            }
        }
        /// A small integer (a tick count, a size).
        fn is(&mut self) -> i32 {
            self.int_small() % 5000
        }
        fn bl(&mut self) -> bool {
            self.below(2) == 0
        }
        fn ofl(&mut self) -> Option<u32> {
            if self.bl() { Some(self.fl()) } else { None }
        }
        /// A double's bits (a constant of the original).
        fn d64(&mut self) -> u64 {
            match self.below(3) {
                0 => (self.below(2_000_000) as f64 / 1000.0 - 1000.0).to_bits(),
                1 => (self.below(2_000_000) as f64 / 1e7 - 0.1).to_bits(),
                _ => {
                    // a random double, its exponent kept within 2^-100 .. 2^100 (beyond
                    // that the products leave the double range, which the x87's stack does not)
                    let b = self.next();
                    let e = 1023 - 100 + (b >> 52) % 201;
                    (b & !(0x7FF << 52)) | (e << 52)
                }
            }
        }
        /// A divisor for `fprem` whose exponent is less than 64 below an integer's (the
        /// x87 reduces only partly beyond that).
        fn fd(&mut self) -> u32 {
            loop {
                let b = self.fl();
                let x = f32::from_bits(b).abs();
                if x == 0.0 || x >= 2f32.powi(-33) {
                    return b;
                }
            }
        }
        /// A small float, as the multiplier of a wave's argument.
        fn fk(&mut self) -> u32 {
            ((self.below(2000) as f64 / 100.0 - 10.0) as f32).to_bits()
        }
        fn k3(&mut self) -> [u32; 3] {
            [self.fk(), self.fk(), self.fk()]
        }
        fn a3(&mut self) -> [u32; 3] {
            [self.fl(), self.fl(), self.fl()]
        }
        fn m3(&mut self) -> [u32; 3] {
            [self.fm(), self.fm(), self.fm()]
        }
        fn a7(&mut self) -> [u32; 7] {
            [
                self.fl(),
                self.fl(),
                self.fl(),
                self.fl(),
                self.fl(),
                self.fl(),
                self.fl(),
            ]
        }
        fn a33(&mut self) -> [[u32; 3]; 3] {
            [self.a3(), self.a3(), self.a3()]
        }
        fn a32(&mut self) -> [[u32; 2]; 3] {
            [
                [self.fl(), self.fl()],
                [self.fl(), self.fl()],
                [self.fl(), self.fl()],
            ]
        }
        fn k4(&mut self) -> [u32; 4] {
            [self.fl(), self.fl(), self.fl(), self.fl()]
        }
        /// 80-bit values as the x87 holds them at 53-bit precision (what a `fstp tbyte`
        /// of this module leaves).
        fn e80(&mut self) -> [u8; 10] {
            let f = f64::from_bits(match self.below(3) {
                0 => f64::from(f32::from_bits(self.fl())).to_bits(),
                1 => (f64::from(f32::from_bits(self.fm())) * f64::from(f32::from_bits(self.fm())))
                    .to_bits(),
                _ => (f64::from(f32::from_bits(self.fl()))
                    + f64::from(f32::from_bits(self.fm())) * 1e-9)
                    .to_bits(),
            });
            to_f80(f)
        }
        fn p80(&mut self) -> [[u8; 10]; 3] {
            [self.e80(), self.e80(), self.e80()]
        }
    }

    /// `fld qword; fstp tbyte`: a double as the 80-bit format, by the hardware.
    fn to_f80(x: f64) -> [u8; 10] {
        let mut out = [0u8; 10];
        unsafe {
            core::arch::asm!(
                "fld qword ptr [{x}]", "fstp tbyte ptr [{o}]",
                x = in(reg) &raw const x, o = in(reg) out.as_mut_ptr(),
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        };
        out
    }

    trait Same {
        fn same(&self, other: &Self, ulps: u32) -> bool;
    }
    impl Same for u32 {
        /// Float bits: equal, both NaN, or within `ulps` of the same sign.
        fn same(&self, o: &Self, ulps: u32) -> bool {
            if self == o {
                return true;
            }
            let (a, b) = (f32::from_bits(*self), f32::from_bits(*o));
            if a.is_nan() && b.is_nan() {
                return true;
            }
            if ulps == 0 {
                return false;
            }
            if a == 0.0 && b == 0.0 {
                return true;
            }
            (self >> 31 == o >> 31) && (self & 0x7FFF_FFFF).abs_diff(o & 0x7FFF_FFFF) <= ulps
        }
    }
    impl Same for i32 {
        fn same(&self, o: &Self, _: u32) -> bool {
            self == o
        }
    }
    impl Same for u16 {
        /// A status word: C3, C2 and C0.
        fn same(&self, o: &Self, _: u32) -> bool {
            (self ^ o) & 0x4500 == 0
        }
    }
    impl Same for [u8; 10] {
        fn same(&self, o: &Self, _: u32) -> bool {
            let nan = |b: &[u8; 10]| {
                u16::from_le_bytes([b[8], b[9]]) & 0x7FFF == 0x7FFF
                    && u64::from_le_bytes(b[..8].try_into().unwrap()) << 1 != 0
            };
            self == o || (nan(self) && nan(o))
        }
    }
    impl<T: Same, const N: usize> Same for [T; N] {
        fn same(&self, o: &Self, u: u32) -> bool {
            self.iter().zip(o).all(|(a, b)| a.same(b, u))
        }
    }
    impl<A: Same, B: Same> Same for (A, B) {
        fn same(&self, o: &Self, u: u32) -> bool {
            self.0.same(&o.0, u) && self.1.same(&o.1, u)
        }
    }

    struct T {
        rng: Rng,
        fails: BTreeMap<&'static str, (u32, Vec<String>)>,
        calls: BTreeMap<&'static str, u32>,
    }

    impl T {
        fn new(seed: u64) -> T {
            T {
                rng: Rng(seed),
                fails: BTreeMap::new(),
                calls: BTreeMap::new(),
            }
        }
        fn record(&mut self, name: &'static str, ok: bool, detail: impl FnOnce() -> String) {
            *self.calls.entry(name).or_default() += 1;
            if !ok {
                let e = self.fails.entry(name).or_default();
                e.0 += 1;
                if e.1.len() < 4 {
                    e.1.push(detail());
                }
            }
        }
        fn report(&self, what: &str) -> String {
            let mut s = String::new();
            for (name, (n, ex)) in &self.fails {
                s += &format!("{what}: {name}: {n} of {} differ, e.g.\n", self.calls[name]);
                for e in ex {
                    s += &format!("    {e}\n");
                }
            }
            s
        }
    }

    /// Runs reference and plain on the same generated arguments.
    macro_rules! diff {
        ($t:expr, $name:ident ( $($g:ident),* )) => { diff!($t, $name($($g),*) tol 0) };
        ($t:expr, $name:ident ( $($g:ident),* ) tol $ulps:expr) => {{
            for _ in 0..CASES {
                let mut r1 = $t.rng.clone();
                let want = super::$name($(r1.$g()),*);
                let mut r2 = $t.rng.clone();
                let got = crate::math::x87_reference_a::AsBits::into_bits(plain::$name($(r2.$g()),*));
                let mut r3 = $t.rng.clone();
                let shown = ($(r3.$g(),)*);
                $t.rng = r1;
                let ok = want.same(&got, $ulps);
                $t.record(stringify!($name), ok, || format!("{shown:x?} want {want:x?} got {got:x?}"));
            }
        }};
    }

    /// The control word at `w` (real and emulated) for the closure, 0x027F after.
    fn with_control(w: u16, f: impl FnOnce()) {
        struct Restore;
        impl Drop for Restore {
            fn drop(&mut self) {
                set_control_word(0x027F);
            }
        }
        let _restore = Restore;
        set_control_word(w);
        assert_eq!(control_word(), w);
        f();
    }

    fn matrix(r: &mut Rng) -> [u32; 16] {
        let mut m = [0u32; 16];
        for x in &mut m {
            *x = if r.below(4) == 0 { r.fl() } else { r.fm() };
        }
        m
    }

    fn pointer_functions(t: &mut T) {
        for _ in 0..CASES {
            let mut r = [0u32; 12];
            for x in &mut r {
                *x = t.rng.fm();
            }
            let (a, b) = (t.rng.fl(), t.rng.fl());
            let (mut want, mut got) = (r, r);
            super::scale_record(want.as_mut_ptr(), a, b);
            got = plain::scale_record_bits(got, a, b);
            let ok = want.same(&got, 0);
            t.record("scale_record", ok, || {
                format!("{r:x?} {a:x} {b:x} want {want:x?} got {got:x?}")
            });

            let m = matrix(&mut t.rng);
            let mp = crate::pointers::game_address(m.as_ptr());
            let (p, k, d, lift) = (t.rng.a3(), t.rng.fl(), t.rng.fl(), t.rng.fl());
            let want = super::turned_place_less(p, k, d, lift, mp);
            let got = plain::turned_place_less(p, k, d, lift, &At(mp));
            let ok = want.same(&got, 0);
            t.record("turned_place_less", ok, || {
                format!("{p:x?} {k:x} {d:x} {lift:x} {m:x?} want {want:x?} got {got:x?}")
            });

            let want = super::turned_place(p, lift, mp);
            let got = plain::turned_place(p, lift, &At(mp));
            let ok = want.same(&got, 0);
            t.record("turned_place", ok, || {
                format!("{p:x?} {lift:x} {m:x?} want {want:x?} got {got:x?}")
            });

            let w = t.rng.a3();
            let wp = crate::pointers::game_address(w.as_ptr());
            let want = super::crossed_turned(p, wp, mp);
            let got = plain::crossed_turned(p, &At(wp), &At(mp));
            let ok = want.same(&got, 0);
            t.record("crossed_turned", ok, || {
                format!("{p:x?} {w:x?} {m:x?} want {want:x?} got {got:x?}")
            });

            let p80 = t.rng.p80();
            let want = super::column_turn(&p80, mp);
            let got = plain::column_turn(&p80, &At(mp)).map(f32::to_bits);
            let ok = want.same(&got, 0);
            t.record("column_turn", ok, || {
                format!("{p80:x?} {m:x?} want {want:x?} got {got:x?}")
            });
        }
    }

    fn all_functions(t: &mut T) {
        diff!(t, sub_compare(fl, fl, bl, fl));
        diff!(t, random_step(ii, a7));
        diff!(t, int_plus_less_squared(ii, fl, fl));
        diff!(t, int_times_plus_on(ii, fl, fl, fl, fl));
        diff!(t, sub_thrice(fl, fl));
        diff!(t, int_plus(ii, fl));
        diff!(t, ease(fl, fl, fl));
        diff!(t, int_plus_to_int(ii, fl));
        diff!(t, zero_less_int_plus_int(ii, ii));
        diff!(t, less_int_plus_int(fl, ii, ii));
        diff!(t, two_pi_less(fl));
        diff!(t, two_pi_less(ft));
        diff!(t, int_times_plus_times(ii, fl, fl, fl));
        diff!(t, over_int(fl, ii));
        diff!(t, zero_times_plus_times(fl, fl, fl, fl, bl));
        diff!(t, prem_plus(ii, fd, fl));
        diff!(t, grown(ii, d64, fl, fl));
        diff!(t, rate_to_int(fl, fl, fl, fl, fl, fl, fl));
        diff!(t, pair_sum_80(ii, fl, ii, fl, fl));
        diff!(t, ext_less(e80, fl));
        diff!(t, three_products(a33));
        diff!(t, times_kept_abs_to_int(fl, fl, fl));
        diff!(t, blend_kept_abs_to_int(fl, fl, fl, fl, fl));
        diff!(t, times_less_abs_times(fl, fl, ofl, fl));
        diff!(t, less_plus_int_times(fl, fl, fl, ii));
        diff!(t, times_int_plus_times_80(fl, fl, ii, ofl, fl));
        diff!(t, pair_sum_less(ii, fl, ii, fl, fl, fl));
        diff!(t, product_plus(fl, fl, fl, fl));
        diff!(t, float_times_int_to_int(fl, ii));
        diff!(t, speed_rate_to_int(fl, fl, fl, fl, fl, fl));
        diff!(t, abs_sum_to_int_kept(fl, fl, fl, fl));
        diff!(t, abs_blend_compare(fl, fl, fl, fl, fl, fl));
        diff!(t, times_plus_to_int(fl, fl, fl));
        diff!(t, plus_int_times_80(fl, fl, ii));
        diff!(t, three_products_80(a33));
        diff!(t, times_compare(fl, fl, fl));
        diff!(t, times_int_to_int(fl, fl, ii));
        diff!(t, int_times_plus_and_negated(ii, fl, fl));
        diff!(t, int_times_less_less(ii, fl, fl, fl));
        diff!(t, abs_sum_times_to_int(fl, fl, fl));
        diff!(t, drifted(ii, fl, a3, ii, a3));
        diff!(t, one_less_inverse(fl, fl, fl));
        diff!(t, times_over_int(fl, fl, ii));
        diff!(t, mul_80(fl, fl));
        diff!(t, int_times_int_to_int(ii, ii));
        diff!(t, sum4_times(fl, fl, fl, fl, fl));
        diff!(t, length_and_rate(a3, fl, fl, fl, fl));
        diff!(t, length_and_rate(m3, fl, fl, fl, fl));
        diff!(t, normalised3(a3, fl));
        diff!(t, normalised3(m3, fl));
        diff!(t, unit_and_length(a3));
        diff!(t, unit_and_length(m3));
        diff!(t, negated_less_plus_plus(fl, fl, fl, fl));
        diff!(t, sine_and_cosine(ft) tol 1);
        diff!(t, turn_yz(fl, fl, fl, fl));
        diff!(t, int_times_plus_int_to_int(ii, fl, ii));
        diff!(t, blend3_80(ii, ii, ii, a3));
        diff!(t, angle_and_wave(fm, is, fm, fm, fm) tol 1);
        diff!(t, angle_and_wave(ft, is, fm, fm, fm) tol 1);
        diff!(t, int_lerp_to_int(ii, ii, fl));
        diff!(t, int_over_int_times(ii, ii, fl));
        diff!(t, clip_lerp3(fl, fl, fl, a32));
        diff!(t, slot_project(fl, fl, fl, fl, fl, fl, fl));
        diff!(t, int_times_times(ii, fl, fl));
        diff!(t, toward(fl, fl, fl));
        diff!(t, diff_times_plus_times(fl, fl, fl, fl, fl));
        diff!(t, particle_square(fl, fl, fl, k4, is));
        diff!(t, plus_times_int(fl, fl, ii));
        diff!(t, plus_sum_times_int(fl, fl, fl, ii));
        diff!(t, less_own_times(fl, fl));
        diff!(t, int_times_int_plus_int(ii, ii, ii));
        diff!(t, int_times3_wave_plus(is, k3, fm, fm, bl) tol 1);
        diff!(t, int_times3_wave_plus(is, k3, fl, fm, bl) tol 1);
        diff!(t, int_times_plus_times_plus(ii, fl, fl, fl, fl));
        diff!(t, turned_sprite(fl, fl, fl, k4, is, fl, fl));
        diff!(t, add_80(fl, fl));
        diff!(t, turn_about(fl, fl, fl, fl, fl, fl));
        diff!(t, times_subr_to_int(fl, fl, fl));
        diff!(t, times_plus_80(fl, fl, fl, bl));
        diff!(t, times_int_plus_int_to_int(fl, ii, ii));
        diff!(t, damp(fl, fl, ii));
        diff!(t, approach(ii, fl, fl, ii));
        diff!(t, less_times_int(fl, fl, ii));
        diff!(t, plus_product_to_int(fl, fl, fl));
        diff!(t, int_times_times_to_int(ii, fl, fl));
        diff!(t, times_plus_negated_80(fl, fl, fl));
        diff!(t, less_times_plus_negated_80(fl, fl, fl, fl));
        diff!(t, fraction_times_to_int(fl, fl));
        diff!(t, int_over_int_to_int(ii, ii));
        diff!(t, sum3_times(fl, fl, fl, fl));
        // moderate values too, where nothing saturates
        diff!(t, ease(fm, fm, fm));
        diff!(t, toward(fm, fm, fm));
        diff!(t, rate_to_int(fm, fm, fm, fm, fm, fm, fm));
        diff!(t, speed_rate_to_int(fm, fm, fm, fm, fm, fm));
        diff!(t, three_products(a33));
        diff!(t, turn_about(fm, fm, fm, fm, fm, fm));
        diff!(t, clip_lerp3(fm, fm, fm, a32));
        diff!(t, particle_square(fm, fm, fm, k4, is));
        diff!(t, turned_sprite(fm, fm, fm, k4, is, fm, fm));
        diff!(t, fraction_times_to_int(fm, fm));
        diff!(t, times_plus_to_int(fm, fm, fm));
        diff!(t, int_lerp_to_int(is, is, fm));
        diff!(t, prem_plus(ii, fm, fm));
        diff!(t, prem_plus(is, fd, fm));
        diff!(t, slot_project(fm, fm, fm, fm, fm, fm, fm));
        diff!(t, drifted(is, fm, m3, is, m3));
        diff!(t, approach(is, fm, fm, is));
        diff!(t, grown(is, d64, fm, fm));
    }

    fn run(w: u16, seed: u64) {
        let mut t = T::new(seed);
        with_control(w, || {
            all_functions(&mut t);
            pointer_functions(&mut t);
        });
        let report = t.report(&format!("control word {w:#06x}"));
        assert!(report.is_empty(), "\n{report}");
    }

    #[test]
    fn plain_equals_x87_at_double_precision() {
        run(0x027F, 0x5EED_0B01);
    }

    #[test]
    fn plain_equals_x87_at_chop() {
        run(0x0E7F, 0x5EED_0B02);
    }

    #[test]
    fn plain_equals_x87_rounding_down() {
        run(0x067F, 0x5EED_0B03);
    }

    #[test]
    fn plain_equals_x87_rounding_up() {
        run(0x0A7F, 0x5EED_0B04);
    }
}
