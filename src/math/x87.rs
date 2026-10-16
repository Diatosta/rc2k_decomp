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
