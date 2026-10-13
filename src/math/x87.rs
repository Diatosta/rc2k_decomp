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
