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
