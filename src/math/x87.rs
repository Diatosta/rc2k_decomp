//! Arithmetic done with the original's own floating point instructions, where a result
//! has to be the original's to the last bit: the x87 rounds through its wider
//! registers, and `rcpss` is an approximation that differs between processors.
//! All values are floats given and returned as their bits.

macro_rules! x87 {
    ($($line:literal),+; $($name:ident = $value:expr),+) => {
        unsafe {
            core::arch::asm!(
                $($line),+,
                $($name = in(reg) $value),+,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            )
        }
    };
}
pub(crate) use x87;

/// `fld a; fadd b; fstp`.
pub fn add(a: u32, b: u32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fld dword ptr [{v}]", "fadd dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fsub b; fstp`.
pub fn sub(a: u32, b: u32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fstp`: a whole number as a float.
pub fn from_int(a: i32) -> u32 {
    let mut out = 0u32;
    x87!("fild dword ptr [{a}]", "fstp dword ptr [{o}]"; a = &raw const a, o = &raw mut out);
    out
}

/// `fld a; fmul b; fstp`.
pub fn mul(a: u32, b: u32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld1; fdiv a; fmul b; fstp`.
pub fn reciprocal_times(a: u32, b: u32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fld1", "fdiv dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld x; fsub a; fsub b; fchs; fadd b; fadd a; fstp`: x mirrored about a + b.
pub fn mirrored(x: u32, a: u32, b: u32) -> u32 {
    let (v, mut out) = ([x, a, b], 0u32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fsub dword ptr [{v} + 8]", "fchs",
        "fadd dword ptr [{v} + 8]", "fadd dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fstp`: a float as it comes back from a register.
pub fn loaded(a: u32) -> u32 {
    let mut out = 0u32;
    x87!("fld dword ptr [{a}]", "fstp dword ptr [{o}]"; a = &raw const a, o = &raw mut out);
    out
}

/// `fld a; fistp`: the whole number, rounded as the x87 is set to round.
pub fn to_int(a: u32) -> i32 {
    let mut out = 0i32;
    x87!("fld dword ptr [{a}]", "fistp dword ptr [{o}]"; a = &raw const a, o = &raw mut out);
    out
}

/// `fld a; fmul b; fistp`: the product as a whole number.
pub fn mul_to_int(a: u32, b: u32) -> i32 {
    let (v, mut out) = ([a, b], 0i32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fild b; fdivp st(1), st; fstp`: a over b, two whole numbers, as a float.
pub fn int_ratio(a: i32, b: i32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fild dword ptr [{v}]", "fild dword ptr [{v} + 4]", ".byte 0xDE, 0xF9",
        "fstp dword ptr [{o}]"; v = v.as_ptr(), o = &raw mut out);
    out
}

/// The same and `fld1; fsubrp st(1), st`: 1 less a over b.
pub fn one_less_int_ratio(a: i32, b: i32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fild dword ptr [{v}]", "fild dword ptr [{v} + 4]", ".byte 0xDE, 0xF9",
        ".byte 0xD9, 0xE8", ".byte 0xDE, 0xE1", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `rcpss xmm0, a; mulss xmm0, b`.
pub fn rcpss_times(a: u32, b: u32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    unsafe {
        core::arch::asm!(
            "rcpss xmm0, dword ptr [{v}]",
            "mulss xmm0, dword ptr [{v} + 4]",
            "movss dword ptr [{o}], xmm0",
            v = in(reg) v.as_ptr(),
            o = in(reg) &raw mut out,
            out("xmm0") _,
            options(nostack),
        );
    }
    out
}

/// `fld a[0]; fld b[0]; fmulp; fld a[1]; fld b[4]; fmulp; fld a[2]; fld b[8]; fmulp;
/// faddp; faddp; fstp out`: a row of one matrix times a column of another, the last
/// two products added first.
pub unsafe fn row_times_column(a: *const u32, b: *const u32, out: *mut u32) {
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
pub unsafe fn add_into(a: *const u32, out: *mut u32) {
    x87!("fld dword ptr [{a}]", "fadd dword ptr [{o}]", "fstp dword ptr [{o}]"; a = a, o = out);
}

/// `fild a; fld1; fadd st, st` four times`; fdivp st(1), st; fstp`: a whole number
/// over sixteen.
pub fn int_over_16(a: i32) -> u32 {
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
pub fn int_hypot(a: i32, b: i32) -> i32 {
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
pub fn div(a: u32, b: u32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fld dword ptr [{v}]", "fdiv dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul b; fld c; fmul d; fsubp st(1), st; fstp`: a times b less c times d.
pub fn product_difference(a: u32, b: u32, c: u32, d: u32) -> u32 {
    let (v, mut out) = ([a, b, c, d], 0u32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]",
        "fld dword ptr [{v} + 8]", "fmul dword ptr [{v} + 12]", ".byte 0xDE, 0xE9",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// The same with `fchs` before the `fstp`.
pub fn negated_product_difference(a: u32, b: u32, c: u32, d: u32) -> u32 {
    let (v, mut out) = ([a, b, c, d], 0u32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]",
        "fld dword ptr [{v} + 8]", "fmul dword ptr [{v} + 12]", ".byte 0xDE, 0xE9",
        "fchs", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld v[0]; fmul st, st; fld v[1]; fmul st, st; fld v[2]; fmul st, st; faddp; faddp;
/// fsqrt; fstp`: the length of three floats.
pub unsafe fn length(v: *const u32) -> u32 {
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
pub fn equal_or_unordered(a: u32, b: u32) -> bool {
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
pub fn sine_cosine(angle: i32, a: u32, b: u32) -> (u32, u32) {
    let (v, mut out) = ([angle as u32, a, b], [0u32; 2]);
    x87!("fild dword ptr [{v}]", "fdiv dword ptr [{v} + 4]", "fdiv dword ptr [{v} + 4]",
        "fdiv dword ptr [{v} + 8]", "fsincos",
        "fstp dword ptr [{o} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    (out[0], out[1])
}

/// `fild a; fdiv d; fsqrt; fstp`: the root of a whole number over a float.
pub fn int_over_root(a: i32, d: u32) -> u32 {
    let (v, mut out) = ([a as u32, d], 0u32);
    x87!("fild dword ptr [{v}]", "fdiv dword ptr [{v} + 4]", "fsqrt", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fsqrt; fstp`.
pub fn sqrt(a: u32) -> u32 {
    let mut out = 0u32;
    x87!("fld dword ptr [{a}]", "fsqrt", "fstp dword ptr [{o}]";
        a = &raw const a, o = &raw mut out);
    out
}

/// `fild n; fldpi; fmulp st(1), st; fild d; fdiv st(1), st; fstp (away); fsin; fstp`:
/// the sine of n times pi over d.
pub fn sine_of_pi_fraction(n: i32, d: i32) -> u32 {
    let (v, mut out) = ([n, d], [0u32; 2]);
    x87!("fild dword ptr [{v}]", "fldpi", ".byte 0xDE, 0xC9",
        "fild dword ptr [{v} + 4]", ".byte 0xDC, 0xF9", "fstp dword ptr [{o} + 4]",
        "fsin", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    out[0]
}

/// `fld a; fld b; fdivp st(1), st; fild m; fmulp st(1), st; fild n; fmulp st(1), st;
/// fistp`: a over b times two whole numbers, as a whole number.
pub fn ratio_scaled_to_int(a: u32, b: u32, m: i32, n: i32) -> i32 {
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
pub fn angle_scaled_to_int(y: i32, x: i32, a: u32, b: u32) -> i32 {
    let (v, mut out) = ([y as u32, x as u32, a, b], 0i32);
    x87!("fild dword ptr [{v}]", "fild dword ptr [{v} + 4]", "fpatan", "fldpi",
        ".byte 0xDE, 0xF9", "fmul dword ptr [{v} + 8]", "fmul dword ptr [{v} + 12]",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fmul b; fstp`: a whole number times a float.
pub fn int_times(a: i32, b: u32) -> u32 {
    let (v, mut out) = ([a as u32, b], 0u32);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fdiv b; fstp`: a whole number over a float.
pub fn int_over(a: i32, b: u32) -> u32 {
    let (v, mut out) = ([a as u32, b], 0u32);
    x87!("fild dword ptr [{v}]", "fdiv dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fsub b; fstp`: a whole number less a float.
pub fn int_less(a: i32, b: u32) -> u32 {
    let (v, mut out) = ([a as u32, b], 0u32);
    x87!("fild dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld1; fdiv a; fstp`.
pub fn reciprocal(a: u32) -> u32 {
    let mut out = 0u32;
    x87!("fld1", "fdiv dword ptr [{a}]", "fstp dword ptr [{o}]";
        a = &raw const a, o = &raw mut out);
    out
}

/// `fild a; fmul b; fild c; fmulp st(1), st; fmul d; fstp`: two whole numbers and two
/// floats multiplied.
pub fn int_float_int_float(a: i32, b: u32, c: i32, d: u32) -> u32 {
    let (v, mut out) = ([a as u32, b, c as u32, d], 0u32);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fild dword ptr [{v} + 8]",
        ".byte 0xDE, 0xC9", "fmul dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul b; fdiv c; fdiv d; fistp`.
pub fn mul_div_div_to_int(a: u32, b: u32, c: u32, d: u32) -> i32 {
    let (v, mut out) = ([a, b, c, d], 0i32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fdiv dword ptr [{v} + 8]",
        "fdiv dword ptr [{v} + 12]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `sub_41FC4D`'s sequence: x, y, z squared and summed (`faddp` twice), `fsqrt`, `fcom`
/// with `zero`; unless equal, each of the three divided by the root still on the stack
/// (80 bits, not rounded first: `fdiv st, st(1)`) and stored back; then the root stored.
/// Returns the root's bits and whether it compared equal.
pub unsafe fn normalise_in_place(v: *mut u32, zero: u32) -> (u32, bool) {
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
pub fn int_product_difference(a: i32, b: i32, c: i32, d: i32) -> u32 {
    let (v, mut out) = ([a, b, c, d], 0u32);
    x87!("fild dword ptr [{v}]", "fild dword ptr [{v} + 4]", ".byte 0xDE, 0xC9",
        "fild dword ptr [{v} + 8]", "fild dword ptr [{v} + 12]", ".byte 0xDE, 0xC9",
        ".byte 0xDE, 0xE9", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fdiv b; fstp`: a whole number divided by a float, rounded once.
pub fn int_div(a: i32, b: u32) -> u32 {
    let (ia, fb, mut out) = (a, b, 0u32);
    x87!("fild dword ptr [{a}]", "fdiv dword ptr [{b}]", "fstp dword ptr [{o}]";
        a = &raw const ia, b = &raw const fb, o = &raw mut out);
    out
}

/// `fild a; fdiv b; fdiv c; fadd st, st; fstp`: a whole number over two floats, doubled,
/// rounded once.
pub fn int_div_div_doubled(a: i32, b: u32, c: u32) -> u32 {
    let (ia, v, mut out) = (a, [b, c], 0u32);
    x87!("fild dword ptr [{a}]", "fdiv dword ptr [{v}]", "fdiv dword ptr [{v} + 4]",
        "fadd st(0), st(0)", "fstp dword ptr [{o}]";
        a = &raw const ia, v = v.as_ptr(), o = &raw mut out);
    out
}

/// `math::turn_scratch_xz`'s turn, its stack steps as the original's bytes: x' = x cos +
/// z sin, z' = z cos - x sin, each rounded once, written back over x and z.
pub unsafe fn turn_in_place(sine: *const u32, cosine: *const u32, x: *mut u32, z: *mut u32) {
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
pub fn surface_share(a: i32, b: u32, c: u32, d: i32, e: u32, f: u32) -> u32 {
    let (v, mut out) = ([a as u32, b, c, d as u32, e, f], 0u32);
    x87!("fild dword ptr [{v}]", "fdiv dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fidiv dword ptr [{v} + 12]", "fmul dword ptr [{v} + 16]", "fadd dword ptr [{v} + 20]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild y; fild x; fpatan; fldpi; fdivp st(1), st; fmul a; fmul b; fmul c; fistp`.
pub fn angle_scaled3_to_int(y: i32, x: i32, a: u32, b: u32, c: u32) -> i32 {
    let (v, mut out) = ([y as u32, x as u32, a, b, c], 0i32);
    x87!("fild dword ptr [{v}]", "fild dword ptr [{v} + 4]", "fpatan", "fldpi",
        ".byte 0xDE, 0xF9", "fmul dword ptr [{v} + 8]", "fmul dword ptr [{v} + 12]",
        "fmul dword ptr [{v} + 16]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild y; fild x; fpatan; fldpi; fdivp st(1), st; fmul a; fmul a; fmul a; fmul b;
/// fmul b; fistp`.
pub fn angle_scaled5_to_int(y: i32, x: i32, a: u32, b: u32) -> i32 {
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
pub fn hypot_ratio(a: i32, b: i32, c: i32, d: i32) -> u32 {
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
pub fn int_times_to_int(a: i32, b: u32) -> i32 {
    let (v, mut out) = ([a as u32, b], 0i32);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fdiv b; fsub c; fstp`: a whole number over a float, less a float, rounded once.
pub fn int_div_less(a: i32, b: u32, c: u32) -> u32 {
    let (v, mut out) = ([a as u32, b, c], 0u32);
    x87!("fild dword ptr [{v}]", "fdiv dword ptr [{v} + 4]", "fsub dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fldz`, then `count` times `fld st; fsin; fstp [out]; fadd step` (the angle summed in the
/// register, never rounded to a float), then `ffree st; fincstp`.
pub unsafe fn sine_table(step: u32, out: *mut u32, count: u32) {
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
        )
    }
}

/// `fld1; fidiv a; fst first; fmul b; fstp second`: one over a whole number, and that
/// (unrounded) times a float.
pub fn reciprocal_int_and_times(a: i32, b: u32) -> (u32, u32) {
    let (v, mut out) = ([a as u32, b], [0u32; 2]);
    x87!("fld1", "fidiv dword ptr [{v}]", "fst dword ptr [{o}]", "fmul dword ptr [{v} + 4]",
        "fstp dword ptr [{o} + 4]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    (out[0], out[1])
}

/// `fld a; fmul b; fst first; fdiv c; fstp second`: a times b, and that (unrounded) over c.
pub fn mul_and_div(a: u32, b: u32, c: u32) -> (u32, u32) {
    let (v, mut out) = ([a, b, c], [0u32; 2]);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fst dword ptr [{o}]",
        "fdiv dword ptr [{v} + 8]", "fstp dword ptr [{o} + 4]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    (out[0], out[1])
}

/// `fld a; fsub b; fdiv c; fmul d; fstp`.
pub fn sub_div_mul(a: u32, b: u32, c: u32, d: u32) -> u32 {
    let (v, mut out) = ([a, b, c, d], 0u32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fdiv dword ptr [{v} + 8]",
        "fmul dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fsub b; fld c; fsub d; fdivp st(1), st; fmul e; fstp`: (a - b) / (c - d) * e.
pub fn diff_ratio_mul(a: u32, b: u32, c: u32, d: u32, e: u32) -> u32 {
    let (v, mut out) = ([a, b, c, d, e], 0u32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fld dword ptr [{v} + 8]",
        "fsub dword ptr [{v} + 12]", ".byte 0xDE, 0xF9", "fmul dword ptr [{v} + 16]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fsub b; fmul m; fadd b; fmul n; fsub less; fstp`: b + (a - b) m, times n, less
/// `less` (0.0 for none), rounded once.
pub fn lerp_scaled_less(a: u32, b: u32, m: u32, n: u32, less: Option<u32>) -> u32 {
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
pub fn negated_sub(a: u32, b: u32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fchs", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fmul x; fild b; fmul y; fild c; fmul z; faddp st(1), st; faddp st(1), st;
/// fistp`: a x + (b y + c z), whole numbers out.
pub fn weighted3_to_int(a: i32, b: i32, c: i32, x: u32, y: u32, z: u32) -> i32 {
    let (v, mut out) = ([a as u32, b as u32, c as u32, x, y, z], 0i32);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 12]", "fild dword ptr [{v} + 4]",
        "fmul dword ptr [{v} + 16]", "fild dword ptr [{v} + 8]", "fmul dword ptr [{v} + 20]",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fmul b; fadd c; fstp`.
pub fn int_times_plus(a: i32, b: u32, c: u32) -> u32 {
    let (v, mut out) = ([a as u32, b, c], 0u32);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild last; fisub first; fidiv by; fldz`, then for each of `count` whole numbers at
/// `values`: `fild; fsub st, st(1); fistp` (less the sum), `fadd st, st(1)` (the sum on by
/// the step), both kept in the registers; then `ffree st; fincstp` twice.
pub unsafe fn ramp_down(values: *mut i32, count: u32, last: i32, first: i32, by: i32) {
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
        )
    }
}

/// `fild a; fisub b; fstp`: the difference of two whole numbers, rounded once.
pub fn int_less_int(a: i32, b: i32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fild dword ptr [{v}]", "fisub dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fisub b; fdiv c; fstp`.
pub fn int_less_int_over(a: i32, b: i32, c: u32) -> u32 {
    let (v, mut out) = ([a as u32, b as u32, c], 0u32);
    x87!("fild dword ptr [{v}]", "fisub dword ptr [{v} + 4]", "fdiv dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fsub b; fdiv c; fstp`.
pub fn sub_div(a: u32, b: u32, c: u32) -> u32 {
    let (v, mut out) = ([a, b, c], 0u32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fdiv dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fsin; fmul b; fadd c; fistp`.
pub fn sine_times_plus_to_int(a: u32, b: u32, c: u32) -> i32 {
    let (v, mut out) = ([a, b, c], 0i32);
    x87!("fld dword ptr [{v}]", "fsin", "fmul dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul st, st; fchs; fmul b; fadd c; fistp`: c - a² b.
pub fn negated_square_times_plus_to_int(a: u32, b: u32, c: u32) -> i32 {
    let (v, mut out) = ([a, b, c], 0i32);
    x87!("fld dword ptr [{v}]", ".byte 0xD8, 0xC8", "fchs", "fmul dword ptr [{v} + 4]",
        "fadd dword ptr [{v} + 8]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild x; fmul st, st; fild y; fmul st, st; fild z; fmul st, st; faddp; faddp; fsqrt;
/// fiadd sum; fistp`: the sum plus the length of (x, y, z).
pub fn length_plus_int(x: i32, y: i32, z: i32, sum: i32) -> i32 {
    let (v, mut out) = ([x, y, z, sum], 0i32);
    x87!("fild dword ptr [{v}]", ".byte 0xD8, 0xC8", "fild dword ptr [{v} + 4]",
        ".byte 0xD8, 0xC8", "fild dword ptr [{v} + 8]", ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fsqrt", "fiadd dword ptr [{v} + 12]",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fidiv b; fmul c; fsubr c; fistp`: c less a / b of c, a whole number.
pub fn share_left_to_int(a: i32, b: i32, c: u32) -> i32 {
    let (v, mut out) = ([a as u32, b as u32, c], 0i32);
    x87!("fild dword ptr [{v}]", "fidiv dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fsubr dword ptr [{v} + 8]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `finit` (9B DB E3): the x87 reset, its control word 0x37F (every exception masked,
/// 64-bit precision, rounding to nearest), as the original does it.
pub fn finit() {
    unsafe {
        core::arch::asm!(
            ".byte 0x9B, 0xDB, 0xE3",
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The x87 control word (`fnstcw`).
pub fn control_word() -> u16 {
    let mut w = 0u16;
    unsafe { core::arch::asm!("fnstcw word ptr [{w}]", w = in(reg) &raw mut w, options(nostack)) };
    w
}

/// `fldcw`: for the tests, the control word put back between the two runs.
pub fn set_control_word(w: u16) {
    unsafe { core::arch::asm!("fldcw word ptr [{w}]", w = in(reg) &raw const w, options(nostack)) };
}

/// `row_times_column` for 3 x 3 matrices: `b`'s column read 12 bytes apart.
pub unsafe fn row_times_column_3(a: *const u32, b: *const u32, out: *mut u32) {
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
pub unsafe fn turn_int_point(p: *const i32, m: *const u32, out: *mut i32) {
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
pub fn mul2(a: u32, b: u32, c: u32) -> u32 {
    let mut out = 0u32;
    x87!(
        "fld dword ptr [{a}]", "fmul dword ptr [{b}]", "fmul dword ptr [{c}]",
        "fstp dword ptr [{o}]";
        a = &a, b = &b, c = &c, o = &mut out
    );
    out
}

/// `fld1; fild i; fsub a; fmul b; fmul st, st; fsubr st, st(1); fsqrt; fsubp st(1), st`,
/// negated (`fchs`) when `negate`: 1 - sqrt(1 - ((i - a) b)^2), a quarter circle's curve.
pub fn circle_curve(i: i32, a: u32, b: u32, negate: bool) -> u32 {
    let mut out = 0u32;
    let n = negate as u32;
    unsafe {
        core::arch::asm!(
            "fld1", "fild dword ptr [{i}]", "fsub dword ptr [{a}]", "fmul dword ptr [{b}]",
            ".byte 0xD8, 0xC8", ".byte 0xD8, 0xE9", "fsqrt", ".byte 0xDE, 0xE9",
            "test {n}, {n}", "jz 2f", "fchs", "2:",
            "fstp dword ptr [{o}]",
            i = in(reg) &i, a = in(reg) &a, b = in(reg) &b, n = in(reg) n, o = in(reg) &mut out,
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
pub unsafe fn circle_points(
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
pub fn negated(a: u32) -> u32 {
    let mut out = 0u32;
    x87!("fld dword ptr [{a}]", "fchs", "fstp dword ptr [{o}]"; a = &a, o = &mut out);
    out
}

/// `fld b; fld a; fsubp st(1), st; fmul k; fstp`: (b - a) k.
pub fn sub_mul(b: u32, a: u32, k: u32) -> u32 {
    let mut out = 0u32;
    x87!(
        "fld dword ptr [{b}]", "fld dword ptr [{a}]", ".byte 0xDE, 0xE9",
        "fmul dword ptr [{k}]", "fstp dword ptr [{o}]";
        b = &b, a = &a, k = &k, o = &mut out
    );
    out
}

/// `fldpi; fidiv c; fimul m; fcos; fchs; fmul t; fadd a; fadd t; fstp`:
/// a + t - t cos(pi m / c).
pub fn cosine_step(c: i32, m: i32, t: u32, a: u32) -> u32 {
    let mut out = 0u32;
    x87!(
        "fldpi", "fidiv dword ptr [{c}]", "fimul dword ptr [{m}]", "fcos", "fchs",
        "fmul dword ptr [{t}]", "fadd dword ptr [{a}]", "fadd dword ptr [{t}]",
        "fstp dword ptr [{o}]";
        c = &c, m = &m, t = &t, a = &a, o = &mut out
    );
    out
}

/// Three floats at `row` each times `c` over their length (`fsqrt`, the last two squares
/// added first; `fdivr c`; `fld st` twice, each multiplied and stored in turn).
pub unsafe fn scale_row(row: *mut u32, c: u32) {
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
        r = row, c = &c
    );
}

/// `fld y; fist n; fisub n; fsubr k; fmul s; fadd x; fstp`: x + (k - (y - n)) s, n the
/// whole number y rounds to (as the x87 is set), which is given back too.
pub fn frac_step(y: u32, k: u32, s: u32, x: u32) -> (i32, u32) {
    let (mut n, mut out) = (0i32, 0u32);
    x87!(
        "fld dword ptr [{y}]", "fist dword ptr [{n}]", "fisub dword ptr [{n}]",
        "fsubr dword ptr [{k}]", "fmul dword ptr [{s}]", "fadd dword ptr [{x}]",
        "fstp dword ptr [{o}]";
        y = &y, n = &mut n, k = &k, s = &s, x = &x, o = &mut out
    );
    (n, out)
}

/// `fld a; frndint; fstp`.
pub fn round_float(a: u32) -> u32 {
    let mut out = 0u32;
    x87!("fld dword ptr [{a}]", "frndint", "fstp dword ptr [{o}]"; a = &a, o = &mut out);
    out
}

/// `fld a; fcomp b; fstsw`: the status word's C0 .. C2 (0x700).
pub fn compare_status(a: u32, b: u32) -> u16 {
    let mut status: u16 = 0;
    x87!(
        "fld dword ptr [{a}]", "fcomp dword ptr [{b}]", "fstsw word ptr [{s}]";
        a = &a, b = &b, s = &mut status
    );
    status & 0x700
}

/// `fld a; fisub n; fistp`.
pub fn float_less_int_to_int(a: u32, n: i32) -> i32 {
    let mut out = 0i32;
    x87!(
        "fld dword ptr [{a}]", "fisub dword ptr [{n}]", "fistp dword ptr [{o}]";
        a = &a, n = &n, o = &mut out
    );
    out
}

/// `fld1; fld c; fsub a; fdivp st(1), st; fld b; fsub a; fmulp; fld x2; fsub x0; fmulp;
/// fadd x0; fstp`: x0 + ((1 / (c - a)) (b - a)) (x2 - x0), where the long edge is at b.
pub fn split_x(a: u32, b: u32, c: u32, x0: u32, x2: u32) -> u32 {
    let mut out = 0u32;
    x87!(
        "fld1", "fld dword ptr [{c}]", "fsub dword ptr [{a}]", ".byte 0xDE, 0xF9",
        "fld dword ptr [{b}]", "fsub dword ptr [{a}]", ".byte 0xDE, 0xC9",
        "fld dword ptr [{x2}]", "fsub dword ptr [{x0}]", ".byte 0xDE, 0xC9",
        "fadd dword ptr [{x0}]", "fstp dword ptr [{o}]";
        a = &a, b = &b, c = &c, x0 = &x0, x2 = &x2, o = &mut out
    );
    out
}

/// `fld b; fsub a; fmul k; fst h; fadd a; fstp m`: the half (b - a) k as a float, and a plus
/// it unrounded, then as a float.
pub fn half_and_middle(b: u32, a: u32, k: u32) -> (u32, u32) {
    let (mut h, mut m) = (0u32, 0u32);
    x87!(
        "fld dword ptr [{b}]", "fsub dword ptr [{a}]", "fmul dword ptr [{k}]",
        "fst dword ptr [{h}]", "fadd dword ptr [{a}]", "fstp dword ptr [{m}]";
        b = &b, a = &a, k = &k, h = &mut h, m = &mut m
    );
    (h, m)
}

/// `fldpi; fidiv n; fstp`.
pub fn pi_over(n: i32) -> u32 {
    let mut out = 0u32;
    x87!("fldpi", "fidiv dword ptr [{n}]", "fstp dword ptr [{o}]"; n = &n, o = &mut out);
    out
}

/// `fld a; [fadd s;] fcos; fchs; fmul h; fadd m; fstp`: m - h cos(a [+ s]).
pub fn less_cosine(a: u32, s: Option<u32>, h: u32, m: u32) -> u32 {
    let mut out = 0u32;
    match s {
        Some(s) => x87!(
            "fld dword ptr [{a}]", "fadd dword ptr [{s}]", "fcos", "fchs",
            "fmul dword ptr [{h}]", "fadd dword ptr [{m}]", "fstp dword ptr [{o}]";
            a = &a, s = &s, h = &h, m = &m, o = &mut out
        ),
        None => x87!(
            "fld dword ptr [{a}]", "fcos", "fchs",
            "fmul dword ptr [{h}]", "fadd dword ptr [{m}]", "fstp dword ptr [{o}]";
            a = &a, h = &h, m = &m, o = &mut out
        ),
    }
    out
}

/// `fld a; fsin; fstp`.
pub fn sine(a: u32) -> u32 {
    let mut out = 0u32;
    x87!("fld dword ptr [{a}]", "fsin", "fstp dword ptr [{o}]"; a = &a, o = &mut out);
    out
}

/// `fld a; fcos; fstp`, and with `negate` an `fchs` before the store.
pub fn cosine(a: u32, negate: bool) -> u32 {
    let mut out = 0u32;
    if negate {
        x87!("fld dword ptr [{a}]", "fcos", "fchs", "fstp dword ptr [{o}]"; a = &a, o = &mut out);
    } else {
        x87!("fld dword ptr [{a}]", "fcos", "fstp dword ptr [{o}]"; a = &a, o = &mut out);
    }
    out
}

/// `fld a; fmul b; fadd c; fstp`.
pub fn mul_add(a: u32, b: u32, c: u32) -> u32 {
    let mut out = 0u32;
    x87!(
        "fld dword ptr [{a}]", "fmul dword ptr [{b}]", "fadd dword ptr [{c}]",
        "fstp dword ptr [{o}]";
        a = &a, b = &b, c = &c, o = &mut out
    );
    out
}

/// `fild a; fmul st, st; fild b; fmul st, st; fild c; fmul st, st; faddp; faddp; fsqrt;
/// fistp`: the length of (a, b, c), the last two squares added first, rounded.
pub fn int_length3(a: i32, b: i32, c: i32) -> i32 {
    let (v, mut out) = ([a, b, c], 0i32);
    x87!(
        "fild dword ptr [{v}]", ".byte 0xD8, 0xC8",
        "fild dword ptr [{v} + 4]", ".byte 0xD8, 0xC8",
        "fild dword ptr [{v} + 8]", ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fsqrt", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &mut out
    );
    out
}

/// `fild a; fiadd b; fistp`: the sum, or 0x80000000 when it does not fit.
pub fn int_sum(a: i32, b: i32) -> i32 {
    let mut out = 0i32;
    x87!(
        "fild dword ptr [{a}]", "fiadd dword ptr [{b}]", "fistp dword ptr [{o}]";
        a = &a, b = &b, o = &mut out
    );
    out
}

/// `fild x0; fmul st, st; fild y0; fmul st, st; faddp; fild x1; .. faddp; fcompp; fstsw`:
/// x1² + y1² against x0² + y0², the status word's C0 .. C2.
pub fn compare_squares(x0: i32, y0: i32, x1: i32, y1: i32) -> u16 {
    let (v, mut status) = ([x0, y0, x1, y1], 0u16);
    x87!(
        "fild dword ptr [{v}]", ".byte 0xD8, 0xC8", "fild dword ptr [{v} + 4]", ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1",
        "fild dword ptr [{v} + 8]", ".byte 0xD8, 0xC8", "fild dword ptr [{v} + 12]", ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1",
        "fcompp", "fstsw word ptr [{s}]";
        v = v.as_ptr(), s = &mut status
    );
    status & 0x700
}

/// `fild a; fidiv n; fstp`.
pub fn int_div_int(a: i32, n: i32) -> u32 {
    let mut out = 0u32;
    x87!("fild dword ptr [{a}]", "fidiv dword ptr [{n}]", "fstp dword ptr [{o}]"; a = &a, n = &n, o = &mut out);
    out
}

/// `fld a; fsub b`, kept as an 80-bit value.
pub fn sub_80(a: u32, b: u32) -> [u8; 10] {
    let mut out = [0u8; 10];
    x87!("fld dword ptr [{a}]", "fsub dword ptr [{b}]", "fstp tbyte ptr [{o}]"; a = &a, b = &b, o = out.as_mut_ptr());
    out
}

/// `fild a; fsub b`, kept as an 80-bit value.
pub fn int_sub_80(a: i32, b: u32) -> [u8; 10] {
    let mut out = [0u8; 10];
    x87!("fild dword ptr [{a}]", "fsub dword ptr [{b}]", "fstp tbyte ptr [{o}]"; a = &a, b = &b, o = out.as_mut_ptr());
    out
}

/// `fld a; fadd b; fistp`.
pub fn add_to_int(a: u32, b: u32) -> i32 {
    let mut out = 0i32;
    x87!("fld dword ptr [{a}]", "fadd dword ptr [{b}]", "fistp dword ptr [{o}]"; a = &a, b = &b, o = &mut out);
    out
}

/// `fld a; fsub b; [fchs;] fcomp c; fstsw`: the status word's C0 .. C2.
pub fn sub_compare(a: u32, b: u32, negate: bool, c: u32) -> u16 {
    let mut status: u16 = 0;
    if negate {
        x87!(
            "fld dword ptr [{a}]", "fsub dword ptr [{b}]", "fchs", "fcomp dword ptr [{c}]",
            "fstsw word ptr [{s}]";
            a = &a, b = &b, c = &c, s = &mut status
        );
    } else {
        x87!(
            "fld dword ptr [{a}]", "fsub dword ptr [{b}]", "fcomp dword ptr [{c}]",
            "fstsw word ptr [{s}]";
            a = &a, b = &b, c = &c, s = &mut status
        );
    }
    status & 0x700
}

/// `fild r; fmul m; fadd c; fld a; fsub b; fmul s; fadd e; fmul k; fmulp; fstp` with `v`
/// [m, c, a, b, s, e, k]: (r m + c) ((a - b) s + e) k, the factors unrounded.
pub fn random_step(r: i32, v: [u32; 7]) -> u32 {
    let mut out = 0u32;
    x87!(
        "fild dword ptr [{r}]", "fmul dword ptr [{v}]", "fadd dword ptr [{v} + 4]",
        "fld dword ptr [{v} + 8]", "fsub dword ptr [{v} + 12]", "fmul dword ptr [{v} + 16]",
        "fadd dword ptr [{v} + 20]", "fmul dword ptr [{v} + 24]", ".byte 0xDE, 0xC9",
        "fstp dword ptr [{o}]";
        r = &r, v = v.as_ptr(), o = &mut out
    );
    out
}

/// `fild y; fadd a; fsub b; fmul st, st; fstp`.
pub fn int_plus_less_squared(y: i32, a: u32, b: u32) -> u32 {
    let mut out = 0u32;
    x87!(
        "fild dword ptr [{y}]", "fadd dword ptr [{a}]", "fsub dword ptr [{b}]",
        ".byte 0xD8, 0xC8", "fstp dword ptr [{o}]";
        y = &y, a = &a, b = &b, o = &mut out
    );
    out
}

/// One of the marks' kind records (0x30 bytes, 12 floats): +0xC and +0x24 negated in
/// place, then each of the twelve times a / b (kept unrounded) rounded to a whole number in
/// place (`fmul st, st(1)`; the last `fmulp`).
pub unsafe fn scale_record(r: *mut u32, a: u32, b: u32) {
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
        r = r, a = &a, b = &b
    );
}

/// `fild a; fmul m; fadd c; fst` (the first), then `fadd d; fsub e; fstp` (the second).
pub fn int_times_plus_on(a: i32, m: u32, c: u32, d: u32, e: u32) -> (u32, u32) {
    let (v, mut out) = ([a as u32, m, c, d, e], [0u32; 2]);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]",
        "fst dword ptr [{o}]", "fadd dword ptr [{v} + 12]", "fsub dword ptr [{v} + 16]",
        "fstp dword ptr [{o} + 4]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    (out[0], out[1])
}

/// `fld a; fsub b; fsub b; fsub b; fstp`.
pub fn sub_thrice(a: u32, b: u32) -> u32 {
    let (v, mut out) = ([a, b], 0u32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fsub dword ptr [{v} + 4]",
        "fsub dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fadd b; fstp`.
pub fn int_plus(a: i32, b: u32) -> u32 {
    let (v, mut out) = ([a as u32, b], 0u32);
    x87!("fild dword ptr [{v}]", "fadd dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld t; fsub c; fmul k; fadd c; fstp`: c eased toward t by k.
pub fn ease(t: u32, c: u32, k: u32) -> u32 {
    let (v, mut out) = ([t, c, k], 0u32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fadd dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fadd b; fistp`.
pub fn int_plus_to_int(a: i32, b: u32) -> i32 {
    let (v, mut out) = ([a as u32, b], 0i32);
    x87!("fild dword ptr [{v}]", "fadd dword ptr [{v} + 4]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fldz; fisub a; fiadd b; fistp`.
pub fn zero_less_int_plus_int(a: i32, b: i32) -> i32 {
    let (v, mut out) = ([a, b], 0i32);
    x87!("fldz", "fisub dword ptr [{v}]", "fiadd dword ptr [{v} + 4]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld f; fisub a; fiadd b; fistp`.
pub fn less_int_plus_int(f: u32, a: i32, b: i32) -> i32 {
    let (v, mut out) = ([f, a as u32, b as u32], 0i32);
    x87!("fld dword ptr [{v}]", "fisub dword ptr [{v} + 4]", "fiadd dword ptr [{v} + 8]",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fldpi; fadd st, st; fsub a; fstp`: 2 pi less a.
pub fn two_pi_less(a: u32) -> u32 {
    let mut out = 0u32;
    x87!("fldpi", ".byte 0xD8, 0xC0", "fsub dword ptr [{a}]", "fstp dword ptr [{o}]";
        a = &a, o = &raw mut out);
    out
}

/// `fild a; fmul m; fadd c; fmul k; fstp`.
pub fn int_times_plus_times(a: i32, m: u32, c: u32, k: u32) -> u32 {
    let (v, mut out) = ([a as u32, m, c, k], 0u32);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]",
        "fmul dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fidiv n; fstp`.
pub fn over_int(a: u32, n: i32) -> u32 {
    let (v, mut out) = ([a, n as u32], 0u32);
    x87!("fld dword ptr [{v}]", "fidiv dword ptr [{v} + 4]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fldz; [fchs;] fmul b; fxch st(1); fmul c; faddp st(1), st; fmul k; fstp`:
/// (±0 b + a c) k, the zero's product kept.
pub fn zero_times_plus_times(a: u32, b: u32, c: u32, k: u32, negative: bool) -> u32 {
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
pub fn prem_plus(r: i32, m: u32, v: u32) -> u32 {
    let (w, mut out) = ([m, r as u32, v], 0u32);
    x87!("fld dword ptr [{w}]", "fild dword ptr [{w} + 4]", "fprem", "fadd dword ptr [{w} + 8]",
        "fstp dword ptr [{o}]", "fstp st(0)";
        w = w.as_ptr(), o = &raw mut out);
    out
}

/// `fild r; fmul qword k; fld v; fmul a; fmulp st(1), st; fadd v; fstp`: v grown by
/// v a r k.
pub fn grown(r: i32, k: u64, v: u32, a: u32) -> u32 {
    let (w, mut out) = ([r as u32, v, a], 0u32);
    x87!("fild dword ptr [{w}]", "fmul qword ptr [{k}]", "fld dword ptr [{w} + 4]",
        "fmul dword ptr [{w} + 8]", ".byte 0xDE, 0xC9", "fadd dword ptr [{w} + 4]",
        "fstp dword ptr [{o}]";
        w = w.as_ptr(), k = &raw const k, o = &raw mut out);
    out
}

/// `fld a; fsub b; fabs; fmul k1; fmul k2; fadd k3; fdivr one; fsubr one; fmul k4;
/// fistp`: a rate by the difference of a and b.
pub fn rate_to_int(a: u32, b: u32, k1: u32, k2: u32, k3: u32, one: u32, k4: u32) -> i32 {
    let (v, mut out) = ([a, b, k1, k2, k3, one, k4], 0i32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fabs",
        "fmul dword ptr [{v} + 8]", "fmul dword ptr [{v} + 12]", "fadd dword ptr [{v} + 16]",
        "fdivr dword ptr [{v} + 20]", "fsubr dword ptr [{v} + 20]", "fmul dword ptr [{v} + 24]",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fmul m1; fild b; fmul m2; faddp st(1), st; fadd d`, kept as 80 bits.
pub fn pair_sum_80(a: i32, m1: u32, b: i32, m2: u32, d: u32) -> [u8; 10] {
    let (v, mut out) = ([a as u32, m1, b as u32, m2, d], [0u8; 10]);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fild dword ptr [{v} + 8]",
        "fmul dword ptr [{v} + 12]", ".byte 0xDE, 0xC1", "fadd dword ptr [{v} + 16]",
        "fstp tbyte ptr [{o}]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    out
}

/// `fld tbyte e; fsub f; fstp`.
pub fn ext_less(e: [u8; 10], f: u32) -> u32 {
    let mut out = 0u32;
    x87!("fld tbyte ptr [{e}]", "fsub dword ptr [{f}]", "fstp dword ptr [{o}]";
        e = e.as_ptr(), f = &f, o = &raw mut out);
    out
}

/// Three products of three, `fld; fmul; fmul` each, summed by `faddp st(1), st` in
/// order: (p0 + p1) + p2.
pub fn three_products(p: [[u32; 3]; 3]) -> u32 {
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
pub fn times_kept_abs_to_int(a: u32, b: u32, c: u32) -> (u32, i32) {
    let (v, mut kept, mut out) = ([a, b, c], 0u32, 0i32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fst dword ptr [{k}]", "fabs",
        "fmul dword ptr [{v} + 8]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), k = &raw mut kept, o = &raw mut out);
    (kept, out)
}

/// `fld a; fsub b; fmul k1; fld b; fmul k2; faddp st(1), st; fst` (the first), `fabs;
/// fmul c; fistp` (the second).
pub fn blend_kept_abs_to_int(a: u32, b: u32, k1: u32, k2: u32, c: u32) -> (u32, i32) {
    let (v, mut kept, mut out) = ([a, b, k1, k2, c], 0u32, 0i32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fld dword ptr [{v} + 4]", "fmul dword ptr [{v} + 12]", ".byte 0xDE, 0xC1",
        "fst dword ptr [{k}]", "fabs", "fmul dword ptr [{v} + 16]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), k = &raw mut kept, o = &raw mut out);
    (kept, out)
}

/// `fld f; fmul s; fld s; fabs; [fmul m;] fsubp st(1), st; fmul k; fstp`: (f s - |s| [m]) k.
pub fn times_less_abs_times(f: u32, s: u32, m: Option<u32>, k: u32) -> u32 {
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
pub fn less_plus_int_times(d: u32, m: u32, f: u32, i: i32) -> u32 {
    let (v, mut out) = ([d, m, f, i as u32], 0u32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fld dword ptr [{v} + 8]",
        "fimul dword ptr [{v} + 12]", ".byte 0xDE, 0xC1", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul b; fimul i; [fadd c;] fmul s`, kept as 80 bits.
pub fn times_int_plus_times_80(a: u32, b: u32, i: i32, c: Option<u32>, s: u32) -> [u8; 10] {
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
pub fn pair_sum_less(a: i32, m1: u32, b: i32, m2: u32, d: u32, e: u32) -> u32 {
    let (v, mut out) = ([a as u32, m1, b as u32, m2, d, e], 0u32);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fild dword ptr [{v} + 8]",
        "fmul dword ptr [{v} + 12]", ".byte 0xDE, 0xC1", "fadd dword ptr [{v} + 16]",
        "fsub dword ptr [{v} + 20]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul b; fmul c; faddp st(1), st` onto the float d, then `fstp`.
pub fn product_plus(a: u32, b: u32, c: u32, d: u32) -> u32 {
    let (v, mut out) = ([d, a, b, c], 0u32);
    x87!("fld dword ptr [{v}]", "fld dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fmul dword ptr [{v} + 12]", ".byte 0xDE, 0xC1", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld f; fimul i; fistp`.
pub fn float_times_int_to_int(f: u32, i: i32) -> i32 {
    let (v, mut out) = ([f, i as u32], 0i32);
    x87!("fld dword ptr [{v}]", "fimul dword ptr [{v} + 4]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul k1; fmul k2; fadd k3; fdivr one; fsubr one; fmul k4; fistp`: a rate by a.
pub fn speed_rate_to_int(a: u32, k1: u32, k2: u32, k3: u32, one: u32, k4: u32) -> i32 {
    let (v, mut out) = ([a, k1, k2, k3, one, k4], 0i32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fadd dword ptr [{v} + 12]", "fdivr dword ptr [{v} + 16]", "fsubr dword ptr [{v} + 16]",
        "fmul dword ptr [{v} + 20]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fabs; fld b; fabs; faddp; fld a; fsub c; fabs; faddp; fadd d; fist` (the
/// first), `fstp` (the second): |a| + |b| + |a - c| + d.
pub fn abs_sum_to_int_kept(a: u32, b: u32, c: u32, d: u32) -> (i32, u32) {
    let (v, mut n, mut kept) = ([a, b, c, d], 0i32, 0u32);
    x87!("fld dword ptr [{v}]", "fabs", "fld dword ptr [{v} + 4]", "fabs", ".byte 0xDE, 0xC1",
        "fld dword ptr [{v}]", "fsub dword ptr [{v} + 8]", "fabs", ".byte 0xDE, 0xC1",
        "fadd dword ptr [{v} + 12]", "fist dword ptr [{n}]", "fstp dword ptr [{k}]";
        v = v.as_ptr(), n = &raw mut n, k = &raw mut kept);
    (n, kept)
}

/// `fld a; fabs; fmul k1; fld b; fsub c; fabs; faddp; fmul k2; fst` (the first), `fcomp
/// k3; fstsw ax` (the second, the status word).
pub fn abs_blend_compare(a: u32, k1: u32, b: u32, c: u32, k2: u32, k3: u32) -> (u32, u16) {
    let (v, mut kept, mut status) = ([a, k1, b, c, k2, k3], 0u32, 0u16);
    x87!("fld dword ptr [{v}]", "fabs", "fmul dword ptr [{v} + 4]", "fld dword ptr [{v} + 8]",
        "fsub dword ptr [{v} + 12]", "fabs", ".byte 0xDE, 0xC1", "fmul dword ptr [{v} + 16]",
        "fst dword ptr [{k}]", "fcomp dword ptr [{v} + 20]", "fnstsw word ptr [{s}]";
        v = v.as_ptr(), k = &raw mut kept, s = &raw mut status);
    (kept, status)
}

/// `fld v; fmul a; fadd b; fistp`.
pub fn times_plus_to_int(v: u32, a: u32, b: u32) -> i32 {
    let (x, mut out) = ([v, a, b], 0i32);
    x87!("fld dword ptr [{x}]", "fmul dword ptr [{x} + 4]", "fadd dword ptr [{x} + 8]",
        "fistp dword ptr [{o}]";
        x = x.as_ptr(), o = &raw mut out);
    out
}

/// `fld d; fld f; fimul i; faddp st(1), st`, kept as 80 bits.
pub fn plus_int_times_80(d: u32, f: u32, i: i32) -> [u8; 10] {
    let (v, mut out) = ([d, f, i as u32], [0u8; 10]);
    x87!("fld dword ptr [{v}]", "fld dword ptr [{v} + 4]", "fimul dword ptr [{v} + 8]",
        ".byte 0xDE, 0xC1", "fstp tbyte ptr [{o}]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    out
}

/// `three_products` kept as 80 bits.
pub fn three_products_80(p: [[u32; 3]; 3]) -> [u8; 10] {
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
pub fn times_compare(a: u32, k: u32, c: u32) -> u16 {
    let (v, mut status) = ([a, k, c], 0u16);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fcom dword ptr [{v} + 8]",
        "fnstsw word ptr [{s}]", "fstp st(0)";
        v = v.as_ptr(), s = &raw mut status);
    status
}

/// `fld a; fmul k; fimul n; fistp`.
pub fn times_int_to_int(a: u32, k: u32, n: i32) -> i32 {
    let (v, mut out) = ([a, k, n as u32], 0i32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fimul dword ptr [{v} + 8]",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fmul b; fadd c; fst` (the first), `fchs; fst` (the second).
pub fn int_times_plus_and_negated(a: i32, b: u32, c: u32) -> (u32, u32) {
    let (v, mut kept, mut negated) = ([a as u32, b, c], 0u32, 0u32);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]",
        "fst dword ptr [{k}]", "fchs", "fstp dword ptr [{n}]";
        v = v.as_ptr(), k = &raw mut kept, n = &raw mut negated);
    (kept, negated)
}

/// `fild a; fmul b; fsub c; fsub d; fstp`.
pub fn int_times_less_less(a: i32, b: u32, c: u32, d: u32) -> u32 {
    let (v, mut out) = ([a as u32, b, c, d], 0u32);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fsub dword ptr [{v} + 8]",
        "fsub dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fabs; fld b; fabs; faddp; fmul k; fistp`.
pub fn abs_sum_times_to_int(a: u32, b: u32, k: u32) -> i32 {
    let (v, mut out) = ([a, b, k], 0i32);
    x87!("fld dword ptr [{v}]", "fabs", "fld dword ptr [{v} + 4]", "fabs", ".byte 0xDE, 0xC1",
        "fmul dword ptr [{v} + 8]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `sub_4CE05E`'s point: (-t0 k, (t1 - d) k, t2 k) turned by the matrix at `m` (its rows
/// 0x10 apart), placed by +0x30 .. and the place taken off again, all on the x87 stack as
/// the original does (`fxch`, register forms as bytes); then x, y, y less `lift`, z.
pub unsafe fn turned_place_less(t: [u32; 3], k: u32, d: u32, lift: u32, m: u32) -> [u32; 4] {
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
pub fn drifted(r: i32, k: u32, p: [u32; 3], ticks: i32, at: [u32; 3]) -> [u32; 3] {
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
pub fn one_less_inverse(a: u32, k: u32, one: u32) -> u32 {
    let (v, mut out) = ([a, k, one], 0u32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]",
        "fdivr dword ptr [{v} + 8]", "fsubr dword ptr [{v} + 8]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul k; fidiv n; fstp`.
pub fn times_over_int(a: u32, k: u32, n: i32) -> u32 {
    let (v, mut out) = ([a, k, n as u32], 0u32);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fidiv dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fmul b`, kept as 80 bits.
pub fn mul_80(a: u32, b: u32) -> [u8; 10] {
    let (v, mut out) = ([a, b], [0u8; 10]);
    x87!("fld dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fstp tbyte ptr [{o}]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    out
}

/// The point (st(0) p0, st(1) p1, st(2) p2) turned and placed by the matrix at `m` on the
/// x87 stack as `sub_4CE283` does (`fxch`, register forms as bytes); then x, y, y less
/// `lift`, z.
pub unsafe fn turned_place(p: [u32; 3], lift: u32, m: u32) -> [u32; 4] {
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
pub fn int_times_int_to_int(a: i32, b: i32) -> i32 {
    let (v, mut out) = ([a as u32, b as u32], 0i32);
    x87!("fild dword ptr [{v}]", "fimul dword ptr [{v} + 4]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld a; fadd b; fadd c; fadd d; fmul k; fstp`.
pub fn sum4_times(a: u32, b: u32, c: u32, d: u32, k: u32) -> u32 {
    let (v, mut out) = ([a, b, c, d, k], 0u32);
    x87!("fld dword ptr [{v}]", "fadd dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]",
        "fadd dword ptr [{v} + 12]", "fmul dword ptr [{v} + 16]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// The length of (a, b, c) (`fmul st, st` each, `faddp` twice, `fsqrt`) `fist` (the
/// first); then `fmul k1; fadd k2; fdivr one; fsubr one; fmul k3; fistp` (the second).
pub fn length_and_rate(p: [u32; 3], k1: u32, k2: u32, one: u32, k3: u32) -> (i32, i32) {
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
pub fn normalised3(p: [u32; 3], one: u32) -> [u32; 3] {
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

/// `sub_4C9AF6`'s third side: the cross product of (p0, p1, p2) and the vector at `w`
/// (unrounded) turned by the transposed matrix at `m` (rows 0x10 apart), as the original
/// does it on the x87 stack (`fxch`, register forms as bytes).
pub unsafe fn crossed_turned(p: [u32; 3], w: u32, m: u32) -> [u32; 3] {
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

/// The vector at `p` made of length 1 as `sub_4C1294` does it: the length (`fmul st, st`
/// each, `faddp` twice, `fsqrt`) `fst` (the second), `fld1; fdivrp`, each by it, stored z
/// first, the factor dropped (`ffree st; fincstp`).
pub fn unit_and_length(p: [u32; 3]) -> ([u32; 3], u32) {
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
pub fn negated_less_plus_plus(a: u32, b: u32, c: u32, d: u32) -> u32 {
    let (v, mut out) = ([a, b, c, d], 0u32);
    x87!("fld dword ptr [{v}]", "fchs", "fsub dword ptr [{v} + 4]", "fadd dword ptr [{v} + 8]",
        "fadd dword ptr [{v} + 12]", "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld t; fsin; fld t; fcos; fstp` (the cosine), `fstp` (the sine).
pub fn sine_and_cosine(t: u32) -> (u32, u32) {
    let (mut s, mut c) = (0u32, 0u32);
    x87!("fld dword ptr [{t}]", "fsin", "fld dword ptr [{t}]", "fcos",
        "fstp dword ptr [{c}]", "fstp dword ptr [{s}]";
        t = &raw const t, c = &raw mut c, s = &raw mut s);
    (s, c)
}

/// `sub_4C1194`'s turn of (y, z) by the stored sine and cosine: z c - y s and z s + y c
/// (`fsubp`, `faddp` on the stack), returned as (new y, new z).
pub fn turn_yz(y: u32, z: u32, s: u32, c: u32) -> (u32, u32) {
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
pub fn int_times_plus_int_to_int(a: i32, k: u32, b: i32) -> i32 {
    let (v, mut out) = ([a as u32, k, b as u32], 0i32);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fiadd dword ptr [{v} + 8]",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fmul ka; fild b; fmul kb; fild c; fmul kc; faddp; faddp`, kept as 80 bits:
/// a ka + (b kb + c kc).
pub fn blend3_80(a: i32, b: i32, c: i32, k: [u32; 3]) -> [u8; 10] {
    let (v, mut out) = ([a as u32, k[0], b as u32, k[1], c as u32, k[2]], [0u8; 10]);
    x87!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fild dword ptr [{v} + 8]",
        "fmul dword ptr [{v} + 12]", "fild dword ptr [{v} + 16]", "fmul dword ptr [{v} + 20]",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fstp tbyte ptr [{o}]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    out
}

/// `fld a; fild v; fdiv kd; fmul km; faddp; fst` (the first), `fsin; fmul ks; fstp` (the
/// second).
pub fn angle_and_wave(a: u32, v: i32, kd: u32, km: u32, ks: u32) -> (u32, u32) {
    let (x, mut angle, mut wave) = ([a, v as u32, kd, km, ks], 0u32, 0u32);
    x87!("fld dword ptr [{x}]", "fild dword ptr [{x} + 4]", "fdiv dword ptr [{x} + 8]",
        "fmul dword ptr [{x} + 12]", ".byte 0xDE, 0xC1", "fst dword ptr [{a}]", "fsin",
        "fmul dword ptr [{x} + 16]", "fstp dword ptr [{w}]";
        x = x.as_ptr(), a = &raw mut angle, w = &raw mut wave);
    (angle, wave)
}

/// `fild a; fisub b; fmul f; fiadd b; fistp`: b + (a - b) f, rounded.
pub fn int_lerp_to_int(a: i32, b: i32, f: u32) -> i32 {
    let (v, mut out) = ([a as u32, b as u32, f], 0i32);
    x87!("fild dword ptr [{v}]", "fisub dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fiadd dword ptr [{v} + 4]", "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fild a; fidiv b; fmul k; fstp`.
pub fn int_over_int_times(a: i32, b: i32, k: u32) -> u32 {
    let (v, mut out) = ([a as u32, b as u32, k], 0u32);
    x87!("fild dword ptr [{v}]", "fidiv dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fstp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `sub_4B8A0B`'s crossing: t = (m - fc) / (fs - fc) (`fld m; fsub fc; fld fs; fsub fc;
/// fdivp st(1), st`), then for each (s, c) pair (s - c) t + c (`fld s; fsub c; fmul st,
/// st(1)`, the last `fmulp`; `fadd c; fstp`), t kept unrounded.
pub fn clip_lerp3(m: u32, fc: u32, fs: u32, pairs: [[u32; 2]; 3]) -> [u32; 3] {
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
