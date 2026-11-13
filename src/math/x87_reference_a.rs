//! The x87 assembly versions of the helpers in `math/x87.rs` up to its "part B" line, kept as the reference the
//! plain Rust versions are tested against (phase 6, 2026-10-24).

use crate::math::x87::x87;

/// `fld a; fadd b; fstp`.
pub(super) fn add(a: u32, b: u32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fld dword ptr [{v}]", "fadd dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fsub b; fstp`.
pub(super) fn sub(a: u32, b: u32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fstp`: a whole number as a float.
pub(super) fn from_int(a: i32) -> u32 {
    let mut out = 0u32;
    x87!("fild dword ptr [{a}]", "fstp dword ptr [{o}]"; a = &raw const a, o = &raw mut out);
    out
}

/// `fld a; fmul b; fstp`.
pub(super) fn mul(a: u32, b: u32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld1; fdiv a; fmul b; fstp`.
pub(super) fn reciprocal_times(a: u32, b: u32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fld1", "fdiv dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld x; fsub a; fsub b; fchs; fadd b; fadd a; fstp`: x mirrored about a + b.
pub(super) fn mirrored(x: u32, a: u32, b: u32) -> u32 {
    let (v, mut out) = ([x, a, b], 0u32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fsub dword ptr [{v} + 8]", "fchs",
        "fadd dword ptr [{v} + 8]", "fadd dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fstp`: a float as it comes back from a register.
pub(super) fn loaded(a: u32) -> u32 {
    let mut out = 0u32;
    x87!("fld dword ptr [{a}]", "fstp dword ptr [{o}]"; a = &raw const a, o = &raw mut out);
    out
}

/// `fld a; fistp`: the whole number, rounded as the x87 is set to round.
pub(super) fn to_int(a: u32) -> i32 {
    let mut out = 0i32;
    x87!("fld dword ptr [{a}]", "fistp dword ptr [{o}]"; a = &raw const a, o = &raw mut out);
    out
}

/// `fld a; fmul b; fistp`: the product as a whole number.
pub(super) fn mul_to_int(a: u32, b: u32) -> i32 {
    let (v, mut out) = ([a, b], 0i32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fild b; fdivp st(1), st; fstp`: a over b, two whole numbers, as a float.
pub(super) fn int_ratio(a: i32, b: i32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fild dword ptr [{v}]", "fild dword ptr [{v} + 4]", ".byte 0xDE, 0xF9",
        "fstp dword ptr [{o}]"; v = v.as_ptr(), o = &raw mut out);
    out
}

/// The same and `fld1; fsubrp st(1), st`: 1 less a over b.
pub(super) fn one_less_int_ratio(a: i32, b: i32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fild dword ptr [{v}]", "fild dword ptr [{v} + 4]", ".byte 0xDE, 0xF9",
        ".byte 0xD9, 0xE8", ".byte 0xDE, 0xE1", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a[0]; fld b[0]; fmulp; fld a[1]; fld b[4]; fmulp; fld a[2]; fld b[8]; fmulp;
/// faddp; faddp; fstp out`: a row of one matrix times a column of another, the last
/// two products added first.
pub(super) fn row_times_column(a: *const u32, b: *const u32, out: *mut u32) {
    x87!(
        "fld dword ptr [{a}]", "fld dword ptr [{b}]", ".byte 0xDE, 0xC9",
        "fld dword ptr [{a} + 4]", "fld dword ptr [{b} + 0x10]", ".byte 0xDE, 0xC9",
        "fld dword ptr [{a} + 8]", "fld dword ptr [{b} + 0x20]", ".byte 0xDE, 0xC9",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1",
        "fstp dword ptr [{o}]";
        a = a, b = b, o = out
    );
}

/// `fld a; fadd out; fstp out`.
pub(super) fn add_into(a: *const u32, out: *mut u32) {
    x87!("fld dword ptr [{a}]", "fadd dword ptr [{o}]", "fstp dword ptr [{o}]"; a = a, o = out);
}

/// `fild a; fld1; fadd st, st` four times`; fdivp st(1), st; fstp`: a whole number
/// over sixteen.
pub(super) fn int_over_16(a: i32) -> u32 {
    let mut out = 0u32;
    x87!(
        "fild dword ptr [{a}]", "fld1",
        ".byte 0xD8, 0xC0", ".byte 0xD8, 0xC0", ".byte 0xD8, 0xC0", ".byte 0xD8, 0xC0",
        ".byte 0xDE, 0xF9",
        "fstp dword ptr [{o}]";
        a = &raw const a, o = &raw mut out
    );
    out
}

/// `fild a; fmul st, st; fild b; fmul st, st; faddp; fsqrt; fistp`: the length of two
/// whole numbers at right angles, as a whole number.
pub(super) fn int_hypot(a: i32, b: i32) -> i32 {
    let (v, mut out) = ([a, b], 0i32);
    x87!(
        "fild dword ptr [{v}]", ".byte 0xD8, 0xC8",
        "fild dword ptr [{v} + 4]", ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1", "fsqrt",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out
    );
    out
}

/// `fld a; fdiv b; fstp`.
pub(super) fn div(a: u32, b: u32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fld dword ptr [{v}]", "fdiv dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul b; fld c; fmul d; fsubp st(1), st; fstp`: a times b less c times d.
pub(super) fn product_difference(a: u32, b: u32, c: u32, d: u32) -> u32 {
    let (v, mut out) = ([a, b, c, d], 0u32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]",
        "fld dword ptr [{v} + 8]", "fmul dword ptr [{v} + 12]", ".byte 0xDE, 0xE9",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// The same with `fchs` before the `fstp`.
pub(super) fn negated_product_difference(a: u32, b: u32, c: u32, d: u32) -> u32 {
    let (v, mut out) = ([a, b, c, d], 0u32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]",
        "fld dword ptr [{v} + 8]", "fmul dword ptr [{v} + 12]", ".byte 0xDE, 0xE9",
        "fchs", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld v[0]; fmul st, st; fld v[1]; fmul st, st; fld v[2]; fmul st, st; faddp; faddp;
/// fsqrt; fstp`: the length of three floats.
pub(super) fn length(v: *const u32) -> u32 {
    let mut out = 0u32;
    x87!("fld dword ptr [{v}]", ".byte 0xD8, 0xC8",
        "fld dword ptr [{v} + 4]", ".byte 0xD8, 0xC8",
        "fld dword ptr [{v} + 8]", ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fsqrt",
        "fstp dword ptr [{o}]";
        v = v, o = &raw mut out);
    out
}

/// `fld a; fcomp b; fnstsw ax; and ah, 40h`: whether the two compare as equal, which
/// they also do when one is no number.
pub(super) fn equal_or_unordered(a: u32, b: u32) -> bool {
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
    status & 0x4000 != 0
}

/// `fild angle; fdiv a; fdiv a; fdiv b; fsincos; fstp cosine; fstp sine`: the sine and
/// the cosine of a whole number divided three times.
pub(super) fn sine_cosine(angle: i32, a: u32, b: u32) -> (u32, u32) {
    let (v, mut out) = ([angle as u32, a, b], [0u32; 2]);
    x87!("fild dword ptr [{v}]", "fdiv dword ptr [{v} + 4]", "fdiv dword ptr [{v} + 4]",
        "fdiv dword ptr [{v} + 8]", "fsincos",
        "fstp dword ptr [{o} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    (out[0], out[1])
}

/// `fild a; fdiv d; fsqrt; fstp`: the root of a whole number over a float.
pub(super) fn int_over_root(a: i32, d: u32) -> u32 {
    let (v, mut out) = ([a as u32, d], 0u32);
    x87!("fild dword ptr [{v}]", "fdiv dword ptr [{v} + 4]", "fsqrt", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fsqrt; fstp`.
pub(super) fn sqrt(a: u32) -> u32 {
    let mut out = 0u32;
    x87!("fld dword ptr [{a}]", "fsqrt", "fstp dword ptr [{o}]";
        a = &raw const a, o = &raw mut out);
    out
}

/// `fild n; fldpi; fmulp st(1), st; fild d; fdiv st(1), st; fstp (away); fsin; fstp`:
/// the sine of n times pi over d.
pub(super) fn sine_of_pi_fraction(n: i32, d: i32) -> u32 {
    let (v, mut out) = ([n, d], [0u32; 2]);
    x87!("fild dword ptr [{v}]", "fldpi", ".byte 0xDE, 0xC9",
        "fild dword ptr [{v} + 4]", ".byte 0xDC, 0xF9", "fstp dword ptr [{o} + 4]",
        "fsin", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    out[0]
}

/// `fld a; fld b; fdivp st(1), st; fild m; fmulp st(1), st; fild n; fmulp st(1), st;
/// fistp`: a over b times two whole numbers, as a whole number.
pub(super) fn ratio_scaled_to_int(a: u32, b: u32, m: i32, n: i32) -> i32 {
    let (v, mut out) = ([a, b, m as u32, n as u32], 0i32);
    x87!("fld dword ptr [{v}]", "fld dword ptr [{v} + 4]", ".byte 0xDE, 0xF9",
        "fild dword ptr [{v} + 8]", ".byte 0xDE, 0xC9",
        "fild dword ptr [{v} + 12]", ".byte 0xDE, 0xC9",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild y; fild x; fpatan; fldpi; fdivp st(1), st; fmul a; fmul b; fistp`: the angle of
/// (x, y) over pi, times two floats, as a whole number.
pub(super) fn angle_scaled_to_int(y: i32, x: i32, a: u32, b: u32) -> i32 {
    let (v, mut out) = ([y as u32, x as u32, a, b], 0i32);
    x87!("fild dword ptr [{v}]", "fild dword ptr [{v} + 4]", "fpatan", "fldpi",
        ".byte 0xDE, 0xF9", "fmul dword ptr [{v} + 8]", "fmul dword ptr [{v} + 12]",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fmul b; fstp`: a whole number times a float.
pub(super) fn int_times(a: i32, b: u32) -> u32 {
    let (v, mut out) = ([a as u32, b], 0u32);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fdiv b; fstp`: a whole number over a float.
pub(super) fn int_over(a: i32, b: u32) -> u32 {
    let (v, mut out) = ([a as u32, b], 0u32);
    x87!("fild dword ptr [{v}]", "fdiv dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fsub b; fstp`: a whole number less a float.
pub(super) fn int_less(a: i32, b: u32) -> u32 {
    let (v, mut out) = ([a as u32, b], 0u32);
    x87!("fild dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld1; fdiv a; fstp`.
pub(super) fn reciprocal(a: u32) -> u32 {
    let mut out = 0u32;
    x87!("fld1", "fdiv dword ptr [{a}]", "fstp dword ptr [{o}]";
        a = &raw const a, o = &raw mut out);
    out
}

/// `fild a; fmul b; fild c; fmulp st(1), st; fmul d; fstp`: two whole numbers and two
/// floats multiplied.
pub(super) fn int_float_int_float(a: i32, b: u32, c: i32, d: u32) -> u32 {
    let (v, mut out) = ([a as u32, b, c as u32, d], 0u32);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fild dword ptr [{v} + 8]",
        ".byte 0xDE, 0xC9", "fmul dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul b; fdiv c; fdiv d; fistp`.
pub(super) fn mul_div_div_to_int(a: u32, b: u32, c: u32, d: u32) -> i32 {
    let (v, mut out) = ([a, b, c, d], 0i32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fdiv dword ptr [{v} + 8]",
        "fdiv dword ptr [{v} + 12]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `race::normalise_work_vector`'s sequence: x, y, z squared and summed (`faddp` twice), `fsqrt`,
/// `fcom` with `zero`; unless equal, each of the three divided by the root still on the stack
/// (80 bits, not rounded first: `fdiv st, st(1)`) and stored back; then the root stored.
/// Returns the root's bits and whether it compared equal.
pub(super) unsafe fn normalise_in_place(v: *mut u32, zero: u32) -> (u32, bool) {
    let z = zero;
    let mut length = 0u32;
    let status: u16;
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{v}]", ".byte 0xD8, 0xC8",
            "fld dword ptr [{v} + 4]", ".byte 0xD8, 0xC8",
            "fld dword ptr [{v} + 8]", ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fsqrt",
            "fcom dword ptr [{z}]",
            "fnstsw ax",
            "test ah, 0x40",
            "jnz 2f",
            "fld dword ptr [{v}]", ".byte 0xD8, 0xF1", "fstp dword ptr [{v}]",
            "fld dword ptr [{v} + 4]", ".byte 0xD8, 0xF1", "fstp dword ptr [{v} + 4]",
            "fld dword ptr [{v} + 8]", ".byte 0xD8, 0xF1", "fstp dword ptr [{v} + 8]",
            "2:",
            "fstp dword ptr [{o}]",
            v = in(reg) v,
            z = in(reg) &raw const z,
            o = in(reg) &raw mut length,
            out("ax") status,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
    (length, status & 0x4000 != 0)
}

/// `fild a; fild b; fmulp; fild c; fild d; fmulp; fsubp st(1), st; fstp`: a times b less c
/// times d, whole numbers, rounded once to a float.
pub(super) fn int_product_difference(a: i32, b: i32, c: i32, d: i32) -> u32 {
    let (v, mut out) = ([a, b, c, d], 0u32);
    x87!("fild dword ptr [{v}]", "fild dword ptr [{v} + 4]", ".byte 0xDE, 0xC9",
        "fild dword ptr [{v} + 8]", "fild dword ptr [{v} + 12]", ".byte 0xDE, 0xC9",
        ".byte 0xDE, 0xE9", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fdiv b; fstp`: a whole number divided by a float, rounded once.
pub(super) fn int_div(a: i32, b: u32) -> u32 {
    let (ia, fb, mut out) = (a, b, 0u32);
    x87!("fild dword ptr [{a}]", "fdiv dword ptr [{b}]", "fstp dword ptr [{o}]";
        a = &raw const ia, b = &raw const fb, o = &raw mut out);
    out
}

/// `fild a; fdiv b; fdiv c; fadd st, st; fstp`: a whole number over two floats, doubled,
/// rounded once.
pub(super) fn int_div_div_doubled(a: i32, b: u32, c: u32) -> u32 {
    let (ia, v, mut out) = (a, [b, c], 0u32);
    x87!("fild dword ptr [{a}]", "fdiv dword ptr [{v}]", "fdiv dword ptr [{v} + 4]",
        "fadd st(0), st(0)", "fstp dword ptr [{o}]";
        a = &raw const ia, v = v.as_ptr(), o = &raw mut out);
    out
}

/// `math::rotate_scratch_about_y`'s turn, its stack steps as the original's bytes: x' = x cos +
/// z sin, z' = z cos - x sin, each rounded once, written back over x and z.
pub(super) fn turn_in_place(sine: *const u32, cosine: *const u32, x: *mut u32, z: *mut u32) {
    x87!(
        "fld dword ptr [{s}]", "fld dword ptr [{c}]", "fld dword ptr [{x}]",
        ".byte 0xD8, 0xC9", "fld dword ptr [{z}]", ".byte 0xD8, 0xCB", ".byte 0xDE, 0xC1",
        ".byte 0xD9, 0xCA", "fld dword ptr [{x}]", ".byte 0xDE, 0xC9", ".byte 0xD9, 0xC9",
        "fld dword ptr [{z}]", ".byte 0xDE, 0xC9", ".byte 0xDE, 0xE1",
        "fstp dword ptr [{z}]", "fstp dword ptr [{x}]";
        s = sine, c = cosine, x = x, z = z
    );
}

/// `fild a; fdiv b; fmul c; fidiv d; fmul e; fadd f; fstp`: `race::add_surface_share`'s
/// sum.
pub(super) fn surface_share(a: i32, b: u32, c: u32, d: i32, e: u32, f: u32) -> u32 {
    let (v, mut out) = ([a as u32, b, c, d as u32, e, f], 0u32);
    x87!("fild dword ptr [{v}]", "fdiv dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fidiv dword ptr [{v} + 12]", "fmul dword ptr [{v} + 16]", "fadd dword ptr [{v} + 20]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild y; fild x; fpatan; fldpi; fdivp st(1), st; fmul a; fmul b; fmul c; fistp`.
pub(super) fn angle_scaled3_to_int(y: i32, x: i32, a: u32, b: u32, c: u32) -> i32 {
    let (v, mut out) = ([y as u32, x as u32, a, b, c], 0i32);
    x87!("fild dword ptr [{v}]", "fild dword ptr [{v} + 4]", "fpatan", "fldpi",
        ".byte 0xDE, 0xF9", "fmul dword ptr [{v} + 8]", "fmul dword ptr [{v} + 12]",
        "fmul dword ptr [{v} + 16]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild y; fild x; fpatan; fldpi; fdivp st(1), st; fmul a; fmul a; fmul a; fmul b;
/// fmul b; fistp`.
pub(super) fn angle_scaled5_to_int(y: i32, x: i32, a: u32, b: u32) -> i32 {
    let (v, mut out) = ([y as u32, x as u32, a, b], 0i32);
    x87!("fild dword ptr [{v}]", "fild dword ptr [{v} + 4]", "fpatan", "fldpi",
        ".byte 0xDE, 0xF9", "fmul dword ptr [{v} + 8]", "fmul dword ptr [{v} + 8]",
        "fmul dword ptr [{v} + 8]", "fmul dword ptr [{v} + 12]", "fmul dword ptr [{v} + 12]",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// The length of (a, b) over that of (c, d), whole numbers, as a float: each `fild; fmul
/// st, st`, `faddp`, `fsqrt`, then `fdivp st(1), st; fstp`.
pub(super) fn hypot_ratio(a: i32, b: i32, c: i32, d: i32) -> u32 {
    let (v, mut out) = ([a, b, c, d], 0u32);
    x87!(
        "fild dword ptr [{v}]", ".byte 0xD8, 0xC8", "fild dword ptr [{v} + 4]", ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1", "fsqrt",
        "fild dword ptr [{v} + 8]", ".byte 0xD8, 0xC8", "fild dword ptr [{v} + 12]", ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1", "fsqrt", ".byte 0xDE, 0xF9", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out
    );
    out
}

/// `fild a; fmul b; fistp`: a whole number times a float, as a whole number.
pub(super) fn int_times_to_int(a: i32, b: u32) -> i32 {
    let (v, mut out) = ([a as u32, b], 0i32);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fdiv b; fsub c; fstp`: a whole number over a float, less a float, rounded once.
pub(super) fn int_div_less(a: i32, b: u32, c: u32) -> u32 {
    let (v, mut out) = ([a as u32, b, c], 0u32);
    x87!("fild dword ptr [{v}]", "fdiv dword ptr [{v} + 4]", "fsub dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fldz`, then `count` times `fld st; fsin; fstp [out]; fadd step` (the angle summed in the
/// register, never rounded to a float), then `ffree st; fincstp`.
pub(super) unsafe fn sine_table(step: u32, out: *mut u32, count: u32) {
    if count == 0 {
        return;
    }
    let s = step;
    unsafe {
        core::arch::asm!(
            "fldz",
            "2:",
            "fld st(0)",
            "fsin",
            "fstp dword ptr [{o}]",
            "add {o}, 4",
            "fadd dword ptr [{s}]",
            "dec {n:e}",
            "jnz 2b",
            "ffree st(0)",
            "fincstp",
            o = inout(reg) out => _,
            n = inout(reg) count => _,
            s = in(reg) &raw const s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
}

/// `fld1; fidiv a; fst first; fmul b; fstp second`: one over a whole number, and that
/// (unrounded) times a float.
pub(super) fn reciprocal_int_and_times(a: i32, b: u32) -> (u32, u32) {
    let (v, mut out) = ([a as u32, b], [0u32; 2]);
    x87!("fld1", "fidiv dword ptr [{v}]", "fst dword ptr [{o}]", "fmul dword ptr [{v} + 4]",
        "fstp dword ptr [{o} + 4]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    (out[0], out[1])
}

/// `fld a; fmul b; fst first; fdiv c; fstp second`: a times b, and that (unrounded) over c.
pub(super) fn mul_and_div(a: u32, b: u32, c: u32) -> (u32, u32) {
    let (v, mut out) = ([a, b, c], [0u32; 2]);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fst dword ptr [{o}]",
        "fdiv dword ptr [{v} + 8]", "fstp dword ptr [{o} + 4]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    (out[0], out[1])
}

/// `fld a; fsub b; fdiv c; fmul d; fstp`.
pub(super) fn sub_div_mul(a: u32, b: u32, c: u32, d: u32) -> u32 {
    let (v, mut out) = ([a, b, c, d], 0u32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fdiv dword ptr [{v} + 8]",
        "fmul dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fsub b; fld c; fsub d; fdivp st(1), st; fmul e; fstp`: (a - b) / (c - d) * e.
pub(super) fn diff_ratio_mul(a: u32, b: u32, c: u32, d: u32, e: u32) -> u32 {
    let (v, mut out) = ([a, b, c, d, e], 0u32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fld dword ptr [{v} + 8]",
        "fsub dword ptr [{v} + 12]", ".byte 0xDE, 0xF9", "fmul dword ptr [{v} + 16]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fsub b; fmul m; fadd b; fmul n; fsub less; fstp`: b + (a - b) m, times n, less
/// `less` (0.0 for none), rounded once.
pub(super) fn lerp_scaled_less(a: u32, b: u32, m: u32, n: u32, less: Option<u32>) -> u32 {
    let (v, mut out) = ([a, b, m, n, less.unwrap_or(0)], 0u32);
    if less.is_some() {
        x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
            "fadd dword ptr [{v} + 4]", "fmul dword ptr [{v} + 12]", "fsub dword ptr [{v} + 16]",
            "fstp dword ptr [{o}]";
            v = v.as_ptr(), o = &raw mut out);
    } else {
        x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
            "fadd dword ptr [{v} + 4]", "fmul dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
            v = v.as_ptr(), o = &raw mut out);
    }
    out
}

/// `fld a; fsub b; fchs; fstp`.
pub(super) fn negated_sub(a: u32, b: u32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fchs", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fmul x; fild b; fmul y; fild c; fmul z; faddp st(1), st; faddp st(1), st;
/// fistp`: a x + (b y + c z), whole numbers out.
pub(super) fn weighted3_to_int(a: i32, b: i32, c: i32, x: u32, y: u32, z: u32) -> i32 {
    let (v, mut out) = ([a as u32, b as u32, c as u32, x, y, z], 0i32);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 12]", "fild dword ptr [{v} + 4]",
        "fmul dword ptr [{v} + 16]", "fild dword ptr [{v} + 8]", "fmul dword ptr [{v} + 20]",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fmul b; fadd c; fstp`.
pub(super) fn int_times_plus(a: i32, b: u32, c: u32) -> u32 {
    let (v, mut out) = ([a as u32, b, c], 0u32);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild last; fisub first; fidiv by; fldz`, then for each of `count` whole numbers at
/// `values`: `fild; fsub st, st(1); fistp` (less the sum), `fadd st, st(1)` (the sum on by
/// the step), both kept in the registers; then `ffree st; fincstp` twice.
pub(super) unsafe fn ramp_down(values: *mut i32, count: u32, last: i32, first: i32, by: i32) {
    if count == 0 {
        return;
    }
    let v = [last, first, by];
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{v}]",
            "fisub dword ptr [{v} + 4]",
            "fidiv dword ptr [{v} + 8]",
            "fldz",
            "2:",
            "fild dword ptr [{p}]",
            ".byte 0xD8, 0xE1",
            "fistp dword ptr [{p}]",
            ".byte 0xD8, 0xC1",
            "add {p}, 4",
            "dec {n:e}",
            "jnz 2b",
            "ffree st(0)",
            "fincstp",
            "ffree st(0)",
            "fincstp",
            p = inout(reg) values => _,
            n = inout(reg) count => _,
            v = in(reg) v.as_ptr(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
}

/// `fild a; fisub b; fstp`: the difference of two whole numbers, rounded once.
pub(super) fn int_less_int(a: i32, b: i32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fild dword ptr [{v}]", "fisub dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fisub b; fdiv c; fstp`.
pub(super) fn int_less_int_over(a: i32, b: i32, c: u32) -> u32 {
    let (v, mut out) = ([a as u32, b as u32, c], 0u32);
    x87!("fild dword ptr [{v}]", "fisub dword ptr [{v} + 4]", "fdiv dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fsub b; fdiv c; fstp`.
pub(super) fn sub_div(a: u32, b: u32, c: u32) -> u32 {
    let (v, mut out) = ([a, b, c], 0u32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fdiv dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fsin; fmul b; fadd c; fistp`.
pub(super) fn sine_times_plus_to_int(a: u32, b: u32, c: u32) -> i32 {
    let (v, mut out) = ([a, b, c], 0i32);
    x87!("fld dword ptr [{v}]", "fsin", "fmul dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul st, st; fchs; fmul b; fadd c; fistp`: c - a² b.
pub(super) fn negated_square_times_plus_to_int(a: u32, b: u32, c: u32) -> i32 {
    let (v, mut out) = ([a, b, c], 0i32);
    x87!("fld dword ptr [{v}]", ".byte 0xD8, 0xC8", "fchs", "fmul dword ptr [{v} + 4]",
        "fadd dword ptr [{v} + 8]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild x; fmul st, st; fild y; fmul st, st; fild z; fmul st, st; faddp; faddp; fsqrt;
/// fiadd sum; fistp`: the sum plus the length of (x, y, z).
pub(super) fn length_plus_int(x: i32, y: i32, z: i32, sum: i32) -> i32 {
    let (v, mut out) = ([x, y, z, sum], 0i32);
    x87!("fild dword ptr [{v}]", ".byte 0xD8, 0xC8", "fild dword ptr [{v} + 4]",
        ".byte 0xD8, 0xC8", "fild dword ptr [{v} + 8]", ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fsqrt", "fiadd dword ptr [{v} + 12]",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fidiv b; fmul c; fsubr c; fistp`: c less a / b of c, a whole number.
pub(super) fn share_left_to_int(a: i32, b: i32, c: u32) -> i32 {
    let (v, mut out) = ([a as u32, b as u32, c], 0i32);
    x87!("fild dword ptr [{v}]", "fidiv dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fsubr dword ptr [{v} + 8]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}
pub(super) fn row_times_column_3(a: *const u32, b: *const u32, out: *mut u32) {
    x87!(
        "fld dword ptr [{a}]", "fld dword ptr [{b}]", ".byte 0xDE, 0xC9",
        "fld dword ptr [{a} + 4]", "fld dword ptr [{b} + 0xC]", ".byte 0xDE, 0xC9",
        "fld dword ptr [{a} + 8]", "fld dword ptr [{b} + 0x18]", ".byte 0xDE, 0xC9",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1",
        "fstp dword ptr [{o}]";
        a = a, b = b, o = out
    );
}

/// `fild z; fild y; fild x`, then x m0 + (y m1 + z m2) and x m3 + (y m4 + z m5) rounded
/// to `out` and `out + 4`, and y m7 + (x m6 + z m8) to `out + 8` (the original's order:
/// `fmul; fxch; fmul; fxch; fmul; faddp; faddp`). A point of three whole numbers turned
/// by a 3 x 3 matrix; `out` may be `p`.
pub(super) fn turn_int_point(p: *const i32, m: *const u32, out: *mut i32) {
    x87!(
        "fild dword ptr [{p} + 8]", "fild dword ptr [{p} + 4]", "fild dword ptr [{p}]",
        "fld st(0)", "fmul dword ptr [{m}]",
        "fld st(2)", "fmul dword ptr [{m} + 4]",
        "fld st(4)", "fmul dword ptr [{m} + 8]",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1",
        "fistp dword ptr [{o}]",
        "fld st(0)", "fmul dword ptr [{m} + 0xC]",
        "fld st(2)", "fmul dword ptr [{m} + 0x10]",
        "fld st(4)", "fmul dword ptr [{m} + 0x14]",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1",
        "fistp dword ptr [{o} + 4]",
        "fmul dword ptr [{m} + 0x18]",
        "fxch st(1)", "fmul dword ptr [{m} + 0x1C]",
        "fxch st(2)", "fmul dword ptr [{m} + 0x20]",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1",
        "fistp dword ptr [{o} + 8]";
        p = p, m = m, o = out
    );
}

/// `fld a; fmul b; fmul c; fstp`.
pub(super) fn mul2(a: u32, b: u32, c: u32) -> u32 {
    let mut out = 0u32;
    x87!(
        "fld dword ptr [{a}]", "fmul dword ptr [{b}]", "fmul dword ptr [{c}]",
        "fstp dword ptr [{o}]";
        a = &raw const a, b = &raw const b, c = &raw const c, o = &raw mut out
    );
    out
}

/// `fld1; fild i; fsub a; fmul b; fmul st, st; fsubr st, st(1); fsqrt; fsubp st(1), st`,
/// negated (`fchs`) when `negate`: 1 - sqrt(1 - ((i - a) b)^2), a quarter circle's curve.
pub(super) fn circle_curve(i: i32, a: u32, b: u32, negate: bool) -> u32 {
    let mut out = 0u32;
    let n = negate as u32;
    unsafe {
        core::arch::asm!(
            "fld1", "fild dword ptr [{i}]", "fsub dword ptr [{a}]", "fmul dword ptr [{b}]",
            ".byte 0xD8, 0xC8", ".byte 0xD8, 0xE9", "fsqrt", ".byte 0xDE, 0xE9",
            "test {n}, {n}", "jz 2f", "fchs", "2:",
            "fstp dword ptr [{o}]",
            i = in(reg) &raw const i, a = in(reg) &raw const a, b = in(reg) &raw const b, n = in(reg) n, o = in(reg) &raw mut out,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
        );
    }
    out
}

/// `fldz`, then `count` times `fld st; fsincos; fmul r; fxch; fmul r; fxch; fstp [out+4];
/// fstp [out]; fldpi; fidiv c; fmul d; faddp` (out 12 on each time), then `ffree st;
/// fincstp`: points round a circle of radius `r`, the angle added up in the registers.
/// `count` at least 1.
pub(super) unsafe fn circle_points(
    r: *const u32,
    out: *mut u32,
    count: u32,
    c: *const i32,
    d: *const u32,
) {
    unsafe {
        core::arch::asm!(
            "fldz",
            "2:",
            "fld st(0)", "fsincos",
            "fmul dword ptr [{r}]", "fxch st(1)", "fmul dword ptr [{r}]", "fxch st(1)",
            "fstp dword ptr [{o} + 4]", "fstp dword ptr [{o}]",
            "fldpi", "fidiv dword ptr [{c}]", "fmul dword ptr [{d}]", ".byte 0xDE, 0xC1",
            "add {o}, 12", "dec {n}", "jnz 2b",
            "ffree st(0)", "fincstp",
            r = in(reg) r, o = inout(reg) out => _, n = inout(reg) count => _,
            c = in(reg) c, d = in(reg) d,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
        );
    }
}

/// `fld a; fchs; fstp`.
pub(super) fn negated(a: u32) -> u32 {
    let mut out = 0u32;
    x87!("fld dword ptr [{a}]", "fchs", "fstp dword ptr [{o}]"; a = &raw const a, o = &raw mut out);
    out
}

/// `fld b; fld a; fsubp st(1), st; fmul k; fstp`: (b - a) k.
pub(super) fn sub_mul(b: u32, a: u32, k: u32) -> u32 {
    let mut out = 0u32;
    x87!(
        "fld dword ptr [{b}]", "fld dword ptr [{a}]", ".byte 0xDE, 0xE9",
        "fmul dword ptr [{k}]", "fstp dword ptr [{o}]";
        b = &raw const b, a = &raw const a, k = &raw const k, o = &raw mut out
    );
    out
}

/// `fldpi; fidiv c; fimul m; fcos; fchs; fmul t; fadd a; fadd t; fstp`:
/// a + t - t cos(pi m / c).
pub(super) fn cosine_step(c: i32, m: i32, t: u32, a: u32) -> u32 {
    // One block behind one register operand: i686 at opt-level 1 has no more general
    // registers to give a block of five or six operands.
    // c, m, t, a, result.
    let mut v = [c as u32, m as u32, t, a, 0u32];
    x87!(
        "fldpi", "fidiv dword ptr [{v}]", "fimul dword ptr [{v} + 4]", "fcos", "fchs",
        "fmul dword ptr [{v} + 8]", "fadd dword ptr [{v} + 12]", "fadd dword ptr [{v} + 8]",
        "fstp dword ptr [{v} + 16]";
        v = v.as_mut_ptr()
    );
    v[4]
}

/// Three floats at `row` each times `c` over their length (`fsqrt`, the last two squares
/// added first; `fdivr c`; `fld st` twice, each multiplied and stored in turn).
pub(super) fn scale_row(row: *mut u32, c: u32) {
    x87!(
        "fld dword ptr [{r}]", ".byte 0xD8, 0xC8",
        "fld dword ptr [{r} + 4]", ".byte 0xD8, 0xC8",
        "fld dword ptr [{r} + 8]", ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1",
        "fsqrt", "fdivr dword ptr [{c}]",
        "fld st(0)", "fld st(0)",
        "fmul dword ptr [{r}]", "fstp dword ptr [{r}]",
        "fmul dword ptr [{r} + 4]", "fstp dword ptr [{r} + 4]",
        "fmul dword ptr [{r} + 8]", "fstp dword ptr [{r} + 8]";
        r = row, c = &raw const c
    );
}

/// `fld y; fist n; fisub n; fsubr k; fmul s; fadd x; fstp`: x + (k - (y - n)) s, n the
/// whole number y rounds to (as the x87 is set), which is given back too.
pub(super) fn frac_step(y: u32, k: u32, s: u32, x: u32) -> (i32, u32) {
    // One block behind one register operand: i686 at opt-level 1 has no more general
    // registers to give a block of five or six operands.
    // y, n, k, s, x, result.
    let mut v = [y, 0u32, k, s, x, 0u32];
    x87!(
        "fld dword ptr [{v}]", "fist dword ptr [{v} + 4]", "fisub dword ptr [{v} + 4]",
        "fsubr dword ptr [{v} + 8]", "fmul dword ptr [{v} + 12]", "fadd dword ptr [{v} + 16]",
        "fstp dword ptr [{v} + 20]";
        v = v.as_mut_ptr()
    );
    (v[1] as i32, v[5])
}

/// `fld a; frndint; fstp`.
pub(super) fn round_float(a: u32) -> u32 {
    let mut out = 0u32;
    x87!("fld dword ptr [{a}]", "frndint", "fstp dword ptr [{o}]"; a = &raw const a, o = &raw mut out);
    out
}

/// `fld a; fcomp b; fstsw`: the status word's C0 .. C2 (0x700).
pub(super) fn compare_status(a: u32, b: u32) -> u16 {
    let mut status: u16 = 0;
    x87!(
        "fld dword ptr [{a}]", "fcomp dword ptr [{b}]", "fstsw word ptr [{s}]";
        a = &raw const a, b = &raw const b, s = &raw mut status
    );
    status & 0x700
}

/// `fld a; fisub n; fistp`.
pub(super) fn float_less_int_to_int(a: u32, n: i32) -> i32 {
    let mut out = 0i32;
    x87!(
        "fld dword ptr [{a}]", "fisub dword ptr [{n}]", "fistp dword ptr [{o}]";
        a = &raw const a, n = &raw const n, o = &raw mut out
    );
    out
}

/// `fld1; fld c; fsub a; fdivp st(1), st; fld b; fsub a; fmulp; fld x2; fsub x0; fmulp;
/// fadd x0; fstp`: x0 + ((1 / (c - a)) (b - a)) (x2 - x0), where the long edge is at b.
pub(super) fn split_x(a: u32, b: u32, c: u32, x0: u32, x2: u32) -> u32 {
    // One block behind one register operand: i686 at opt-level 1 has no more general
    // registers to give a block of five or six operands.
    // a, b, c, x0, x2, result.
    let mut v = [a, b, c, x0, x2, 0u32];
    x87!(
        "fld1", "fld dword ptr [{v} + 8]", "fsub dword ptr [{v}]", ".byte 0xDE, 0xF9",
        "fld dword ptr [{v} + 4]", "fsub dword ptr [{v}]", ".byte 0xDE, 0xC9",
        "fld dword ptr [{v} + 16]", "fsub dword ptr [{v} + 12]", ".byte 0xDE, 0xC9",
        "fadd dword ptr [{v} + 12]", "fstp dword ptr [{v} + 20]";
        v = v.as_mut_ptr()
    );
    v[5]
}

/// `fld b; fsub a; fmul k; fst h; fadd a; fstp m`: the half (b - a) k as a float, and a plus
/// it unrounded, then as a float.
pub(super) fn half_and_middle(b: u32, a: u32, k: u32) -> (u32, u32) {
    // One block behind one register operand: i686 at opt-level 1 has no more general
    // registers to give a block of five or six operands.
    // b, a, k, h, m.
    let mut v = [b, a, k, 0u32, 0u32];
    x87!(
        "fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fst dword ptr [{v} + 12]", "fadd dword ptr [{v} + 4]", "fstp dword ptr [{v} + 16]";
        v = v.as_mut_ptr()
    );
    (v[3], v[4])
}

/// `fldpi; fidiv n; fstp`.
pub(super) fn pi_over(n: i32) -> u32 {
    let mut out = 0u32;
    x87!("fldpi", "fidiv dword ptr [{n}]", "fstp dword ptr [{o}]"; n = &raw const n, o = &raw mut out);
    out
}

/// `fld a; [fadd s;] fcos; fchs; fmul h; fadd m; fstp`: m - h cos(a [+ s]).
pub(super) fn less_cosine(a: u32, s: Option<u32>, h: u32, m: u32) -> u32 {
    let mut out = 0u32;
    match s {
        Some(s) => {
            // a, s, h, m, result behind one register (see `cosine_step`).
            let mut v = [a, s, h, m, 0u32];
            x87!(
                "fld dword ptr [{v}]", "fadd dword ptr [{v} + 4]", "fcos", "fchs",
                "fmul dword ptr [{v} + 8]", "fadd dword ptr [{v} + 12]", "fstp dword ptr [{v} + 16]";
                v = v.as_mut_ptr()
            );
            out = v[4];
        }
        None => x87!(
            "fld dword ptr [{a}]", "fcos", "fchs",
            "fmul dword ptr [{h}]", "fadd dword ptr [{m}]", "fstp dword ptr [{o}]";
            a = &raw const a, h = &raw const h, m = &raw const m, o = &raw mut out
        ),
    }
    out
}

/// `fld a; fsin; fstp`.
pub(super) fn sine(a: u32) -> u32 {
    let mut out = 0u32;
    x87!("fld dword ptr [{a}]", "fsin", "fstp dword ptr [{o}]"; a = &raw const a, o = &raw mut out);
    out
}

/// `fld a; fcos; fstp`, and with `negate` an `fchs` before the store.
pub(super) fn cosine(a: u32, negate: bool) -> u32 {
    let mut out = 0u32;
    if negate {
        x87!("fld dword ptr [{a}]", "fcos", "fchs", "fstp dword ptr [{o}]"; a = &raw const a, o = &raw mut out);
    } else {
        x87!("fld dword ptr [{a}]", "fcos", "fstp dword ptr [{o}]"; a = &raw const a, o = &raw mut out);
    }
    out
}

/// `fld a; fmul b; fadd c; fstp`.
pub(super) fn mul_add(a: u32, b: u32, c: u32) -> u32 {
    let mut out = 0u32;
    x87!(
        "fld dword ptr [{a}]", "fmul dword ptr [{b}]", "fadd dword ptr [{c}]",
        "fstp dword ptr [{o}]";
        a = &raw const a, b = &raw const b, c = &raw const c, o = &raw mut out
    );
    out
}

/// `fild a; fmul st, st; fild b; fmul st, st; fild c; fmul st, st; faddp; faddp; fsqrt;
/// fistp`: the length of (a, b, c), the last two squares added first, rounded.
pub(super) fn int_length3(a: i32, b: i32, c: i32) -> i32 {
    let (v, mut out) = ([a, b, c], 0i32);
    x87!(
        "fild dword ptr [{v}]", ".byte 0xD8, 0xC8",
        "fild dword ptr [{v} + 4]", ".byte 0xD8, 0xC8",
        "fild dword ptr [{v} + 8]", ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fsqrt", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out
    );
    out
}

/// `fild a; fiadd b; fistp`: the sum, or 0x80000000 when it does not fit.
pub(super) fn int_sum(a: i32, b: i32) -> i32 {
    let mut out = 0i32;
    x87!(
        "fild dword ptr [{a}]", "fiadd dword ptr [{b}]", "fistp dword ptr [{o}]";
        a = &raw const a, b = &raw const b, o = &raw mut out
    );
    out
}

/// `fild x0; fmul st, st; fild y0; fmul st, st; faddp; fild x1; .. faddp; fcompp; fstsw`:
/// x1² + y1² against x0² + y0², the status word's C0 .. C2.
pub(super) fn compare_squares(x0: i32, y0: i32, x1: i32, y1: i32) -> u16 {
    let (v, mut status) = ([x0, y0, x1, y1], 0u16);
    x87!(
        "fild dword ptr [{v}]", ".byte 0xD8, 0xC8", "fild dword ptr [{v} + 4]", ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1",
        "fild dword ptr [{v} + 8]", ".byte 0xD8, 0xC8", "fild dword ptr [{v} + 12]", ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1",
        "fcompp", "fstsw word ptr [{s}]";
        v = v.as_ptr(), s = &raw mut status
    );
    status & 0x700
}

/// `fild a; fidiv n; fstp`.
pub(super) fn int_div_int(a: i32, n: i32) -> u32 {
    let mut out = 0u32;
    x87!("fild dword ptr [{a}]", "fidiv dword ptr [{n}]", "fstp dword ptr [{o}]"; a = &raw const a, n = &raw const n, o = &raw mut out);
    out
}

/// `fld a; fsub b`, kept as an 80-bit value.
pub(super) fn sub_80(a: u32, b: u32) -> [u8; 10] {
    let mut out = [0u8; 10];
    x87!("fld dword ptr [{a}]", "fsub dword ptr [{b}]", "fstp tbyte ptr [{o}]"; a = &raw const a, b = &raw const b, o = out.as_mut_ptr());
    out
}

/// `fild a; fsub b`, kept as an 80-bit value.
pub(super) fn int_sub_80(a: i32, b: u32) -> [u8; 10] {
    let mut out = [0u8; 10];
    x87!("fild dword ptr [{a}]", "fsub dword ptr [{b}]", "fstp tbyte ptr [{o}]"; a = &raw const a, b = &raw const b, o = out.as_mut_ptr());
    out
}

/// `fld a; fadd b; fistp`.
pub(super) fn add_to_int(a: u32, b: u32) -> i32 {
    let mut out = 0i32;
    x87!("fld dword ptr [{a}]", "fadd dword ptr [{b}]", "fistp dword ptr [{o}]"; a = &raw const a, b = &raw const b, o = &raw mut out);
    out
}

// ---------------------------------------------------------------------------------------------
// The differential tests: each plain Rust version in `math/x87.rs` against the assembly above,
// at the four rounding modes (the real and the emulated control word both set).

use crate::math::x87 as plain;
use crate::math::x87::set_control_word;
use crate::math::x87_float::{X, f80, from_f80};

/// Seeded generator (xorshift64*).
struct G(u64);

impl G {
    fn new(seed: u64) -> G {
        G(seed | 1)
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    /// A float's bits: random patterns, edge values, whole numbers, halves, values around
    /// the range of `i32`, denormals; now and then an infinity or a NaN.
    fn f(&mut self) -> u32 {
        const EDGE: [f32; 18] = [
            0.0,
            -0.0,
            1.0,
            -1.0,
            0.5,
            -0.5,
            1.5,
            2.5,
            -2.5,
            0.1,
            f32::MIN_POSITIVE,
            f32::MAX,
            -f32::MAX,
            2147483648.0,
            2147483520.0,
            -2147483648.0,
            1e-45,
            3.15,
        ];
        match self.below(10) {
            9 => [
                0x7F80_0000u32,
                0xFF80_0000,
                0x7FC0_0000,
                0xFFC0_0000,
                0x7F80_0001,
                0xFFA0_0000,
            ][self.below(6) as usize],
            0 => loop {
                let bits = self.next() as u32;
                if (bits >> 23) & 0xFF != 0xFF {
                    return bits;
                }
            },
            1 => EDGE[self.below(EDGE.len() as u64) as usize].to_bits(),
            2 => ((self.below(401) as i32 - 200) as f32).to_bits(),
            3 => (((self.below(200_001) as i32 - 100_000) as f32) + 0.5).to_bits(),
            4 => (((self.next() >> 11) as f64 / (1u64 << 53) as f64 * 2e4 - 1e4) as f32).to_bits(),
            5 => (((self.next() >> 11) as f64 / (1u64 << 53) as f64 * 6e9 - 3e9) as f32).to_bits(),
            6 => self.next() as u32 & 0x807F_FFFF,
            7 => (((self.next() >> 11) as f64 / (1u64 << 53) as f64 * 4.0 - 2.0) as f32).to_bits(),
            _ => (self.next() as i32 as f32).to_bits(),
        }
    }

    /// A float for the sine and cosine: modest angles, some larger, some edge values.
    fn fa(&mut self) -> u32 {
        match self.below(6) {
            0 => ((self.below(2001) as f32 - 1000.0) / 8.0).to_bits(),
            1 => {
                (((self.next() >> 11) as f64 / (1u64 << 53) as f64 * 20.0 - 10.0) as f32).to_bits()
            }
            2 => (((self.next() >> 11) as f64 / (1u64 << 53) as f64 * 2e6 - 1e6) as f32).to_bits(),
            3 => [0.0f32, -0.0, 1e-30, 3.15, 1.55, -3.15, 6.3][self.below(7) as usize].to_bits(),
            _ => self.f() & 0x8000_FFFF | 0x3F00_0000,
        }
    }

    /// A scale for an angle: between 0.5 and 1000.5.
    fn fs(&mut self) -> u32 {
        (((self.next() >> 11) as f64 / (1u64 << 53) as f64 * 1000.0 + 0.5) as f32).to_bits()
    }

    /// A whole number: random, small, or an extreme.
    fn i(&mut self) -> i32 {
        match self.below(6) {
            0 => self.next() as i32,
            1 => self.below(4001) as i32 - 2000,
            2 => {
                [0, 1, -1, i32::MIN, i32::MAX, 46340, 46341, -46341, 65536][self.below(9) as usize]
            }
            3 => self.below(2_000_001) as i32 - 1_000_000,
            4 => (self.next() as i32) >> (self.below(31) as u32),
            _ => self.below(33) as i32 - 16,
        }
    }

    /// A non-zero whole number up to 1000 in size (an angle's divisor: a tiny angle would
    /// leave only the x87's extended cosine to tell `1 - cos` from 0).
    fn c1000(&mut self) -> i32 {
        loop {
            let v = self.below(2001) as i32 - 1000;
            if v != 0 {
                return v;
            }
        }
    }

    /// A non-zero whole number.
    fn nz(&mut self) -> i32 {
        loop {
            let v = self.i();
            if v != 0 {
                return v;
            }
        }
    }

    /// A small count for loops.
    fn count(&mut self, max: u64) -> u32 {
        self.below(max + 1) as u32
    }
}

/// How far a float result may differ: `ulps` units in the last place of a float, or `abs`
/// in absolute value. Only the sine and cosine need any: the x87's `fsin` / `fcos` reduce
/// the angle with a 66-bit pi, so near a multiple of pi they are off by about 4e-21 per
/// multiple of pi in absolute terms (the true value is what libm gives; the x87 is the one
/// that is off, by hundreds of float ulps when the result is near zero).
#[derive(Clone, Copy, Debug)]
struct Tol {
    ulps: u32,
    abs: f64,
}

const EXACT: Tol = Tol { ulps: 0, abs: 0.0 };

impl Tol {
    /// A sine or cosine of `arg`: one float ulp, or the 66-bit pi error of an angle that size,
    /// plus one double ulp of 1 (the size of a sine or cosine): processors' `fcos` differ there
    /// (an i5-1335U gives 1 - h cos(1.3e-30) = 0 rounding toward zero, the others 2^-53).
    fn trig(arg: f64) -> Tol {
        Tol {
            ulps: 1,
            abs: 1e-20 * (1.0 + arg.abs()) + f64::EPSILON,
        }
    }

    /// The same, for a result multiplied by `k`.
    fn times(self, k: f64) -> Tol {
        Tol {
            abs: self.abs * k.abs(),
            ..self
        }
    }
}

fn fl(bits: u32) -> f64 {
    f64::from(f32::from_bits(bits))
}

/// A plain helper's result as the reference's: the bits of a float result, a whole number or
/// a status word as it is.
pub(super) trait AsBits {
    type Out;
    fn into_bits(self) -> Self::Out;
}

impl AsBits for f32 {
    type Out = u32;
    fn into_bits(self) -> u32 {
        self.to_bits()
    }
}

macro_rules! as_they_are {
    ($($t:ty),*) => {$(
        impl AsBits for $t {
            type Out = $t;
            fn into_bits(self) -> $t {
                self
            }
        }
    )*};
}
as_they_are!(u8, u16, u32, i32, bool, f64);

impl<A: AsBits, B: AsBits> AsBits for (A, B) {
    type Out = (A::Out, B::Out);
    fn into_bits(self) -> Self::Out {
        (self.0.into_bits(), self.1.into_bits())
    }
}

impl<A: AsBits, B: AsBits, C: AsBits> AsBits for (A, B, C) {
    type Out = (A::Out, B::Out, C::Out);
    fn into_bits(self) -> Self::Out {
        (self.0.into_bits(), self.1.into_bits(), self.2.into_bits())
    }
}

impl<A: AsBits, const N: usize> AsBits for [A; N] {
    type Out = [A::Out; N];
    fn into_bits(self) -> Self::Out {
        self.map(AsBits::into_bits)
    }
}

/// Results that must agree: floats as bits (any NaN equals any NaN, within `tol`),
/// everything else exactly.
trait Same {
    fn same(&self, other: &Self, tol: Tol) -> bool;
}

fn ordered(bits: u32) -> i64 {
    if bits >> 31 == 1 {
        -i64::from(bits & 0x7FFF_FFFF)
    } else {
        i64::from(bits)
    }
}

impl Same for u32 {
    fn same(&self, other: &u32, tol: Tol) -> bool {
        let (x, y) = (f32::from_bits(*self), f32::from_bits(*other));
        if x.is_nan() || y.is_nan() {
            return x.is_nan() && y.is_nan();
        }
        self == other
            || (ordered(*self) - ordered(*other)).unsigned_abs() <= u64::from(tol.ulps)
            || (x.is_finite() && y.is_finite() && (fl(*self) - fl(*other)).abs() <= tol.abs)
    }
}

impl Same for i32 {
    fn same(&self, other: &i32, _tol: Tol) -> bool {
        self == other
    }
}

impl Same for u16 {
    fn same(&self, other: &u16, _tol: Tol) -> bool {
        self == other
    }
}

impl Same for bool {
    fn same(&self, other: &bool, _tol: Tol) -> bool {
        self == other
    }
}

impl<A: Same, B: Same> Same for (A, B) {
    fn same(&self, other: &Self, tol: Tol) -> bool {
        self.0.same(&other.0, tol) && self.1.same(&other.1, tol)
    }
}

impl Same for [u8; 10] {
    fn same(&self, other: &[u8; 10], _tol: Tol) -> bool {
        let nan = |v: &[u8; 10]| {
            u16::from_le_bytes([v[8], v[9]]) & 0x7FFF == 0x7FFF
                && u64::from_le_bytes(v[..8].try_into().unwrap()) << 1 != 0
        };
        self == other || (nan(self) && nan(other))
    }
}

impl Same for Vec<u32> {
    fn same(&self, other: &Vec<u32>, tol: Tol) -> bool {
        self.len() == other.len() && self.iter().zip(other).all(|(a, b)| a.same(b, tol))
    }
}

impl Same for Vec<i32> {
    fn same(&self, other: &Vec<i32>, _tol: Tol) -> bool {
        self == other
    }
}

/// Restores the control word at the end of a test, panicking or not.
struct Restore;

impl Drop for Restore {
    fn drop(&mut self) {
        set_control_word(0x027F);
    }
}

const CASES: usize = 30_000;

/// The control words the game's floats run under: nearest (double precision), chop, down
/// and up. Both the real and the emulated control word are set.
const MODES: [u16; 4] = [0x027F, 0x0E7F, 0x067F, 0x0A7F];

/// Runs `body` `CASES` times at each of the control words.
fn each_mode(seed: u64, modes: &[u16], mut body: impl FnMut(&mut G)) {
    let _restore = Restore;
    for &cw in modes {
        set_control_word(cw);
        let mut g = G::new(seed ^ u64::from(cw));
        for _ in 0..CASES {
            body(&mut g);
        }
    }
}

fn compare<T: Same + core::fmt::Debug>(
    name: &str,
    tol: Tol,
    reference: T,
    plain: T,
    inputs: String,
) {
    assert!(
        reference.same(&plain, tol),
        "{name}({inputs}) at control word {:#06x}: reference {reference:x?}, plain {plain:x?}",
        plain::control_word()
    );
}

/// `diff!(g, "name", tolerance, a = f, b = i; reference expression, plain expression)`: draws
/// each named input from the generator (`G::f`, `G::i`, ..) and compares the two.
macro_rules! diff {
    ($g:ident, $name:literal, $tol:expr, $($v:ident = $gen:ident),*; $r:expr, $p:expr) => {{
        $(let $v = $g.$gen();)*
        let reference = $r;
        let plain = AsBits::into_bits($p);
        compare($name, $tol, reference, plain, format!("{:x?}", ($($v,)*)));
    }};
}

#[test]
fn float_arithmetic() {
    each_mode(1, &MODES, |g| {
        diff!(g, "add", EXACT, a = f, b = f; add(a, b), plain::add(a, b));
        diff!(g, "sub", EXACT, a = f, b = f; sub(a, b), plain::sub(a, b));
        diff!(g, "from_int", EXACT, a = i; from_int(a), plain::from_int(a));
        diff!(g, "mul", EXACT, a = f, b = f; mul(a, b), plain::mul(a, b));
        diff!(g, "reciprocal_times", EXACT, a = f, b = f; reciprocal_times(a, b), plain::reciprocal_times(a, b));
        diff!(g, "mirrored", EXACT, x = f, a = f, b = f; mirrored(x, a, b), plain::mirrored(x, a, b));
        diff!(g, "loaded", EXACT, a = f; loaded(a), plain::loaded(a));
        diff!(g, "int_ratio", EXACT, a = i, b = i; int_ratio(a, b), plain::int_ratio(a, b));
        diff!(g, "one_less_int_ratio", EXACT, a = i, b = i; one_less_int_ratio(a, b), plain::one_less_int_ratio(a, b));
        diff!(g, "int_over_16", EXACT, a = i; int_over_16(a), plain::int_over_16(a));
        diff!(g, "div", EXACT, a = f, b = f; div(a, b), plain::div(a, b));
        diff!(g, "product_difference", EXACT, a = f, b = f, c = f, d = f;
            product_difference(a, b, c, d), plain::product_difference(a, b, c, d));
        diff!(g, "negated_product_difference", EXACT, a = f, b = f, c = f, d = f;
            negated_product_difference(a, b, c, d), plain::negated_product_difference(a, b, c, d));
        diff!(g, "equal_or_unordered", EXACT, a = f, b = f; equal_or_unordered(a, b), plain::equal_or_unordered(a, b));
        diff!(g, "int_over_root", EXACT, a = i, d = f; int_over_root(a, d), plain::int_over_root(a, d));
        diff!(g, "sqrt", EXACT, a = f; sqrt(a), plain::sqrt(a));
        diff!(g, "int_times", EXACT, a = i, b = f; int_times(a, b), plain::int_times(a, b));
        diff!(g, "int_over", EXACT, a = i, b = f; int_over(a, b), plain::int_over(a, b));
        diff!(g, "int_less", EXACT, a = i, b = f; int_less(a, b), plain::int_less(a, b));
        diff!(g, "reciprocal", EXACT, a = f; reciprocal(a), plain::reciprocal(a));
        diff!(g, "int_float_int_float", EXACT, a = i, b = f, c = i, d = f;
            int_float_int_float(a, b, c, d), plain::int_float_int_float(a, b, c, d));
        diff!(g, "int_product_difference", EXACT, a = i, b = i, c = i, d = i;
            int_product_difference(a, b, c, d), plain::int_product_difference(a, b, c, d));
        diff!(g, "int_div", EXACT, a = i, b = f; int_div(a, b), plain::int_over(a, b));
        diff!(g, "int_div_div_doubled", EXACT, a = i, b = f, c = f;
            int_div_div_doubled(a, b, c), plain::int_div_div_doubled(a, b, c));
        diff!(g, "surface_share", EXACT, a = i, b = f, c = f, d = i, e = f, f = f;
            surface_share(a, b, c, d, e, f), plain::surface_share(a, b, c, d, e, f));
        diff!(g, "hypot_ratio", EXACT, a = i, b = i, c = i, d = i;
            hypot_ratio(a, b, c, d), plain::hypot_ratio(a, b, c, d));
        diff!(g, "int_div_less", EXACT, a = i, b = f, c = f; int_div_less(a, b, c), plain::int_div_less(a, b, c));
        diff!(g, "reciprocal_int_and_times", EXACT, a = i, b = f;
            reciprocal_int_and_times(a, b), plain::reciprocal_int_and_times(a, b));
        diff!(g, "mul_and_div", EXACT, a = f, b = f, c = f; mul_and_div(a, b, c), plain::mul_and_div(a, b, c));
        diff!(g, "sub_div_mul", EXACT, a = f, b = f, c = f, d = f;
            sub_div_mul(a, b, c, d), plain::sub_div_mul(a, b, c, d));
        diff!(g, "diff_ratio_mul", EXACT, a = f, b = f, c = f, d = f, e = f;
            diff_ratio_mul(a, b, c, d, e), plain::diff_ratio_mul(a, b, c, d, e));
        diff!(g, "lerp_scaled_less", EXACT, a = f, b = f, m = f, n = f, l = f;
            lerp_scaled_less(a, b, m, n, Some(l)), plain::lerp_scaled_less(a, b, m, n, Some(l)));
        diff!(g, "lerp_scaled_less (none)", EXACT, a = f, b = f, m = f, n = f;
            lerp_scaled_less(a, b, m, n, None), plain::lerp_scaled_less(a, b, m, n, None::<u32>));
        diff!(g, "negated_sub", EXACT, a = f, b = f; negated_sub(a, b), plain::negated_sub(a, b));
        diff!(g, "int_times_plus", EXACT, a = i, b = f, c = f; int_times_plus(a, b, c), plain::int_times_plus(a, b, c));
        diff!(g, "int_less_int", EXACT, a = i, b = i; int_less_int(a, b), plain::int_less_int(a, b));
        diff!(g, "int_less_int_over", EXACT, a = i, b = i, c = f;
            int_less_int_over(a, b, c), plain::int_less_int_over(a, b, c));
        diff!(g, "sub_div", EXACT, a = f, b = f, c = f; sub_div(a, b, c), plain::sub_div(a, b, c));
    });
}

#[test]
fn float_arithmetic_b() {
    each_mode(2, &MODES, |g| {
        diff!(g, "mul2", EXACT, a = f, b = f, c = f; mul2(a, b, c), plain::mul2(a, b, c));
        diff!(g, "circle_curve", EXACT, i = i, a = f, b = f;
            circle_curve(i, a, b, false), plain::circle_curve(i, a, b, false));
        diff!(g, "circle_curve (negate)", EXACT, i = i, a = f, b = f;
            circle_curve(i, a, b, true), plain::circle_curve(i, a, b, true));
        diff!(g, "negated", EXACT, a = f; negated(a), plain::negated(a));
        diff!(g, "sub_mul", EXACT, b = f, a = f, k = f; sub_mul(b, a, k), plain::sub_mul(b, a, k));
        diff!(g, "frac_step", EXACT, y = f, k = f, s = f, x = f; frac_step(y, k, s, x), plain::frac_step(y, k, s, x));
        diff!(g, "compare_status", EXACT, a = f, b = f; compare_status(a, b), plain::compare_status(a, b));
        diff!(g, "split_x", EXACT, a = f, b = f, c = f, x0 = f, x2 = f;
            split_x(a, b, c, x0, x2), plain::split_x(a, b, c, x0, x2));
        diff!(g, "half_and_middle", EXACT, b = f, a = f, k = f;
            half_and_middle(b, a, k), plain::half_and_middle(b, a, k));
        diff!(g, "pi_over", EXACT, n = i; pi_over(n), plain::pi_over(n));
        diff!(g, "mul_add", EXACT, a = f, b = f, c = f; mul_add(a, b, c), plain::mul_add(a, b, c));
        diff!(g, "int_div_int", EXACT, a = i, n = i; int_div_int(a, n), plain::int_ratio(a, n));
        diff!(g, "sub_80", EXACT, a = f, b = f; sub_80(a, b), plain::sub_80(a, b));
        diff!(g, "int_sub_80", EXACT, a = i, b = f; int_sub_80(a, b), plain::int_sub_80(a, b));
    });
}

#[test]
fn whole_numbers_after_division_or_transcendental() {
    each_mode(9, &MODES, |g| {
        diff!(g, "ratio_scaled_to_int", EXACT, a = f, b = f, m = i, n = i;
            ratio_scaled_to_int(a, b, m, n), plain::ratio_scaled_to_int(a, b, m, n));
        diff!(g, "mul_div_div_to_int", EXACT, a = f, b = f, c = f, d = f;
            mul_div_div_to_int(a, b, c, d), plain::mul_div_div_to_int(a, b, c, d));
        diff!(g, "share_left_to_int", EXACT, a = i, b = i, c = f;
            share_left_to_int(a, b, c), plain::share_left_to_int(a, b, c));
        diff!(g, "angle_scaled_to_int", EXACT, y = i, x = i, a = f, b = f;
            angle_scaled_to_int(y, x, a, b), plain::angle_scaled_to_int(y, x, a, b));
        diff!(g, "angle_scaled3_to_int", EXACT, y = i, x = i, a = f, b = f, c = f;
            angle_scaled3_to_int(y, x, a, b, c), plain::angle_scaled3_to_int(y, x, a, b, c));
        diff!(g, "angle_scaled5_to_int", EXACT, y = i, x = i, a = f, b = f;
            angle_scaled5_to_int(y, x, a, b), plain::angle_scaled5_to_int(y, x, a, b));
        diff!(g, "sine_times_plus_to_int", EXACT, a = fa, b = f, c = f;
            sine_times_plus_to_int(a, b, c), plain::sine_times_plus_to_int(a, b, c));
    });
}

#[test]
fn results_as_whole_numbers() {
    each_mode(7, &MODES, |g| {
        diff!(g, "to_int", EXACT, a = f; to_int(a), plain::to_int(a));
        diff!(g, "mul_to_int", EXACT, a = f, b = f; mul_to_int(a, b), plain::mul_to_int(a, b));
        diff!(g, "int_hypot", EXACT, a = i, b = i; int_hypot(a, b), plain::int_hypot(a, b));
        diff!(g, "int_times_to_int", EXACT, a = i, b = f; int_times_to_int(a, b), plain::int_times_to_int(a, b));
        diff!(g, "weighted3_to_int", EXACT, a = i, b = i, c = i, x = f, y = f, z = f;
            weighted3_to_int(a, b, c, x, y, z), plain::weighted3_to_int(a, b, c, x, y, z));
        diff!(g, "negated_square_times_plus_to_int", EXACT, a = f, b = f, c = f;
            negated_square_times_plus_to_int(a, b, c), plain::negated_square_times_plus_to_int(a, b, c));
        diff!(g, "length_plus_int", EXACT, x = i, y = i, z = i, s = i;
            length_plus_int(x, y, z, s), plain::length_plus_int(x, y, z, s));
        diff!(g, "round_float", EXACT, a = f; round_float(a), plain::round_float(a));
        diff!(g, "float_less_int_to_int", EXACT, a = f, n = i;
            float_less_int_to_int(a, n), plain::float_less_int_to_int(a, n));
        diff!(g, "int_length3", EXACT, a = i, b = i, c = i; int_length3(a, b, c), plain::int_length3(a, b, c));
        diff!(g, "int_sum", EXACT, a = i, b = i; int_sum(a, b), plain::int_sum(a, b));
        diff!(g, "compare_squares", EXACT, a = i, b = i, c = i, d = i;
            compare_squares(a, b, c, d), plain::compare_squares(a, b, c, d));
        diff!(g, "add_to_int", EXACT, a = f, b = f; add_to_int(a, b), plain::add_to_int(a, b));
    });
}

/// The comparisons see no-number inputs too.
#[test]
fn comparisons_with_nan() {
    let nans = [
        0x7FC0_0000u32,
        0xFFC0_0000,
        0x7F80_0001,
        0x7F80_0000,
        0xFF80_0000,
    ];
    each_mode(3, &MODES, |g| {
        let n = nans[g.below(5) as usize];
        let a = g.f();
        for (x, y) in [(n, a), (a, n), (n, n)] {
            compare(
                "equal_or_unordered",
                EXACT,
                equal_or_unordered(x, y),
                plain::equal_or_unordered(x, y),
                format!("{x:x} {y:x}"),
            );
            compare(
                "compare_status",
                EXACT,
                compare_status(x, y),
                plain::compare_status(x, y),
                format!("{x:x} {y:x}"),
            );
        }
        let mut r = [g.f(), g.f(), g.f()];
        let mut p = r;
        let z = n;
        let one = unsafe { normalise_in_place(r.as_mut_ptr(), z) };
        let two = plain::normalise_in_place((&mut p[..3]).try_into().unwrap(), z);
        let two = (two.0.to_bits(), two.1);
        compare(
            "normalise_in_place (nan zero)",
            EXACT,
            (one, r.to_vec()),
            (two, p.to_vec()),
            format!("{z:x}"),
        );
    });
}

#[test]
fn trigonometry() {
    each_mode(4, &MODES, |g| {
        // the sines and cosines: within one ulp of a float (2 ulps for the two-step ones)
        diff!(g, "sine_cosine", Tol::trig(f64::from(angle % 100_000) / fl(a) / fl(a) / fl(b)), angle = i, a = fs, b = fs; sine_cosine(angle % 100_000, a, b), plain::sine_cosine(angle % 100_000, a, b));
        diff!(g, "sine_of_pi_fraction", Tol::trig(f64::from(n) / f64::from(d) * 3.15), n = c1000, d = c1000; sine_of_pi_fraction(n, d), plain::sine_of_pi_fraction(n, d));
        diff!(g, "cosine_step", Tol::trig(f64::from(m) / f64::from(c) * 3.15).times(fl(t)), c = c1000, m = i, t = f, a = f;
            cosine_step(c, m, t, a), plain::cosine_step(c, m, t, a));
        diff!(g, "less_cosine", Tol::trig(fl(a) + fl(s)).times(fl(h)), a = fa, s = fa, h = f, m = f;
            less_cosine(a, Some(s), h, m), plain::less_cosine(a, Some(s), h, m));
        diff!(g, "less_cosine (none)", Tol::trig(fl(a)).times(fl(h)), a = fa, h = f, m = f;
            less_cosine(a, None, h, m), plain::less_cosine(a, None::<u32>, h, m));
        diff!(g, "sine", Tol::trig(fl(a)), a = fa; sine(a), plain::sine(a));
        diff!(g, "cosine", Tol::trig(fl(a)), a = fa; cosine(a, false), plain::cosine(a, false));
        diff!(g, "cosine (negate)", Tol::trig(fl(a)), a = fa; cosine(a, true), plain::cosine(a, true));
        let step = g.fa();
        let count = g.count(40);
        let mut r = vec![0xAAAA_AAAAu32; 41];
        let mut p = r.clone();
        unsafe {
            sine_table(step, r.as_mut_ptr(), count);
            plain::sine_table(step, &mut p[..count as usize]);
        }
        compare(
            "sine_table",
            Tol::trig(f64::from(count) * fl(step)),
            r,
            p,
            format!("{step:x} {count}"),
        );
        let radius = g.f();
        let (c, d) = (g.nz() % 1000, g.fa());
        let count = 1 + g.count(11);
        let mut r = vec![0xAAAA_AAAAu32; 40];
        let mut p = r.clone();
        unsafe {
            circle_points(
                &raw const radius,
                r.as_mut_ptr(),
                count,
                &raw const c,
                &raw const d,
            );
            plain::circle_points(radius, &mut p, count, c, d);
        }
        compare(
            "circle_points",
            Tol::trig(40.0 * fl(d)).times(fl(radius)),
            r,
            p,
            format!("{radius:x} {count} {c} {d:x}"),
        );
    });
}

#[test]
fn pointer_functions() {
    each_mode(5, &MODES, |g| {
        let a = [g.f(), g.f(), g.f()];
        // a matrix's worth of floats (9) for the strided ones
        let m: Vec<u32> = (0..9).map(|_| g.f()).collect();
        // `row_times_column` reads b + 0x10 and b + 0x20: a 12-dword matrix
        let b: Vec<u32> = (0..12).map(|_| g.f()).collect();
        let (mut r, mut p) = (vec![0xAAAA_AAAAu32], vec![0xAAAA_AAAAu32]);
        row_times_column(a.as_ptr(), b.as_ptr(), r.as_mut_ptr());
        plain::row_times_column(a.as_ptr(), b.as_ptr(), p.as_mut_ptr());
        compare("row_times_column", EXACT, r, p, format!("{a:x?} {b:x?}"));
        let (mut r, mut p) = (vec![0xAAAA_AAAAu32], vec![0xAAAA_AAAAu32]);
        row_times_column_3(a.as_ptr(), m.as_ptr(), r.as_mut_ptr());
        let cells = |v: &[u32]| {
            v.iter()
                .map(|&x| crate::session::Dword::new(x))
                .collect::<Vec<_>>()
        };
        p[0] = crate::scene::matrix3::row_times_column_3(&cells(&a)[..], &cells(&m)[..]);
        compare("row_times_column_3", EXACT, r, p, format!("{a:x?} {m:x?}"));

        let (x, start) = (g.f(), g.f());
        let (mut r, mut p) = (vec![start], vec![start]);
        add_into(&raw const x, r.as_mut_ptr());
        plain::add_into(&raw const x, p.as_mut_ptr());
        compare("add_into", EXACT, r, p, format!("{x:x} {start:x}"));

        let (r1, p1) = ({ length(a.as_ptr()) }, { plain::length(a.as_ptr()) });
        compare("length", EXACT, r1, p1, format!("{a:x?}"));

        let zero = [0u32, 0x8000_0000, g.f(), 0x3F80_0000][g.below(4) as usize];
        let mut v = [g.f(), g.f(), g.f()];
        if g.below(8) == 0 {
            v = [0, 0x8000_0000, 0];
        }
        let (mut r, mut p) = (v, v);
        let one = unsafe { normalise_in_place(r.as_mut_ptr(), zero) };
        let two = plain::normalise_in_place((&mut p[..3]).try_into().unwrap(), zero);
        let two = (two.0.to_bits(), two.1);
        compare(
            "normalise_in_place",
            EXACT,
            (one, r.to_vec()),
            (two, p.to_vec()),
            format!("{v:x?} {zero:x}"),
        );

        let (s, c) = (g.fa(), g.fa());
        let (x, z) = (g.f(), g.f());
        let (mut rx, mut rz, mut px, mut pz) = (x, z, x, z);
        turn_in_place(&raw const s, &raw const c, &raw mut rx, &raw mut rz);
        plain::turn_in_place(s, c, &mut px, &mut pz);
        compare(
            "turn_in_place",
            EXACT,
            vec![rx, rz],
            vec![px, pz],
            format!("{s:x} {c:x} {x:x} {z:x}"),
        );

        let row = [g.f(), g.f(), g.f()];
        let k = g.f();
        let (mut r, mut p) = (row, row);
        scale_row(r.as_mut_ptr(), k);
        p = plain::scale_row(p, k).map(f32::to_bits);
        compare(
            "scale_row",
            EXACT,
            r.to_vec(),
            p.to_vec(),
            format!("{row:x?} {k:x}"),
        );

        let count = g.count(16);
        let values: Vec<i32> = (0..16).map(|_| g.i()).collect();
        let (last, first, by) = (g.i(), g.i(), g.nz());
        let (mut r, mut p) = (values.clone(), values.clone());
        unsafe {
            ramp_down(r.as_mut_ptr(), count, last, first, by);
            plain::ramp_down(&mut p[..count as usize], last, first, by);
        }
        compare(
            "ramp_down",
            EXACT,
            r,
            p,
            format!("{count} {last} {first} {by}"),
        );
    });
}

#[test]
fn pointer_functions_to_int() {
    each_mode(8, &MODES, |g| {
        let pt = [g.i(), g.i(), g.i()];
        let mat: Vec<u32> = (0..9).map(|_| g.f()).collect();
        let (mut r, mut p) = (vec![0x5555_5555i32; 3], vec![0x5555_5555i32; 3]);
        turn_int_point(pt.as_ptr(), mat.as_ptr(), r.as_mut_ptr());
        let cells = |v: &[u32]| {
            v.iter()
                .map(|&x| crate::session::Dword::new(x))
                .collect::<Vec<_>>()
        };
        let turn = |from: &[crate::session::Dword], to: &[crate::session::Dword]| {
            crate::scene::matrix3::turn_int_point(from, &cells(&mat)[..], to, 0);
        };
        let (from, to) = (cells(&pt.map(|x| x as u32)), cells(&[0x5555_5555; 3]));
        turn(&from, &to);
        for (o, c) in p.iter_mut().zip(&to) {
            *o = c.get() as i32;
        }
        compare("turn_int_point", EXACT, r, p, format!("{pt:x?} {mat:x?}"));
        // out over p
        let (mut r, mut p) = (pt, pt);
        turn_int_point(r.as_ptr(), mat.as_ptr(), r.as_mut_ptr());
        let in_place = cells(&pt.map(|x| x as u32));
        turn(&in_place, &in_place);
        for (o, c) in p.iter_mut().zip(&in_place) {
            *o = c.get() as i32;
        }
        compare(
            "turn_int_point (in place)",
            EXACT,
            r.to_vec(),
            p.to_vec(),
            format!("{pt:x?} {mat:x?}"),
        );
    });
}

#[test]
fn eighty_bit_conversions() {
    let mut g = G::new(6);
    for _ in 0..200_000 {
        let bits = match g.below(4) {
            0 => g.next(),
            1 => g.next() & 0x800F_FFFF_FFFF_FFFF,
            2 => g.next() & 0x80FF_FFFF_FFFF_FFFF,
            _ => (u64::from(g.f()) << 32) | (g.next() & 0xFFFF_FFFF),
        };
        let x = f64::from_bits(bits);
        if x.is_nan() {
            continue;
        }
        let v = f80(X(x));
        assert_eq!(from_f80(v).0.to_bits(), bits, "round trip of {bits:#x}");
        // against the hardware: `fld qword; fstp tbyte`
        let mut out = [0u8; 10];
        x87!("fld qword ptr [{x}]", "fstp tbyte ptr [{o}]"; x = &raw const x, o = out.as_mut_ptr());
        assert_eq!(v, out, "f64_to_f80 of {bits:#x}");
        // and back through the hardware: `fld tbyte; fstp qword`, rounding to nearest even
        let mut raw = [0u8; 10];
        raw[..8].copy_from_slice(&(g.next() | (1 << 63)).to_le_bytes());
        let exp = (0x3C00 + g.below(0x0800) as u16) | ((g.below(2) as u16) << 15);
        raw[8..].copy_from_slice(&exp.to_le_bytes());
        let mut back = 0.0f64;
        x87!("fld tbyte ptr [{v}]", "fstp qword ptr [{o}]"; v = raw.as_ptr(), o = &raw mut back);
        assert_eq!(
            from_f80(raw).0.to_bits(),
            back.to_bits(),
            "f80_to_f64 of {raw:x?}"
        );
    }
}
