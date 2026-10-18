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
