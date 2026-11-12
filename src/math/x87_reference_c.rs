//! The x87 assembly versions of the helpers in `math/x87/run.rs`, `math/x87/load.rs`, `math/normalise_vector.rs`, kept as the reference the
//! plain Rust versions are tested against (phase 6, 2026-10-24).
//!
//! The assembly is verbatim (modules `load`, `run`, `norm`); the tests below run every
//! function both ways on seeded inputs with the real and the emulated control word at
//! 0x027F (and, for the functions that round to a whole number, also at chop, down and up)
//! and compare bit for bit (any NaN equals any NaN); the transcendental ones within an
//! ulp of the float result (2 ulp of a double).

#![allow(clippy::all, dead_code)]

/// `math/x87/load.rs`'s assembly versions.
pub(super) mod load {
    use crate::math::x87::x87;

    /// `fild a; fimul b; fdiv c; fistp`: a whole number times a whole number over a float,
    /// rounded once at the end as the x87 is set to round.
    pub(super) fn int_times_int_over_to_int(a: i32, b: i32, c: u32) -> i32 {
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

    /// `fld a; fabs; fistp`: the size of a float as a whole number.
    pub(super) fn abs_to_int(a: u32) -> i32 {
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
    pub(super) fn add_to_int(a: u32, b: u32) -> i32 {
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
    pub(super) fn float_hypot(a: u32, b: u32) -> u32 {
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
    pub(super) fn below_or_unordered(a: u32, b: u32) -> bool {
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
    pub(super) fn scaled_abs_less_below(a: u32, h: u32, k: u32, b: u32) -> bool {
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
    /// into `out[2]` from the x87's precision (0x4573FA).
    pub(super) fn rotate_rounding_z(point: *const i32, matrix: *const u32, out: *mut u32) {
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
    pub(super) fn int_square_sum(a: i32, b: i32) -> u32 {
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
    pub(super) fn float_square_sum(a: u32, b: u32) -> u32 {
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
    pub(super) fn product3(a: u32, b: u32, c: u32) -> u32 {
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
    pub(super) fn int_over_float(a: i32, b: u32) -> u32 {
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
    pub(super) fn int_plus_times(a: i32, b: u32, c: u32) -> u32 {
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
    pub(super) fn abs_below_or_unordered(a: u32, k: u32) -> bool {
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
    pub(super) fn rotate_rounding(point: *const i32, matrix: *const u32, out: *mut u32) {
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
    pub(super) fn int_square_sum_scaled_to_int(a: i32, b: i32, k: u32, times: u32) -> u32 {
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
    /// camera's roll eased towards a target (`race::view_matrices`), rounded once.
    pub(super) fn eased_roll(i: i32, a: u32, b: u32, v: u32, c: u32, m: u32, h: u32) -> u32 {
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
    pub(super) fn product4_to_int(a: u32, b: u32, c: u32, d: u32) -> u32 {
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

    /// The sun's direction in `race::draw_sky`: `fild a; fchs; fadd k; fmul s; fsincos; fild b;
    /// fmul s; fsincos; fmul st(2), st; fmulp st(3), st; fxch st(1)`, then each of the three
    /// times `m` and stored: cos a cos b, sin b, sin a cos b (each x `m`).
    pub(super) fn sun_direction(a: i32, k: u32, s: u32, b: i32, m: u32) -> [u32; 3] {
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
    pub(super) fn to_tbyte(a: u32) -> [u8; 10] {
        let (v, mut out) = ([a], [0u8; 10]);
        x87!("fld dword ptr [{v}]", "fstp tbyte ptr [{o}]"; v = v.as_ptr(), o = out.as_mut_ptr());
        out
    }

    /// 80 bits rounded to a whole number as the x87 is set to round (`fld tbyte; fistp`).
    pub(super) fn tbyte_to_int(a: [u8; 10]) -> u32 {
        let mut out = 0u32;
        x87!("fld tbyte ptr [{v}]", "fistp dword ptr [{o}]"; v = a.as_ptr(), o = &raw mut out);
        out
    }

    /// A point of whole numbers onto the screen as in `race::draw_sky` (the original's
    /// instructions; `.byte 0xDE, 0xF1` its `fdivrp st(1), st`): `v` = x, y, z, the flat
    /// depth, the scale, the centre's y and x. Flat: z dropped (into `scratch`) for the flat
    /// depth + 1. Gives x as a float and rounded, y as a float and rounded.
    pub(super) fn sun_on_screen(v: &[u32; 7], flat: bool, scratch: *mut u32) -> [u32; 4] {
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
    pub(super) fn sun_size(x: u32, cx: u32, y: u32, cy: u32, h: u32, k: u32) -> u32 {
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
}

/// `math/x87/run.rs`'s assembly versions.
pub(super) mod run {
    use crate::math::x87::x87;

    /// `fld v[0]; fmul st, st; fld v[1]; fmul st, st; fld v[2]; fmul st, st; faddp; faddp;
    /// fsqrt`: the length of three floats as an 80-bit value (not rounded to a float).
    pub(super) fn length_80(v: *const u32) -> [u8; 10] {
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
    pub(super) fn scale_by_80(v: *mut u32, s: [u8; 10]) {
        x87!("fld tbyte ptr [{s}]",
        "fld dword ptr [{v}]", ".byte 0xD8, 0xC9", "fstp dword ptr [{v}]",
        "fld dword ptr [{v} + 4]", ".byte 0xD8, 0xC9", "fstp dword ptr [{v} + 4]",
        "fld dword ptr [{v} + 8]", ".byte 0xD8, 0xC9", "fstp dword ptr [{v} + 8]",
        "fstp st(0)";
        v = v, s = s.as_ptr());
    }

    /// `fld a; fadd b; fmul h; fdiv d; fstp`.
    pub(super) fn sum_times_over(a: u32, b: u32, h: u32, d: u32) -> u32 {
        let (v, mut out) = ([a, b, h, d], 0u32);
        x87!("fld dword ptr [{v}]", "fadd dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fdiv dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fild a; fild c; fsubp; fmul f; fild b; fild c; fsubp; fmul g; fild c; faddp; faddp;
    /// fistp`: c + (a - c) f + (b - c) g as a whole number (the sums in that order).
    pub(super) fn plane_height(a: i32, b: i32, c: i32, f: u32, g: u32) -> i32 {
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
    /// `Numbers.k65536` (bits) at [6]. `o` gets f764, f75C, f760, f810, i800, f814, i804, and with
    /// `sse` f768 (`rcpss` of f764) at [7].
    pub(super) unsafe fn face_shares(i: &[u32; 7], o: &mut [u32; 8], sse: bool) {
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
    pub(super) fn less_plus_less_product_80(
        a: u32,
        b: u32,
        c: Option<u32>,
        d: u32,
        e: u32,
    ) -> [u8; 10] {
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
    pub(super) fn int_times_times_over_to_int(i: i32, a: u32, b: u32, c: u32) -> i32 {
        let (v, mut out) = ([i as u32, a, b, c], 0i32);
        x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fdiv dword ptr [{v} + 12]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld a; fmul st, st; fmul k; fadd c; fmul g; fst; fchs; fstp`: (a a k + c) g and its
    /// negative, each rounded to a float.
    pub(super) fn square_times_plus_times(a: u32, k: u32, c: u32, g: u32) -> (u32, u32) {
        let (v, mut out) = ([a, k, c, g], [0u32; 2]);
        x87!("fld dword ptr [{v}]", ".byte 0xD8, 0xC8", "fmul dword ptr [{v} + 4]",
        "fadd dword ptr [{v} + 8]", "fmul dword ptr [{v} + 12]", "fst dword ptr [{o}]",
        "fchs", "fstp dword ptr [{o} + 4]";
        v = v.as_ptr(), o = out.as_mut_ptr());
        (out[0], out[1])
    }

    /// `fld1; fchs; fstp`: -1.0 as the x87 stores it.
    pub(super) fn minus_one() -> u32 {
        let mut out = 0u32;
        x87!("fld1", "fchs", "fstp dword ptr [{o}]"; o = &raw mut out);
        out
    }

    /// `fld a; fcomp b; fnstsw`: the whole status word's C0 .. C3 (0x4700).
    pub(super) fn compare_full(a: u32, b: u32) -> u16 {
        let (v, mut status) = ([a, b], 0u16);
        x87!("fld dword ptr [{v}]", "fcomp dword ptr [{v} + 4]", "fnstsw word ptr [{s}]";
        v = v.as_ptr(), s = &raw mut status);
        status & 0x4700
    }

    /// `fld1`, then for each (a, b, c) `fld a; fmul b; fmul c; fsubp st(1), st`: 1 less the
    /// products, unrounded (the 80 bits carried between the terms through memory, exactly).
    pub(super) fn one_less_products(terms: &[[u32; 3]]) -> [u8; 10] {
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
    pub(super) fn ext_times(e: [u8; 10], k: u32) -> [u8; 10] {
        let mut out = [0u8; 10];
        x87!("fld tbyte ptr [{e}]", "fmul dword ptr [{k}]", "fstp tbyte ptr [{o}]";
        e = e.as_ptr(), k = &raw const k, o = out.as_mut_ptr());
        out
    }

    /// A ramp through (t0, t1) .. (t2, t3) as the original reads it: below t0 (or no number)
    /// t1; not below t2, t3; t2 - t0 equal to `zero` (or no number), t1; else
    /// `fld x; fsub t0; fld t2; fsub t0; fdivp; fld t3; fsub t1; fmulp; fadd t1`. Rounded to
    /// a float (the `fstp`).
    pub(super) fn ramp(x: u32, t: [u32; 4], zero: u32) -> u32 {
        if compare_full(x, t[0]) & 0x100 != 0 {
            return crate::math::x87::loaded(t[1]).to_bits();
        }
        if compare_full(x, t[2]) & 0x100 == 0 {
            return crate::math::x87::loaded(t[3]).to_bits();
        }
        let mut status = 0u16;
        x87!("fld dword ptr [{t} + 8]", "fsub dword ptr [{t}]", "fcomp dword ptr [{z}]",
        "fnstsw word ptr [{s}]";
        t = t.as_ptr(), z = &raw const zero, s = &raw mut status);
        if status & 0x4000 != 0 {
            return crate::math::x87::loaded(t[1]).to_bits();
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
    pub(super) fn mul3(a: u32, b: u32, c: u32, d: u32) -> u32 {
        let (v, mut out) = ([a, b, c, d], 0u32);
        x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fmul dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld a; fmul st, st; fstp`.
    pub(super) fn square(a: u32) -> u32 {
        let mut out = 0u32;
        x87!("fld dword ptr [{a}]", ".byte 0xD8, 0xC8", "fstp dword ptr [{o}]";
        a = &raw const a, o = &raw mut out);
        out
    }

    /// `fild i; fdiv d; fld hi; fsub lo; fmulp st(1), st; fadd lo; fstp`: lo + i / d (hi -
    /// lo), the products in that order.
    pub(super) fn int_lerp(i: i32, d: u32, hi: u32, lo: u32) -> u32 {
        let (v, mut out) = ([i as u32, d, hi, lo], 0u32);
        x87!("fild dword ptr [{v}]", "fdiv dword ptr [{v} + 4]", "fld dword ptr [{v} + 8]",
        "fsub dword ptr [{v} + 12]", ".byte 0xDE, 0xC9", "fadd dword ptr [{v} + 12]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld a; fsub b; fdiv c; fistp`.
    pub(super) fn sub_div_to_int(a: u32, b: u32, c: u32) -> i32 {
        let (v, mut out) = ([a, b, c], 0i32);
        x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fdiv dword ptr [{v} + 8]",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fild a` as an 80-bit value; with `negate`, `fchs` after.
    pub(super) fn int_80(a: i32, negate: bool) -> [u8; 10] {
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
    pub(super) fn ext_to_int(e: [u8; 10]) -> i32 {
        let mut out = 0i32;
        x87!("fld tbyte ptr [{e}]", "fistp dword ptr [{o}]"; e = e.as_ptr(), o = &raw mut out);
        out
    }

    /// `fild a; fild b; fdivp st(1), st; fadd c; fstp`: a / b + c.
    pub(super) fn int_over_int_plus(a: i32, b: i32, c: u32) -> u32 {
        let (v, mut out) = ([a as u32, b as u32, c], 0u32);
        x87!("fild dword ptr [{v}]", "fild dword ptr [{v} + 4]", ".byte 0xDE, 0xF9",
        "fadd dword ptr [{v} + 8]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld a; fsincos; fstp c; fstp s`: (sine, cosine) rounded to floats.
    pub(super) fn sine_cosine_of(a: u32) -> (u32, u32) {
        let mut out = [0u32; 2];
        x87!("fld dword ptr [{a}]", "fsincos", "fstp dword ptr [{o} + 4]", "fstp dword ptr [{o}]";
        a = &raw const a, o = out.as_mut_ptr());
        (out[0], out[1])
    }

    /// `fld1; fsub a; fstp`.
    pub(super) fn one_less(a: u32) -> u32 {
        let mut out = 0u32;
        x87!("fld1", "fsub dword ptr [{a}]", "fstp dword ptr [{o}]";
        a = &raw const a, o = &raw mut out);
        out
    }

    /// `fld x; fabs; fsqrt; fmul t; fadd c; fstp`.
    pub(super) fn abs_sqrt_times_plus(x: u32, t: u32, c: u32) -> u32 {
        let (v, mut out) = ([x, t, c], 0u32);
        x87!("fld dword ptr [{v}]", "fabs", "fsqrt", "fmul dword ptr [{v} + 4]",
        "fadd dword ptr [{v} + 8]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld a; fadd b; fadd c; fstp`.
    pub(super) fn add3(a: u32, b: u32, c: u32) -> u32 {
        let (v, mut out) = ([a, b, c], 0u32);
        x87!("fld dword ptr [{v}]", "fadd dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld a; fmul b; fmul c; fistp`.
    pub(super) fn mul2_to_int(a: u32, b: u32, c: u32) -> i32 {
        let (v, mut out) = ([a, b, c], 0i32);
        x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld a; fmul b; fchs; fadd c; fstp`: c less a b.
    pub(super) fn neg_product_plus(a: u32, b: u32, c: u32) -> u32 {
        let (v, mut out) = ([a, b, c], 0u32);
        x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fchs", "fadd dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld a; fmul b; fchs; fstp`.
    pub(super) fn neg_mul(a: u32, b: u32) -> u32 {
        let (v, mut out) = ([a, b], 0u32);
        x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fchs", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld a; fabs; fcomp b; fnstsw`: C0 .. C3 (0x4700).
    pub(super) fn abs_compare(a: u32, b: u32) -> u16 {
        let (v, mut status) = ([a, b], 0u16);
        x87!("fld dword ptr [{v}]", "fabs", "fcomp dword ptr [{v} + 4]", "fnstsw word ptr [{s}]";
        v = v.as_ptr(), s = &raw mut status);
        status & 0x4700
    }

    /// `fld a; fabs; fstp`.
    pub(super) fn abs_of(a: u32) -> u32 {
        let mut out = 0u32;
        x87!("fld dword ptr [{a}]", "fabs", "fstp dword ptr [{o}]"; a = &raw const a, o = &raw mut out);
        out
    }

    /// `fld a; fmul st, st; fld b; fmul st, st; faddp; fsqrt; fistp`: the length of two
    /// floats at right angles, as a whole number.
    pub(super) fn float_hypot_to_int(a: u32, b: u32) -> i32 {
        let (v, mut out) = ([a, b], 0i32);
        x87!("fld dword ptr [{v}]", ".byte 0xD8, 0xC8", "fld dword ptr [{v} + 4]", ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1", "fsqrt", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fild i; fsub a; fmul k; fadd a; fistp`: a + (i - a) k as a whole number.
    pub(super) fn int_toward_to_int(i: i32, a: u32, k: u32) -> i32 {
        let (v, mut out) = ([i as u32, a, k], 0i32);
        x87!("fild dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fadd dword ptr [{v} + 4]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld a; fmul k; fist; fcomp z; fnstsw`: the product as a whole number, and the
    /// compare's C0 .. C3 (0x4700) of the product (unrounded) against z.
    pub(super) fn times_fist_compare(a: u32, k: u32, z: u32) -> (i32, u16) {
        let (v, mut out, mut status) = ([a, k, z], 0i32, 0u16);
        x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fist dword ptr [{o}]",
        "fcomp dword ptr [{v} + 8]", "fnstsw word ptr [{s}]";
        v = v.as_ptr(), o = &raw mut out, s = &raw mut status);
        (out, status & 0x4700)
    }

    /// `fld a; fdiv b; fmul c; fmul d; fmul e; fistp`.
    pub(super) fn div_mul3_to_int(a: u32, b: u32, c: u32, d: u32, e: u32) -> i32 {
        let (v, mut out) = ([a, b, c, d, e], 0i32);
        x87!("fld dword ptr [{v}]", "fdiv dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fmul dword ptr [{v} + 12]", "fmul dword ptr [{v} + 16]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld e (80 bits); fdiv a; fdiv b; fstp`.
    pub(super) fn ext_div2(e: [u8; 10], a: u32, b: u32) -> u32 {
        let (v, mut out) = ([a, b], 0u32);
        x87!("fld tbyte ptr [{e}]", "fdiv dword ptr [{v}]", "fdiv dword ptr [{v} + 4]",
        "fstp dword ptr [{o}]";
        e = e.as_ptr(), v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld t0; fadd t1; ..; fdiv d0; fdiv d1; ..; fstp`: the terms' sum over the divisors,
    /// the 80 bits carried between the steps through memory (exactly).
    pub(super) fn sum_over(terms: &[u32], divisors: &[u32]) -> u32 {
        let mut acc = [0u8; 10];
        x87!("fld dword ptr [{a}]", "fstp tbyte ptr [{o}]"; a = &raw const terms[0], o = acc.as_mut_ptr());
        for t in &terms[1..] {
            x87!("fld tbyte ptr [{o}]", "fadd dword ptr [{t}]", "fstp tbyte ptr [{o}]";
            o = acc.as_mut_ptr(), t = std::ptr::from_ref::<u32>(t));
        }
        for d in divisors {
            x87!("fld tbyte ptr [{o}]", "fdiv dword ptr [{d}]", "fstp tbyte ptr [{o}]";
            o = acc.as_mut_ptr(), d = std::ptr::from_ref::<u32>(d));
        }
        crate::scene::matrix3::rounded(acc)
    }

    /// `fld x; fmul st, st; fmul k; fdiv d; fstp`.
    pub(super) fn square_times_over(x: u32, k: u32, d: u32) -> u32 {
        let (v, mut out) = ([x, k, d], 0u32);
        x87!("fld dword ptr [{v}]", ".byte 0xD8, 0xC8", "fmul dword ptr [{v} + 4]",
        "fdiv dword ptr [{v} + 8]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld1; fsub a; fmul b; fadd a; fstp`: a + (1 - a) b.
    pub(super) fn toward_one(a: u32, b: u32) -> u32 {
        let (v, mut out) = ([a, b], 0u32);
        x87!("fld1", "fsub dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fadd dword ptr [{v}]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld a; fsub b; fmul k; fimul i; fadd b; fdiv d; fdiv w; fstp`.
    pub(super) fn lerp_int_over2(a: u32, b: u32, k: u32, i: i32, d: u32, w: u32) -> u32 {
        let (v, mut out) = ([a, b, k, i as u32, d, w], 0u32);
        x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fimul dword ptr [{v} + 12]", "fadd dword ptr [{v} + 4]", "fdiv dword ptr [{v} + 16]",
        "fdiv dword ptr [{v} + 20]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld top; fsub low; fild n; fsub one; fdivp st(1), st`: the step between gears,
    /// unrounded.
    pub(super) fn gear_step_80(top: u32, low: u32, n: i32, one: u32) -> [u8; 10] {
        let (v, mut out) = ([top, low, n as u32, one], [0u8; 10]);
        x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fild dword ptr [{v} + 8]",
        "fsub dword ptr [{v} + 12]", ".byte 0xDE, 0xF9", "fstp tbyte ptr [{o}]";
        v = v.as_ptr(), o = out.as_mut_ptr());
        out
    }

    /// `fadd st, st(1)` of two 80-bit values (a in st(0), b in st(1)), unrounded.
    pub(super) fn ext_add(a: [u8; 10], b: [u8; 10]) -> [u8; 10] {
        let mut out = [0u8; 10];
        x87!("fld tbyte ptr [{b}]", "fld tbyte ptr [{a}]", ".byte 0xD8, 0xC1", "fstp tbyte ptr [{o}]",
        "fstp st(0)";
        a = a.as_ptr(), b = b.as_ptr(), o = out.as_mut_ptr());
        out
    }

    /// `fld v; fsub one; fmul a; fadd one; fmul g; fmul m; fchs; fmul a; fstp`.
    pub(super) fn weight_force(v: u32, one: u32, a: u32, g: u32, m: u32) -> u32 {
        let (x, mut out) = ([v, one, a, g, m], 0u32);
        x87!("fld dword ptr [{x}]", "fsub dword ptr [{x} + 4]", "fmul dword ptr [{x} + 8]",
        "fadd dword ptr [{x} + 4]", "fmul dword ptr [{x} + 12]", "fmul dword ptr [{x} + 16]",
        "fchs", "fmul dword ptr [{x} + 8]", "fstp dword ptr [{o}]";
        x = x.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld a; fmul b; fmul c; fchs; fadd d; fstp`: d less a b c.
    pub(super) fn neg_product2_plus(a: u32, b: u32, c: u32, d: u32) -> u32 {
        let (v, mut out) = ([a, b, c, d], 0u32);
        x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fchs", "fadd dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `physics::clamped_asin`'s body (0x42120B) on the 80-bit `x`, its scratch float at `s`
    /// and its limit at `l` (negated and put back on the way, as the original does): the
    /// angle whose sine is `x`, by `fpatan` of `x` over the root of 1 less the square of `x`
    /// stored as a float and held to -limit ..= limit.
    pub(super) unsafe fn clamped_asin_80(x: [u8; 10], s: *mut u32, l: *mut u32) -> [u8; 10] {
        let mut out = [0u8; 10];
        let io = [x.as_ptr() as usize as u32, out.as_mut_ptr() as usize as u32];
        unsafe {
            core::arch::asm!(
                "mov eax, [{io}]",
                "fld tbyte ptr [eax]",
                "fst dword ptr [{s}]",
                "fld dword ptr [{l}]",
                "fld dword ptr [{s}]",
                "fcompp",
                "fnstsw ax",
                "shr ah, 1",
                "jb 2f",
                "fld dword ptr [{l}]",
                "fstp dword ptr [{s}]",
                "2:",
                "fld dword ptr [{l}]",
                "fchs",
                "fstp dword ptr [{l}]",
                "fld dword ptr [{l}]",
                "fld dword ptr [{s}]",
                "fcompp",
                "fnstsw ax",
                "shr ah, 1",
                "jnb 3f",
                "fld dword ptr [{l}]",
                "fstp dword ptr [{s}]",
                "3:",
                "fld dword ptr [{l}]",
                "fchs",
                "fstp dword ptr [{l}]",
                "fld dword ptr [{s}]",
                ".byte 0xD8, 0xC8", // fmul st, st
                "fld1",
                ".byte 0xDE, 0xE1", // fsubrp st(1), st: 1 less the square
                "fsqrt",
                "fpatan",
                "mov eax, [{io} + 4]",
                "fstp tbyte ptr [eax]",
                io = in(reg) io.as_ptr(),
                s = in(reg) s,
                l = in(reg) l,
                out("eax") _,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
        out
    }

    /// `fld y; fld x; fpatan; fstp`: the angle of (x, y), as a float.
    pub(super) fn patan(y: u32, x: u32) -> u32 {
        let (v, mut out) = ([y, x], 0u32);
        x87!("fld dword ptr [{v}]", "fld dword ptr [{v} + 4]", "fpatan", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld a; fld b; fcos; fdivp st(1), st`: a over the cosine of b, unrounded.
    pub(super) fn over_cos_80(a: u32, b: u32) -> [u8; 10] {
        let (v, mut out) = ([a, b], [0u8; 10]);
        x87!("fld dword ptr [{v}]", "fld dword ptr [{v} + 4]", "fcos", ".byte 0xDE, 0xF9",
        "fstp tbyte ptr [{o}]";
        v = v.as_ptr(), o = out.as_mut_ptr());
        out
    }

    /// `fld a; fmul b; fmul c; fmul d; fistp`.
    pub(super) fn mul3_to_int(a: u32, b: u32, c: u32, d: u32) -> i32 {
        let (v, mut out) = ([a, b, c, d], 0i32);
        x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fmul dword ptr [{v} + 12]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld tbyte x; fchs; fstp dword`.
    pub(super) fn neg_rounded(x: [u8; 10]) -> u32 {
        let mut out = 0u32;
        x87!("fld tbyte ptr [{x}]", "fchs", "fstp dword ptr [{o}]"; x = x.as_ptr(), o = &raw mut out);
        out
    }

    /// `physics::lever_pushes`' share (0x43DE23 .. 0x43DEAC): A = |a| + |d|, B = |b| + |c|
    /// (each `fld; fabs; fld; fabs; faddp`), their sum B + A compared with `zero` (`fcom`);
    /// equal or unordered: the sum stored, then A over it (the second wins): returns A
    /// rounded and true; else A / (B + A) rounded and false.
    pub(super) fn lever_share(a: u32, d: u32, b: u32, c: u32, zero: u32) -> (u32, bool) {
        let v = [a, d, b, c, zero];
        let mut out = [0u32; 2];
        unsafe {
            core::arch::asm!(
                "fld dword ptr [{v}]",
                "fabs",
                "fld dword ptr [{v} + 4]",
                "fabs",
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fld dword ptr [{v} + 8]",
                "fabs",
                "fld dword ptr [{v} + 12]",
                "fabs",
                ".byte 0xDE, 0xC1",
                ".byte 0xD8, 0xC1", // fadd st, st(1)
                "fcom dword ptr [{v} + 16]",
                "fnstsw ax",
                "and ah, 0x40",
                "jnz 2f",
                ".byte 0xDE, 0xF9", // fdivp st(1), st
                "fstp dword ptr [{o}]",
                "jmp 3f",
                "2:",
                "fstp dword ptr [{o}]",
                "fstp dword ptr [{o}]",
                "mov dword ptr [{o} + 4], 1",
                "3:",
                v = in(reg) v.as_ptr(),
                o = in(reg) out.as_mut_ptr(),
                out("eax") _,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
        (out[0], out[1] != 0)
    }

    /// `fld a; fmul b; fmul c; fchs; fstp`.
    pub(super) fn neg_mul2(a: u32, b: u32, c: u32) -> u32 {
        let (v, mut out) = ([a, b, c], 0u32);
        x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]", "fchs",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld a; fsub b; fmul c; fstp`.
    pub(super) fn sub_mul(a: u32, b: u32, c: u32) -> u32 {
        let (v, mut out) = ([a, b, c], 0u32);
        x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld a; fsub b; fadd c; fld x; fsub d; fmul e; faddp st(1), st; fmul f; fistp`:
    /// ((a - b + c) + (x - d) e) f as a whole number.
    pub(super) fn wheel_int(v: [u32; 7]) -> i32 {
        let mut out = 0i32;
        x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]",
        "fld dword ptr [{v} + 12]", "fsub dword ptr [{v} + 16]", "fmul dword ptr [{v} + 20]",
        ".byte 0xDE, 0xC1", "fmul dword ptr [{v} + 24]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld x; fabs; fdiv a; fadd b; fmul c; fstp`.
    pub(super) fn abs_div_add_mul(x: u32, a: u32, b: u32, c: u32) -> u32 {
        let (v, mut out) = ([x, a, b, c], 0u32);
        x87!("fld dword ptr [{v}]", "fabs", "fdiv dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]",
        "fmul dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld a; fmul b; fmul c; fadd d; fstp`.
    pub(super) fn mul2_plus(a: u32, b: u32, c: u32, d: u32) -> u32 {
        let (v, mut out) = ([a, b, c, d], 0u32);
        x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fadd dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld a; fld s; fabs; fadd b; fdivp st(1), st; fsub c; fstp`: a / (|s| + b) - c.
    pub(super) fn over_abs_plus_minus(a: u32, s: u32, b: u32, c: u32) -> u32 {
        let (v, mut out) = ([a, s, b, c], 0u32);
        x87!("fld dword ptr [{v}]", "fld dword ptr [{v} + 4]", "fabs", "fadd dword ptr [{v} + 8]",
        ".byte 0xDE, 0xF9", "fsub dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fild a; fisub b` as an 80-bit value: a - b, exact.
    pub(super) fn ints_diff_80(a: i32, b: i32) -> [u8; 10] {
        let (v, mut out) = ([a, b], [0u8; 10]);
        x87!("fild dword ptr [{v}]", "fisub dword ptr [{v} + 4]", "fstp tbyte ptr [{o}]";
        v = v.as_ptr(), o = out.as_mut_ptr());
        out
    }

    /// `fld x; fabs; fidiv n; fstp`.
    pub(super) fn abs_fidiv(x: u32, n: i32) -> u32 {
        let (v, mut out) = ([x, n as u32], 0u32);
        x87!("fld dword ptr [{v}]", "fabs", "fidiv dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fild a; fidiv b; fstp`.
    pub(super) fn int_fidiv(a: i32, b: i32) -> u32 {
        let (v, mut out) = ([a, b], 0u32);
        x87!("fild dword ptr [{v}]", "fidiv dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld1; fsub d; fmul s; fadd d; fstp`: d + (1 - d) s.
    pub(super) fn one_less_times_plus(d: u32, s: u32) -> u32 {
        let (v, mut out) = ([d, s], 0u32);
        x87!("fld1", "fsub dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fadd dword ptr [{v}]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld a; fsub b; fmul c; fadd d; fstp`.
    pub(super) fn sub_mul_add(a: u32, b: u32, c: u32, d: u32) -> u32 {
        let (v, mut out) = ([a, b, c, d], 0u32);
        x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fadd dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld a; fmul b; fdiv c; fcomp k; fnstsw`: C0 .. C3 (0x4700).
    pub(super) fn mul_div_status(a: u32, b: u32, c: u32, k: u32) -> u16 {
        let (v, mut status) = ([a, b, c, k], 0u16);
        x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fdiv dword ptr [{v} + 8]",
        "fcomp dword ptr [{v} + 12]", "fnstsw word ptr [{s}]";
        v = v.as_ptr(), s = &raw mut status);
        status & 0x4700
    }

    /// `fld a; fmul b; fcomp c; fnstsw`: C0 .. C3 (0x4700).
    pub(super) fn mul_status(a: u32, b: u32, c: u32) -> u16 {
        let (v, mut status) = ([a, b, c], 0u16);
        x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fcomp dword ptr [{v} + 8]",
        "fnstsw word ptr [{s}]";
        v = v.as_ptr(), s = &raw mut status);
        status & 0x4700
    }

    /// `fld a; fmul b; fistp` with the rounding set to chop (`fstcw`, RC 11, `fldcw`, put back
    /// after), the precision as it was.
    pub(super) fn trunc_mul_to_int(a: u32, b: u32) -> i32 {
        let v = [a, b];
        let mut out = 0i32;
        let mut cw = [0u16; 2];
        unsafe {
            core::arch::asm!(
                "fnstcw word ptr [{c}]",
                "mov ax, word ptr [{c}]",
                "and ax, 0xF3FF",
                "or ax, 0x0C00",
                "mov word ptr [{c} + 2], ax",
                "fldcw word ptr [{c} + 2]",
                "fld dword ptr [{v}]",
                "fmul dword ptr [{v} + 4]",
                "fistp dword ptr [{o}]",
                "fldcw word ptr [{c}]",
                c = in(reg) cw.as_mut_ptr(),
                v = in(reg) v.as_ptr(),
                o = in(reg) &raw mut out,
                out("eax") _,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
        out
    }

    /// `fld s; fsub a0; fld a1; fsub a0; fdivp; fld c1; fsub c0; fmulp; fadd c0; fstp`: the
    /// table line between (a0, c0) and (a1, c1) at s.
    pub(super) fn lerp_table(s: u32, a0: u32, a1: u32, c0: u32, c1: u32) -> u32 {
        let (v, mut out) = ([s, a0, a1, c0, c1], 0u32);
        x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fld dword ptr [{v} + 8]",
        "fsub dword ptr [{v} + 4]", ".byte 0xDE, 0xF9", "fld dword ptr [{v} + 16]",
        "fsub dword ptr [{v} + 12]", ".byte 0xDE, 0xC9", "fadd dword ptr [{v} + 12]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld a; fdiv b; fmul c; fstp`.
    pub(super) fn div_then_mul(a: u32, b: u32, c: u32) -> u32 {
        let (v, mut out) = ([a, b, c], 0u32);
        x87!("fld dword ptr [{v}]", "fdiv dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld a; fadd b; fild n; fmulp; fdiv k; fsub b; fstp`.
    pub(super) fn sum_times_int_over_minus(a: u32, b: u32, n: i32, k: u32) -> u32 {
        let (v, mut out) = ([a, b, n as u32, k], 0u32);
        x87!("fld dword ptr [{v}]", "fadd dword ptr [{v} + 4]", "fild dword ptr [{v} + 8]",
        ".byte 0xDE, 0xC9", "fdiv dword ptr [{v} + 12]", "fsub dword ptr [{v} + 4]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fld a; fmul b; fdiv c; fdiv d; fstp`.
    pub(super) fn mul_div2(a: u32, b: u32, c: u32, d: u32) -> u32 {
        let (v, mut out) = ([a, b, c, d], 0u32);
        x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fdiv dword ptr [{v} + 8]",
        "fdiv dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
        out
    }

    /// `fild i; fmul k; fadd a; (fadd b;) fstp`.
    pub(super) fn int_mul_add(i: i32, k: u32, a: u32, b: Option<u32>) -> u32 {
        let (v, mut out) = ([i as u32, k, a, b.unwrap_or(0)], 0u32);
        if b.is_some() {
            x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]",
            "fadd dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
            v = v.as_ptr(), o = &raw mut out);
        } else {
            x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]",
            "fstp dword ptr [{o}]";
            v = v.as_ptr(), o = &raw mut out);
        }
        out
    }

    /// `fld a; fdiv b` as an 80-bit value.
    pub(super) fn div_80(a: u32, b: u32) -> [u8; 10] {
        let (v, mut out) = ([a, b], [0u8; 10]);
        x87!("fld dword ptr [{v}]", "fdiv dword ptr [{v} + 4]", "fstp tbyte ptr [{o}]";
        v = v.as_ptr(), o = out.as_mut_ptr());
        out
    }

    /// `fld tbyte e; fcomp dword b; fnstsw`: the status word's C0 .. C3 (0x4700), an 80-bit
    /// value against a float.
    pub(super) fn compare_80(e: [u8; 10], b: u32) -> u16 {
        let mut status = 0u16;
        x87!("fld tbyte ptr [{e}]", "fcomp dword ptr [{b}]", "fnstsw word ptr [{s}]";
        e = e.as_ptr(), b = &raw const b, s = &raw mut status);
        status & 0x4700
    }
}

/// `normalise_vector`'s x87 pieces (0x45C3FE).
pub(super) mod norm {
    /// The squares' sum as a float.
    pub(super) fn squares_sum(v: [u32; 3]) -> u32 {
        let mut sum = 0u32;
        unsafe {
            core::arch::asm!(
                "fld dword ptr [{v}]",
                ".byte 0xD8, 0xC8", // fmul st, st
                "fld dword ptr [{v} + 4]",
                ".byte 0xD8, 0xC8", // fmul st, st
                "fld dword ptr [{v} + 8]",
                ".byte 0xD8, 0xC8", // fmul st, st
                ".byte 0xDE, 0xC1", // faddp st(1), st
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fstp dword ptr [{s}]",
                v = in(reg) v.as_ptr(),
                s = in(reg) &raw mut sum,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
        sum
    }

    /// The components times 1 over the length (the two roots' product), and that inverse.
    pub(super) fn scaled_by_inverse(v: [u32; 3], root: u32, exponent_root: u32) -> ([u32; 3], u32) {
        // The vector, the two roots, the length, 1 over it.
        let mut w = [v[0], v[1], v[2], root, exponent_root, 0, 0];
        unsafe {
            core::arch::asm!(
                "fld dword ptr [{w} + 12]",
                "fmul dword ptr [{w} + 16]",
                "fst dword ptr [{w} + 20]",
                "fld1",
                ".byte 0xDE, 0xF1", // fdivrp st(1), st
                "fld dword ptr [{w}]",
                ".byte 0xD8, 0xC9", // fmul st, st(1)
                "fstp dword ptr [{w}]",
                "fld dword ptr [{w} + 4]",
                ".byte 0xD8, 0xC9", // fmul st, st(1)
                "fstp dword ptr [{w} + 4]",
                "fld dword ptr [{w} + 8]",
                ".byte 0xD8, 0xC9", // fmul st, st(1)
                "fstp dword ptr [{w} + 8]",
                "fstp dword ptr [{w} + 24]",
                w = in(reg) w.as_mut_ptr(),
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
        ([w[0], w[1], w[2]], w[6])
    }
}

// ---- the differential tests ----

#[cfg(test)]
mod tests {
    use super::{load, norm, run};
    use crate::math::x87::set_control_word;
    use crate::math::x87_float::{X, f80, from_f80};
    use crate::math::x87_reference_a::AsBits;
    use crate::math::{normalise_vector, x87};

    /// The control words tried: double precision with nearest (the game's), chop, down and
    /// up. Both the real and the emulated control word are set.
    const ROUNDING: [u16; 4] = [0x027F, 0x0E7F, 0x067F, 0x0A7F];

    /// `fld qword; fstp tbyte`.
    fn f64_to_f80(x: f64) -> [u8; 10] {
        f80(X(x))
    }
    /// `fld tbyte; fstp qword` (to nearest even).
    fn f80_to_f64(b: [u8; 10]) -> f64 {
        from_f80(b).0
    }
    const CASES: usize = 20_000;

    struct Rng(u64);

    impl Rng {
        fn new(name: &str) -> Rng {
            // FNV-1a of the name: every function its own seed.
            let mut h = 0xCBF2_9CE4_8422_2325u64;
            for b in name.bytes() {
                h = (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01B3);
            }
            Rng(h | 1)
        }
        fn u64(&mut self) -> u64 {
            self.0 ^= self.0 >> 12;
            self.0 ^= self.0 << 25;
            self.0 ^= self.0 >> 27;
            self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
        }
        fn u32(&mut self) -> u32 {
            (self.u64() >> 32) as u32
        }
        fn below(&mut self, n: usize) -> usize {
            (self.u64() % n as u64) as usize
        }
        /// A finite float's bits: random bit patterns and the values that round oddly.
        fn f(&mut self) -> u32 {
            const EDGE: [u32; 24] = [
                0x0000_0000,
                0x8000_0000,
                0x3F80_0000,
                0xBF80_0000,
                0x3F00_0000,
                0x3FC0_0000,
                0x4020_0000,
                0xBF00_0000,
                0xBFC0_0000,
                0x0000_0001,
                0x007F_FFFF,
                0x0080_0000,
                0x7F7F_FFFF,
                0xFF7F_FFFF,
                0x4EFF_FFFF,
                0x4F00_0000,
                0xCF00_0000,
                0x4F80_0000,
                0x501C_F6C0,
                0x3DCC_CCCD,
                0x4049_0FDB,
                0x4AFF_FFFF,
                0xCAFF_FFFF,
                0x3FC9_0FDB,
            ];
            match self.below(12) {
                0 => loop {
                    let b = self.u32();
                    if f32::from_bits(b).is_finite() {
                        return b;
                    }
                },
                1 | 2 => EDGE[self.below(EDGE.len())],
                3 => ((self.below(2001) as i32 - 1000) as f32).to_bits(),
                4 => ((self.below(4001) as i32 - 2000) as f32 * 0.5).to_bits(),
                5 => ((self.below(8001) as i32 - 4000) as f32 * 0.25).to_bits(),
                6 => {
                    let (s, e, m) = (
                        self.u32() & 0x8000_0000,
                        100 + self.below(51) as u32,
                        self.u32(),
                    );
                    s | (e << 23) | (m & 0x7F_FFFF)
                }
                7 => {
                    // Near the edge of the whole numbers.
                    let base = if self.below(2) == 0 {
                        2147483648.0f32
                    } else {
                        -2147483648.0
                    };
                    (base + (self.below(1025) as f32 - 512.0) * 128.0).to_bits()
                }
                8 => (self.u32() as f32 / u32::MAX as f32 * 2.0 - 1.0).to_bits(),
                _ => {
                    let (s, e, m) = (
                        self.u32() & 0x8000_0000,
                        117 + self.below(21) as u32,
                        self.u32(),
                    );
                    s | (e << 23) | (m & 0x7F_FFFF)
                }
            }
        }
        /// As `f`, now and then no number or an infinity.
        fn fw(&mut self) -> u32 {
            match self.below(20) {
                0 => 0x7FC0_0000,
                1 => 0xFFC0_0000,
                2 => 0x7F80_0000,
                3 => 0xFF80_0000,
                _ => self.f(),
            }
        }
        /// A positive float of a plain size.
        fn pos(&mut self) -> u32 {
            self.f() & 0x7FFF_FFFF
        }
        /// A whole number: any, small, or at the edges.
        fn i(&mut self) -> i32 {
            const EDGE: [i32; 12] = [
                0,
                1,
                -1,
                i32::MAX,
                i32::MIN,
                65536,
                -65536,
                46341,
                16_777_216,
                16_777_217,
                -16_777_217,
                1 << 30,
            ];
            match self.below(8) {
                0 => self.u32() as i32,
                1 | 2 => self.below(201) as i32 - 100,
                3 => EDGE[self.below(EDGE.len())],
                4 => (self.u32() >> 8) as i32 - (1 << 23),
                5 => (self.u32() >> 12) as i32 - (1 << 19),
                _ => self.below(4001) as i32 - 2000,
            }
        }
        /// A float of a size an angle may have.
        fn angle(&mut self) -> u32 {
            match self.below(6) {
                0 => (self.u32() as f32 / u32::MAX as f32 * 2000.0 - 1000.0).to_bits(),
                1 => ((self.below(801) as f32 - 400.0) * core::f32::consts::FRAC_PI_2).to_bits(),
                2 => (self.u32() as f32 / u32::MAX as f32 * 20.0 - 10.0).to_bits(),
                3 => (self.u32() as f32 / u32::MAX as f32 * 6.0e5 - 3.0e5).to_bits(),
                _ => (self.u32() as f32 / u32::MAX as f32 * 8.0 - 4.0).to_bits(),
            }
        }
        /// An 80-bit value that came from a double.
        fn e80(&mut self) -> [u8; 10] {
            let x = match self.below(4) {
                0 => f64::from(f32::from_bits(self.f())),
                1 => f64::from(self.i()),
                2 => f64::from(f32::from_bits(self.f())) * (1.0 + self.u32() as f64 / 4.0e12),
                _ => (self.u64() as f64 / u64::MAX as f64 - 0.5) * 4.0,
            };
            f64_to_f80(x)
        }
    }

    fn float_key(b: u32) -> i64 {
        if b >> 31 == 1 {
            -i64::from(b & 0x7FFF_FFFF)
        } else {
            i64::from(b)
        }
    }
    fn double_key(b: u64) -> i128 {
        if b >> 63 == 1 {
            -i128::from(b & 0x7FFF_FFFF_FFFF_FFFF)
        } else {
            i128::from(b)
        }
    }

    /// Bit for bit, any NaN equal to any NaN.
    fn eqf(a: &u32, b: &u32) -> bool {
        a == b || (f32::from_bits(*a).is_nan() && f32::from_bits(*b).is_nan())
    }
    fn eqf2(a: &(u32, u32), b: &(u32, u32)) -> bool {
        eqf(&a.0, &b.0) && eqf(&a.1, &b.1)
    }
    fn eqf3(a: &[u32; 3], b: &[u32; 3]) -> bool {
        (0..3).all(|k| eqf(&a[k], &b[k]))
    }
    fn eq80(a: &[u8; 10], b: &[u8; 10]) -> bool {
        let nan = |x: &[u8; 10]| {
            let mant = u64::from_le_bytes(x[..8].try_into().unwrap());
            u16::from_le_bytes([x[8], x[9]]) & 0x7FFF == 0x7FFF && mant << 1 != 0
        };
        a == b || (nan(a) && nan(b))
    }
    fn exact<T: PartialEq>(a: &T, b: &T) -> bool {
        a == b
    }
    /// Within `u` ulp of a float.
    fn near_f(a: &u32, b: &u32, u: i64) -> bool {
        eqf(a, b) || (float_key(*a) - float_key(*b)).abs() <= u
    }
    /// Within `u` ulp of a double (the 80-bit values read as doubles).
    fn near_80(a: &[u8; 10], b: &[u8; 10], u: i128) -> bool {
        if eq80(a, b) {
            return true;
        }
        let (x, y) = (f80_to_f64(*a), f80_to_f64(*b));
        (double_key(x.to_bits()) - double_key(y.to_bits())).abs() <= u
    }

    /// Runs `reference` and `plain` on `CASES` inputs per control word, and fails on
    /// any difference.
    fn check<T: core::fmt::Debug, R: core::fmt::Debug, P: AsBits<Out = R>>(
        name: &str,
        cws: &[u16],
        gen_input: impl Fn(&mut Rng) -> T,
        reference: impl Fn(&T) -> R,
        plain: impl Fn(&T) -> P,
        same: impl Fn(&R, &R) -> bool,
    ) {
        let mut rng = Rng::new(name);
        let mut bad = Vec::new();
        let mut count = 0;
        for &cw in cws {
            set_control_word(cw);
            for _ in 0..CASES {
                let input = gen_input(&mut rng);
                let (a, b) = (reference(&input), plain(&input).into_bits());
                if !same(&a, &b) {
                    count += 1;
                    if bad.len() < 5 {
                        bad.push(format!(
                            "cw {cw:04X}: {input:X?} reference {a:X?} plain {b:X?}"
                        ));
                    }
                }
            }
        }
        set_control_word(0x027F);
        assert!(
            count == 0,
            "{name}: {count} differ, e.g.\n{}",
            bad.join("\n")
        );
    }

    /// `check` at all four control words.
    fn check_all<T: core::fmt::Debug, R: core::fmt::Debug, P: AsBits<Out = R>>(
        name: &str,
        gen_input: impl Fn(&mut Rng) -> T,
        reference: impl Fn(&T) -> R,
        plain: impl Fn(&T) -> P,
        same: impl Fn(&R, &R) -> bool,
    ) {
        check(name, &ROUNDING, gen_input, reference, plain, same);
    }

    // ---- the boundary: 80 bits and doubles ----

    #[test]
    fn f64_to_f80_is_fld_qword_fstp_tbyte() {
        let mut rng = Rng::new("f64_to_f80");
        set_control_word(0x027F);
        for k in 0..100_000 {
            let bits = match k % 5 {
                0 => rng.u64(),
                1 => rng.u64() & 0x800F_FFFF_FFFF_FFFF, // denormals and zeros
                2 => f64::from(f32::from_bits(rng.f())).to_bits(),
                3 => (rng.i() as f64).to_bits(),
                _ => 0x7FF0_0000_0000_0000 | (rng.u64() & 0x800F_FFFF_FFFF_FFFF),
            };
            let x = [bits];
            let mut out = [0u8; 10];
            unsafe {
                core::arch::asm!(
                    "fld qword ptr [{x}]", "fstp tbyte ptr [{o}]",
                    x = in(reg) x.as_ptr(), o = in(reg) out.as_mut_ptr(),
                    out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                    out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                    options(nostack),
                );
            }
            let mine = f64_to_f80(f64::from_bits(bits));
            // A NaN keeps its payload; the quiet bit the instruction sets is set here too.
            assert!(
                eq80(&out, &mine),
                "{bits:016X}: x87 {out:02X?} plain {mine:02X?}"
            );
        }
    }

    #[test]
    fn f80_to_f64_is_fld_tbyte_fstp_qword() {
        let mut rng = Rng::new("f80_to_f64");
        set_control_word(0x027F);
        for k in 0..100_000 {
            let mant = (rng.u64() | (1 << 63)) & if k % 3 == 0 { !0x7FF } else { !0 };
            let e = 16383 - 1000 + rng.below(2000) as u16;
            let mut b = [0u8; 10];
            b[..8].copy_from_slice(&mant.to_le_bytes());
            let sign = if rng.below(2) == 0 { 0 } else { 0x8000 };
            b[8..].copy_from_slice(&(e | sign).to_le_bytes());
            let mut out = [0u64];
            unsafe {
                core::arch::asm!(
                    "fld tbyte ptr [{x}]", "fstp qword ptr [{o}]",
                    x = in(reg) b.as_ptr(), o = in(reg) out.as_mut_ptr(),
                    out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                    out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                    options(nostack),
                );
            }
            assert_eq!(out[0], f80_to_f64(b).to_bits(), "{b:02X?}");
        }
        // Zero, infinity, a number too large for a double.
        assert_eq!(f80_to_f64([0; 10]).to_bits(), 0);
        assert_eq!(f80_to_f64(f64_to_f80(f64::NEG_INFINITY)), f64::NEG_INFINITY);
        assert!(f80_to_f64(f64_to_f80(f64::NAN)).is_nan());
        assert_eq!(f80_to_f64(f64_to_f80(-0.0)).to_bits(), (-0.0f64).to_bits());
    }

    // ---- math/x87/load.rs ----

    #[test]
    fn load_whole_number_results() {
        check_all(
            "int_times_int_over_to_int",
            |g| (g.i(), g.i(), g.f()),
            |&(a, b, c)| load::int_times_int_over_to_int(a, b, c),
            |&(a, b, c)| x87::int_times_int_over_to_int(a, b, c),
            exact,
        );
        check(
            "abs_to_int",
            &ROUNDING,
            Rng::f,
            |&a| load::abs_to_int(a),
            |&a| x87::abs_to_int(a),
            exact,
        );
        check_all(
            "add_to_int",
            |g| (g.f(), g.f()),
            |&(a, b)| load::add_to_int(a, b),
            |&(a, b)| x87::add_to_int(a, b),
            exact,
        );
        check_all(
            "int_square_sum_scaled_to_int",
            |g| (g.i(), g.i(), g.f(), 1 + g.below(2) as u32),
            |&(a, b, k, t)| load::int_square_sum_scaled_to_int(a, b, k, t),
            |&(a, b, k, t)| x87::int_square_sum_scaled_to_int(a, b, k, t),
            exact,
        );
        check(
            "product4_to_int",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f()),
            |&(a, b, c, d)| load::product4_to_int(a, b, c, d),
            |&(a, b, c, d)| x87::mul3_to_int(a, b, c, d) as u32,
            exact,
        );
        check(
            "tbyte_to_int",
            &ROUNDING,
            Rng::e80,
            |&a| load::tbyte_to_int(a),
            |&a| x87::tbyte_to_int(a),
            exact,
        );
        check_all(
            "sun_size",
            |g| (g.f(), g.f(), g.f(), g.f(), g.f(), g.f()),
            |&(x, cx, y, cy, h, k)| load::sun_size(x, cx, y, cy, h, k),
            |&(x, cx, y, cy, h, k)| x87::sun_size(x, cx, y, cy, h, k),
            exact,
        );
    }

    #[test]
    fn load_float_results() {
        check(
            "float_hypot",
            &ROUNDING,
            |g| (g.f(), g.f()),
            |&(a, b)| load::float_hypot(a, b),
            |&(a, b)| x87::float_hypot(a, b),
            eqf,
        );
        check(
            "int_square_sum",
            &ROUNDING,
            |g| (g.i(), g.i()),
            |&(a, b)| load::int_square_sum(a, b),
            |&(a, b)| x87::int_square_sum(a, b),
            eqf,
        );
        check(
            "float_square_sum",
            &ROUNDING,
            |g| (g.f(), g.f()),
            |&(a, b)| load::float_square_sum(a, b),
            |&(a, b)| x87::float_square_sum(a, b),
            eqf,
        );
        check(
            "product3",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f()),
            |&(a, b, c)| load::product3(a, b, c),
            |&(a, b, c)| x87::mul2(a, b, c),
            eqf,
        );
        check(
            "int_over_float",
            &ROUNDING,
            |g| (g.i(), g.f()),
            |&(a, b)| load::int_over_float(a, b),
            |&(a, b)| x87::int_over(a, b),
            eqf,
        );
        check(
            "int_plus_times",
            &ROUNDING,
            |g| (g.i(), g.f(), g.f()),
            |&(a, b, c)| load::int_plus_times(a, b, c),
            |&(a, b, c)| x87::int_plus_times(a, b, c),
            eqf,
        );
        check(
            "eased_roll",
            &ROUNDING,
            |g| (g.i(), g.f(), g.f(), g.f(), g.f(), g.f(), g.f()),
            |&(i, a, b, v, c, m, h)| load::eased_roll(i, a, b, v, c, m, h),
            |&(i, a, b, v, c, m, h)| x87::eased_roll(i, a, b, v, c, m, h),
            eqf,
        );
        check(
            "to_tbyte",
            &ROUNDING,
            Rng::f,
            |&a| load::to_tbyte(a),
            |&a| x87::to_tbyte(a),
            eq80,
        );
    }

    /// `fsincos`: the sine and cosine of the angles within an ulp of a float.
    #[test]
    fn load_sun_direction() {
        check(
            "sun_direction",
            &ROUNDING,
            |g| {
                (
                    g.i() % 4096,
                    g.angle(),
                    (g.below(1000) as f32 * 1e-4).to_bits(),
                    g.i() % 4096,
                    g.f(),
                )
            },
            |&(a, k, s, b, m)| load::sun_direction(a, k, s, b, m),
            |&(a, k, s, b, m)| x87::sun_direction(a, k, s, b, m),
            |a, b| (0..3).all(|k| near_f(&a[k], &b[k], 1)),
        );
    }

    #[test]
    fn load_comparisons() {
        check(
            "below_or_unordered",
            &ROUNDING,
            |g| (g.fw(), g.fw()),
            |&(a, b)| load::below_or_unordered(a, b),
            |&(a, b)| x87::below_or_unordered(a, b),
            exact,
        );
        check(
            "scaled_abs_less_below",
            &ROUNDING,
            |g| (g.fw(), g.f(), g.f(), g.fw()),
            |&(a, h, k, b)| load::scaled_abs_less_below(a, h, k, b),
            |&(a, h, k, b)| x87::scaled_abs_less_below(a, h, k, b),
            exact,
        );
        check(
            "abs_below_or_unordered",
            &ROUNDING,
            |g| (g.fw(), g.fw()),
            |&(a, k)| load::abs_below_or_unordered(a, k),
            |&(a, k)| x87::abs_below_or_unordered(a, k),
            exact,
        );
    }

    fn matrix_input(g: &mut Rng) -> ([i32; 3], [u32; 16]) {
        let mut m = [0u32; 16];
        for x in &mut m {
            *x = g.f();
        }
        ([g.i(), g.i(), g.i()], m)
    }

    #[test]
    fn load_rotations() {
        check_all(
            "rotate_rounding_z",
            matrix_input,
            |(p, m)| {
                let mut o = [0xAAAA_AAAAu32; 3];
                {
                    load::rotate_rounding_z(p.as_ptr(), m.as_ptr(), o.as_mut_ptr());
                };
                o
            },
            |(p, m)| x87::rotate_rounding_z(*p, m),
            |a, b| eqf(&a[0], &b[0]) && eqf(&a[1], &b[1]) && a[2] == b[2],
        );
        check_all(
            "rotate_rounding",
            matrix_input,
            |(p, m)| {
                let mut o = [0xAAAA_AAAAu32; 3];
                {
                    load::rotate_rounding(p.as_ptr(), m.as_ptr(), o.as_mut_ptr());
                };
                o
            },
            |(p, m)| x87::rotate_rounding(*p, m),
            exact,
        );
    }

    #[test]
    fn load_sun_on_screen() {
        check_all(
            "sun_on_screen",
            |g| {
                (
                    [
                        g.i() as u32,
                        g.i() as u32,
                        g.i() as u32,
                        g.i() as u32,
                        g.f(),
                        g.f(),
                        g.f(),
                    ],
                    g.below(2) == 0,
                )
            },
            |&(v, flat)| {
                let mut scratch = 0xAAAA_AAAAu32;
                let o = load::sun_on_screen(&v, flat, &raw mut scratch);
                (o, scratch)
            },
            |&(v, flat)| {
                let scratch = crate::session::Dword::new(0xAAAA_AAAA);
                let point = [v[0] as i32, v[1] as i32, v[2] as i32, v[3] as i32];
                let (place, hit) = x87::sun_on_screen(point, v[4], [v[5], v[6]], flat, &scratch);
                let o = [
                    place[0].to_bits(),
                    hit[0] as u32,
                    place[1].to_bits(),
                    hit[1] as u32,
                ];
                (o, scratch.get())
            },
            |a, b| {
                eqf(&a.0[0], &b.0[0])
                    && a.0[1] == b.0[1]
                    && eqf(&a.0[2], &b.0[2])
                    && a.0[3] == b.0[3]
                    && eqf(&a.1, &b.1)
            },
        );
    }

    // ---- math/x87/run.rs ----

    #[test]
    fn run_whole_number_results() {
        check_all(
            "plane_height",
            |g| (g.i(), g.i(), g.i(), g.f(), g.f()),
            |&(a, b, c, f, h)| run::plane_height(a, b, c, f, h),
            |&(a, b, c, f, h)| x87::plane_height(a, b, c, f, h),
            exact,
        );
        check(
            "int_times_times_over_to_int",
            &ROUNDING,
            |g| (g.i(), g.f(), g.f(), g.f()),
            |&(i, a, b, c)| run::int_times_times_over_to_int(i, a, b, c),
            |&(i, a, b, c)| x87::int_times_times_over_to_int(i, a, b, c),
            exact,
        );
        check_all(
            "sub_div_to_int",
            |g| (g.f(), g.f(), g.f()),
            |&(a, b, c)| run::sub_div_to_int(a, b, c),
            |&(a, b, c)| x87::sub_div_to_int(a, b, c),
            exact,
        );
        check(
            "ext_to_int",
            &ROUNDING,
            Rng::e80,
            |&e| run::ext_to_int(e),
            |&e| x87::ext_to_int(e),
            exact,
        );
        check(
            "mul2_to_int",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f()),
            |&(a, b, c)| run::mul2_to_int(a, b, c),
            |&(a, b, c)| x87::mul2_to_int(a, b, c),
            exact,
        );
        check_all(
            "float_hypot_to_int",
            |g| (g.f(), g.f()),
            |&(a, b)| run::float_hypot_to_int(a, b),
            |&(a, b)| x87::float_hypot_to_int(a, b),
            exact,
        );
        check_all(
            "int_toward_to_int",
            |g| (g.i(), g.f(), g.f()),
            |&(i, a, k)| run::int_toward_to_int(i, a, k),
            |&(i, a, k)| x87::int_toward_to_int(i, a, k),
            exact,
        );
        check(
            "times_fist_compare",
            &ROUNDING,
            |g| (g.fw(), g.f(), g.fw()),
            |&(a, k, z)| run::times_fist_compare(a, k, z),
            |&(a, k, z)| x87::times_fist_compare(a, k, z),
            exact,
        );
        check_all(
            "div_mul3_to_int",
            |g| (g.f(), g.f(), g.f(), g.f(), g.f()),
            |&(a, b, c, d, e)| run::div_mul3_to_int(a, b, c, d, e),
            |&(a, b, c, d, e)| x87::div_mul3_to_int(a, b, c, d, e),
            exact,
        );
        check(
            "mul3_to_int",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f()),
            |&(a, b, c, d)| run::mul3_to_int(a, b, c, d),
            |&(a, b, c, d)| x87::mul3_to_int(a, b, c, d),
            exact,
        );
        check_all(
            "wheel_int",
            |g| [g.f(), g.f(), g.f(), g.f(), g.f(), g.f(), g.f()],
            |&v| run::wheel_int(v),
            |&v| x87::wheel_int(v),
            exact,
        );
        check(
            "trunc_mul_to_int",
            &ROUNDING,
            |g| (g.f(), g.f()),
            |&(a, b)| run::trunc_mul_to_int(a, b),
            |&(a, b)| x87::trunc_mul_to_int(a, b),
            exact,
        );
    }

    #[test]
    fn run_float_results() {
        check(
            "sum_times_over",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f()),
            |&(a, b, h, d)| run::sum_times_over(a, b, h, d),
            |&(a, b, h, d)| x87::sum_times_over(a, b, h, d),
            eqf,
        );
        check(
            "square_times_plus_times",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f()),
            |&(a, k, c, h)| run::square_times_plus_times(a, k, c, h),
            |&(a, k, c, h)| x87::square_times_plus_times(a, k, c, h),
            eqf2,
        );
        check(
            "minus_one",
            &ROUNDING,
            |_| (),
            |_| run::minus_one(),
            |_| (-1.0f32).to_bits(),
            exact,
        );
        check(
            "mul3",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f()),
            |&(a, b, c, d)| run::mul3(a, b, c, d),
            |&(a, b, c, d)| x87::mul3(a, b, c, d),
            eqf,
        );
        check(
            "square",
            &ROUNDING,
            Rng::f,
            |&a| run::square(a),
            |&a| x87::mul(a, a),
            eqf,
        );
        check(
            "int_lerp",
            &ROUNDING,
            |g| (g.i(), g.f(), g.f(), g.f()),
            |&(i, d, hi, lo)| run::int_lerp(i, d, hi, lo),
            |&(i, d, hi, lo)| x87::int_lerp(i, d, hi, lo),
            eqf,
        );
        check(
            "int_over_int_plus",
            &ROUNDING,
            |g| (g.i(), g.i(), g.f()),
            |&(a, b, c)| run::int_over_int_plus(a, b, c),
            |&(a, b, c)| x87::int_over_int_plus(a, b, c),
            eqf,
        );
        check(
            "one_less",
            &ROUNDING,
            Rng::f,
            |&a| run::one_less(a),
            |&a| x87::one_less(a),
            eqf,
        );
        check(
            "abs_sqrt_times_plus",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f()),
            |&(x, t, c)| run::abs_sqrt_times_plus(x, t, c),
            |&(x, t, c)| x87::abs_sqrt_times_plus(x, t, c),
            eqf,
        );
        check(
            "add3",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f()),
            |&(a, b, c)| run::add3(a, b, c),
            |&(a, b, c)| x87::add3(a, b, c),
            eqf,
        );
        check(
            "neg_product_plus",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f()),
            |&(a, b, c)| run::neg_product_plus(a, b, c),
            |&(a, b, c)| x87::neg_product_plus(a, b, c),
            eqf,
        );
        check(
            "neg_mul",
            &ROUNDING,
            |g| (g.f(), g.f()),
            |&(a, b)| run::neg_mul(a, b),
            |&(a, b)| x87::neg_mul(a, b),
            eqf,
        );
        check(
            "abs_of",
            &ROUNDING,
            Rng::f,
            |&a| run::abs_of(a),
            |&a| x87::abs_of(a),
            eqf,
        );
        check(
            "square_times_over",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f()),
            |&(x, k, d)| run::square_times_over(x, k, d),
            |&(x, k, d)| x87::square_times_over(x, k, d),
            eqf,
        );
        check(
            "toward_one",
            &ROUNDING,
            |g| (g.f(), g.f()),
            |&(a, b)| run::toward_one(a, b),
            |&(a, b)| x87::toward_one(a, b),
            eqf,
        );
        check(
            "lerp_int_over2",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.i(), g.f(), g.f()),
            |&(a, b, k, i, d, w)| run::lerp_int_over2(a, b, k, i, d, w),
            |&(a, b, k, i, d, w)| x87::lerp_int_over2(a, b, k, i, d, w),
            eqf,
        );
        check(
            "weight_force",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f(), g.f()),
            |&(v, one, a, h, m)| run::weight_force(v, one, a, h, m),
            |&(v, one, a, h, m)| x87::weight_force(v, one, a, h, m),
            eqf,
        );
        check(
            "neg_product2_plus",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f()),
            |&(a, b, c, d)| run::neg_product2_plus(a, b, c, d),
            |&(a, b, c, d)| x87::neg_product2_plus(a, b, c, d),
            eqf,
        );
        check(
            "neg_mul2",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f()),
            |&(a, b, c)| run::neg_mul2(a, b, c),
            |&(a, b, c)| x87::neg_mul2(a, b, c),
            eqf,
        );
        check(
            "sub_mul",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f()),
            |&(a, b, c)| run::sub_mul(a, b, c),
            |&(a, b, c)| x87::sub_mul(a, b, c),
            eqf,
        );
        check(
            "abs_div_add_mul",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f()),
            |&(x, a, b, c)| run::abs_div_add_mul(x, a, b, c),
            |&(x, a, b, c)| x87::abs_div_add_mul(x, a, b, c),
            eqf,
        );
        check(
            "mul2_plus",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f()),
            |&(a, b, c, d)| run::mul2_plus(a, b, c, d),
            |&(a, b, c, d)| x87::mul2_plus(a, b, c, d),
            eqf,
        );
        check(
            "over_abs_plus_minus",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f()),
            |&(a, s, b, c)| run::over_abs_plus_minus(a, s, b, c),
            |&(a, s, b, c)| x87::over_abs_plus_minus(a, s, b, c),
            eqf,
        );
        check(
            "abs_fidiv",
            &ROUNDING,
            |g| (g.f(), g.i()),
            |&(x, n)| run::abs_fidiv(x, n),
            |&(x, n)| x87::abs_fidiv(x, n),
            eqf,
        );
        check(
            "int_fidiv",
            &ROUNDING,
            |g| (g.i(), g.i()),
            |&(a, b)| run::int_fidiv(a, b),
            |&(a, b)| x87::int_ratio(a, b),
            eqf,
        );
        check(
            "one_less_times_plus",
            &ROUNDING,
            |g| (g.f(), g.f()),
            |&(d, s)| run::one_less_times_plus(d, s),
            |&(d, s)| x87::toward_one(d, s),
            eqf,
        );
        check(
            "sub_mul_add",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f()),
            |&(a, b, c, d)| run::sub_mul_add(a, b, c, d),
            |&(a, b, c, d)| x87::sub_mul_add(a, b, c, d),
            eqf,
        );
        check(
            "lerp_table",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f(), g.f()),
            |&(s, a0, a1, c0, c1)| run::lerp_table(s, a0, a1, c0, c1),
            |&(s, a0, a1, c0, c1)| x87::lerp_table(s, a0, a1, c0, c1),
            eqf,
        );
        check(
            "div_then_mul",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f()),
            |&(a, b, c)| run::div_then_mul(a, b, c),
            |&(a, b, c)| x87::div_then_mul(a, b, c),
            eqf,
        );
        check(
            "sum_times_int_over_minus",
            &ROUNDING,
            |g| (g.f(), g.f(), g.i(), g.f()),
            |&(a, b, n, k)| run::sum_times_int_over_minus(a, b, n, k),
            |&(a, b, n, k)| x87::sum_times_int_over_minus(a, b, n, k),
            eqf,
        );
        check(
            "mul_div2",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f()),
            |&(a, b, c, d)| run::mul_div2(a, b, c, d),
            |&(a, b, c, d)| x87::mul_div2(a, b, c, d),
            eqf,
        );
        check(
            "int_mul_add",
            &ROUNDING,
            |g| {
                (
                    g.i(),
                    g.f(),
                    g.f(),
                    if g.below(2) == 0 { None } else { Some(g.f()) },
                )
            },
            |&(i, k, a, b)| run::int_mul_add(i, k, a, b),
            |&(i, k, a, b)| x87::int_mul_add(i, k, a, b),
            eqf,
        );
        check(
            "ext_div2",
            &ROUNDING,
            |g| (g.e80(), g.f(), g.f()),
            |&(e, a, b)| run::ext_div2(e, a, b),
            |&(e, a, b)| x87::ext_div2(e, a, b),
            eqf,
        );
        check(
            "neg_rounded",
            &ROUNDING,
            Rng::e80,
            |&x| run::neg_rounded(x),
            |&x| x87::neg_rounded(x),
            eqf,
        );
        check(
            "lever_share",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.f(), g.fw()),
            |&(a, d, b, c, z)| run::lever_share(a, d, b, c, z),
            |&(a, d, b, c, z)| {
                let (share, zero) = x87::lever_share(a, d, b, c, z);
                (share.to_bits(), zero)
            },
            |a, b| eqf(&a.0, &b.0) && a.1 == b.1,
        );
        // Zero sums: the equal-to-zero branch needs them.
        check(
            "lever_share (zeros)",
            &ROUNDING,
            |g| {
                (
                    g.f() & 0x8000_0000,
                    g.f() & 0x8000_0000,
                    g.f() & 0x8000_0000,
                    g.f() & 0x8000_0000,
                    g.f() & 0x8000_0000,
                )
            },
            |&(a, d, b, c, z)| run::lever_share(a, d, b, c, z),
            |&(a, d, b, c, z)| {
                let (share, zero) = x87::lever_share(a, d, b, c, z);
                (share.to_bits(), zero)
            },
            |a, b| eqf(&a.0, &b.0) && a.1 == b.1,
        );
        // The sums over the divisors: a list each.
        check(
            "sum_over",
            &ROUNDING,
            |g| {
                let t: Vec<u32> = (0..1 + g.below(5)).map(|_| g.f()).collect();
                let d: Vec<u32> = (0..g.below(4)).map(|_| g.f()).collect();
                (t, d)
            },
            |(t, d)| run::sum_over(t, d),
            |(t, d)| x87::sum_over(t, d),
            eqf,
        );
        check(
            "ramp",
            &ROUNDING,
            |g| {
                let mut t = [g.fw(), g.f(), g.f(), g.f()];
                // Now and then the ends equal or in order, so that every branch is seen.
                match g.below(4) {
                    0 => t[2] = t[0],
                    1 => {
                        let (lo, hi) = (g.pos(), g.pos());
                        t[0] = lo.min(hi);
                        t[2] = lo.max(hi);
                    }
                    _ => {}
                }
                let x = if g.below(3) == 0 { t[0] } else { g.fw() };
                (x, t, if g.below(4) == 0 { 0 } else { g.fw() })
            },
            |&(x, t, z)| run::ramp(x, t, z),
            |&(x, t, z)| x87::ramp(x, t, z),
            eqf,
        );
    }

    #[test]
    fn run_comparisons() {
        check(
            "compare_full",
            &ROUNDING,
            |g| (g.fw(), g.fw()),
            |&(a, b)| run::compare_full(a, b),
            |&(a, b)| x87::compare_full(a, b),
            exact,
        );
        check(
            "compare_full (equal)",
            &ROUNDING,
            |g| {
                let a = g.f();
                (a, a)
            },
            |&(a, b)| run::compare_full(a, b),
            |&(a, b)| x87::compare_full(a, b),
            exact,
        );
        check(
            "abs_compare",
            &ROUNDING,
            |g| (g.fw(), g.fw()),
            |&(a, b)| run::abs_compare(a, b),
            |&(a, b)| x87::abs_compare(a, b),
            exact,
        );
        check(
            "mul_div_status",
            &ROUNDING,
            |g| (g.f(), g.f(), g.f(), g.fw()),
            |&(a, b, c, k)| run::mul_div_status(a, b, c, k),
            |&(a, b, c, k)| x87::mul_div_status(a, b, c, k),
            exact,
        );
        check(
            "mul_status",
            &ROUNDING,
            |g| (g.f(), g.f(), g.fw()),
            |&(a, b, c)| run::mul_status(a, b, c),
            |&(a, b, c)| x87::mul_status(a, b, c),
            exact,
        );
        check(
            "compare_80",
            &ROUNDING,
            |g| (g.e80(), g.fw()),
            |&(e, b)| run::compare_80(e, b),
            |&(e, b)| x87::compare_80(e, b),
            exact,
        );
    }

    #[test]
    fn run_extended_results() {
        check(
            "length_80",
            &ROUNDING,
            |g| [g.f(), g.f(), g.f()],
            |v| run::length_80(v.as_ptr()),
            |v| x87::length_80(*v),
            eq80,
        );
        check(
            "scale_by_80",
            &ROUNDING,
            |g| ([g.f(), g.f(), g.f()], g.e80()),
            |&(v, s)| {
                let mut v = v;
                {
                    run::scale_by_80(v.as_mut_ptr(), s);
                };
                v
            },
            |&(v, s)| x87::scale_by_80(v, s),
            eqf3,
        );
        check(
            "less_plus_less_product_80",
            &ROUNDING,
            |g| {
                (
                    g.f(),
                    g.f(),
                    if g.below(2) == 0 { None } else { Some(g.f()) },
                    g.f(),
                    g.f(),
                )
            },
            |&(a, b, c, d, e)| run::less_plus_less_product_80(a, b, c, d, e),
            |&(a, b, c, d, e)| x87::less_plus_less_product_80(a, b, c, d, e),
            eq80,
        );
        check(
            "one_less_products",
            &ROUNDING,
            |g| {
                (0..g.below(4))
                    .map(|_| [g.f(), g.f(), g.f()])
                    .collect::<Vec<_>>()
            },
            |t| run::one_less_products(t),
            |t| x87::one_less_products(t),
            eq80,
        );
        check(
            "ext_times",
            &ROUNDING,
            |g| (g.e80(), g.f()),
            |&(e, k)| run::ext_times(e, k),
            |&(e, k)| x87::ext_times(e, k),
            eq80,
        );
        check(
            "int_80",
            &ROUNDING,
            |g| (g.i(), g.below(2) == 0),
            |&(a, n)| run::int_80(a, n),
            |&(a, n)| x87::int_80(a, n),
            eq80,
        );
        check(
            "gear_step_80",
            &ROUNDING,
            |g| (g.f(), g.f(), g.i(), g.f()),
            |&(t, l, n, o)| run::gear_step_80(t, l, n, o),
            |&(t, l, n, o)| x87::gear_step_80(t, l, n, o),
            eq80,
        );
        check(
            "ext_add",
            &ROUNDING,
            |g| (g.e80(), g.e80()),
            |&(a, b)| run::ext_add(a, b),
            |&(a, b)| x87::ext_add(a, b),
            eq80,
        );
        check(
            "ints_diff_80",
            &ROUNDING,
            |g| (g.i(), g.i()),
            |&(a, b)| run::ints_diff_80(a, b),
            |&(a, b)| x87::ints_diff_80(a, b),
            eq80,
        );
        check(
            "div_80",
            &ROUNDING,
            |g| (g.f(), g.f()),
            |&(a, b)| run::div_80(a, b),
            |&(a, b)| x87::div_80(a, b),
            eq80,
        );
    }

    #[test]
    fn run_face_shares() {
        // The SSE path's `rcpss` is the tables' (`arch::rcpss`): compared only where this
        // processor's is the same.
        let sse_too = crate::arch::estimates_are_the_processors();
        for sse in [false, true] {
            if sse && !sse_too {
                eprintln!(
                    "SKIPPED: face_shares' SSE path, this processor's rcpss is not the tables'"
                );
                continue;
            }
            check_all(
                "face_shares",
                |g| {
                    (
                        [
                            g.i() as u32,
                            g.i() as u32,
                            g.i() as u32,
                            g.i() as u32,
                            g.i() as u32,
                            g.i() as u32,
                            g.f(),
                        ],
                        sse,
                    )
                },
                |&(i, sse)| {
                    let mut o = [0xAAAA_AAAAu32; 8];
                    unsafe { run::face_shares(&i, &mut o, sse) };
                    o
                },
                |&(i, sse)| {
                    let mut o = [0xAAAA_AAAAu32; 8];
                    {
                        x87::face_shares(&i, &mut o, sse);
                    };
                    o
                },
                |a, b| {
                    (0..8).all(|k| {
                        if matches!(k, 4 | 6) {
                            a[k] == b[k]
                        } else {
                            eqf(&a[k], &b[k])
                        }
                    })
                },
            );
        }
    }

    // ---- the transcendental ones: within an ulp (floats) or two (doubles) ----

    #[test]
    fn run_sine_cosine_of() {
        check(
            "sine_cosine_of",
            &ROUNDING,
            Rng::angle,
            |&a| run::sine_cosine_of(a),
            |&a| x87::sine_cosine_of(a),
            |a, b| near_f(&a.0, &b.0, 1) && near_f(&a.1, &b.1, 1),
        );
    }

    #[test]
    fn run_patan() {
        check(
            "patan",
            &ROUNDING,
            |g| (g.f(), g.f()),
            |&(y, x)| run::patan(y, x),
            |&(y, x)| x87::patan(y, x),
            |a, b| near_f(a, b, 1),
        );
    }

    /// `a / cos b`: `fcos` reduces the argument with a 66-bit pi, so near a zero of the
    /// cosine its value is off by about `b 2^-66` (absolute); libm's cosine is the true
    /// one. The test allows that error (relative to the cosine) besides the 2 ulp, over
    /// the angles the game gives it (the pitch, within +-pi/2).
    #[test]
    fn run_over_cos_80() {
        let allow = |b: u32| {
            4.0 * (f64::from(f32::from_bits(b)).abs() + 1.0) * 2f64.powi(-66)
                / f64::from(f32::from_bits(b)).cos().abs()
        };
        check(
            "over_cos_80",
            &ROUNDING,
            |g| {
                (
                    g.f(),
                    match g.below(4) {
                        0 => 0x3FC9_0FDB, // pi / 2
                        1 => 0xBFC9_0FDB,
                        _ => (g.u32() as f32 / u32::MAX as f32 * 3.2 - 1.6).to_bits(),
                    },
                )
            },
            |&(a, b)| (run::over_cos_80(a, b), allow(b)),
            |&(a, b)| (x87::over_cos_80(a, b), allow(b)),
            |x, y| {
                near_80(&x.0, &y.0, 2) || {
                    let (p, q) = (f80_to_f64(x.0), f80_to_f64(y.0));
                    (p - q).abs() <= x.1 * p.abs()
                }
            },
        );
    }

    #[test]
    fn run_clamped_asin_80() {
        let call = |f: unsafe fn([u8; 10], *mut u32, *mut u32) -> [u8; 10],
                    x: [u8; 10],
                    s0: u32,
                    l0: u32| {
            let (mut s, mut l) = (s0, l0);
            let r = unsafe { f(x, &raw mut s, &raw mut l) };
            (r, s, l)
        };
        check(
            "clamped_asin_80",
            &ROUNDING,
            |g| {
                let x = match g.below(3) {
                    0 => f64_to_f80((g.u32() as f64 / u32::MAX as f64 - 0.5) * 2.4),
                    1 => g.e80(),
                    _ => f64_to_f80(f64::from(f32::from_bits(g.f())).clamp(-1.0, 1.0)),
                };
                let l = match g.below(4) {
                    0 => g.fw(),
                    1 => 0x3F80_0000,
                    _ => (g.u32() as f32 / u32::MAX as f32 * 1.5).to_bits(),
                };
                (x, g.f(), l)
            },
            |&(x, s, l)| call(run::clamped_asin_80, x, s, l),
            |&(x, s, l)| {
                let (mut s, mut l) = (s, l);
                (x87::clamped_asin_80(x, &mut s, &mut l), s, l)
            },
            |a, b| near_80(&a.0, &b.0, 2) && eqf(&a.1, &b.1) && eqf(&a.2, &b.2),
        );
    }

    // ---- normalise_vector's pieces ----

    #[test]
    fn norm_squares_and_scaling() {
        check(
            "normalise squares_sum",
            &ROUNDING,
            |g| [g.f(), g.f(), g.f()],
            |&v| norm::squares_sum(v),
            |&v| normalise_vector::squares_sum(v),
            eqf,
        );
        check(
            "normalise scaled_by_inverse",
            &ROUNDING,
            |g| ([g.f(), g.f(), g.f()], g.pos(), g.pos()),
            |&(v, r, e)| norm::scaled_by_inverse(v, r, e),
            |&(v, r, e)| normalise_vector::scaled_by_inverse(v, r, e),
            |a, b| eqf3(&a.0, &b.0) && eqf(&a.1, &b.1),
        );
    }
}
