//! The steps the physics' long functions share once their flow is in Rust (stream B,
//! 2026-10-18): the game's variables by IDA address, the x87 compares as the originals
//! read them (`fld; fcomp` / `fcompp` and the status word's C0 / C3), the holds and
//! eases their patterns repeat, and `x87_run!`, a block of the original's x87
//! instructions over the game's variables (generated from IDA, no jumps, no integer code,
//! no calls).

use crate::math::x87;
use crate::math::x87_run::compare_full;

/// The game's dword at IDA address `x` (or a place of a moved block).
pub fn g(x: impl crate::store::At) -> u32 {
    unsafe { crate::store::at_read(x) }
}

/// Writes the game's dword at IDA address `x`.
pub fn p(x: impl crate::store::At, v: u32) {
    unsafe { crate::store::at_write(x, v) }
}

/// The game's address of IDA address `x`.
pub fn a(x: impl crate::store::At) -> u32 {
    x.here() as u32
}

/// `fld a; fcomp b`: C0, a below b or unordered.
pub fn c0(a: u32, b: u32) -> bool {
    compare_full(a, b) & 0x100 != 0
}

/// `fld a; fcomp b`: C3, a equal to b or unordered.
pub fn c3(a: u32, b: u32) -> bool {
    compare_full(a, b) & 0x4000 != 0
}

/// `fld lim; fld v; fcompp; jnb skip; fld lim; fstp v`: v raised to lim when below it
/// (or unordered).
pub fn hold_low(v: usize, lim: usize) {
    if c0(g(v), g(lim)) {
        p(v, x87::loaded(g(lim)));
    }
}

/// `fld lim; fld v; fcompp; jb skip; fld lim; fstp v`: v lowered to lim unless below it
/// (or unordered).
pub fn hold_high(v: usize, lim: usize) {
    if !c0(g(v), g(lim)) {
        p(v, x87::loaded(g(lim)));
    }
}

/// `fld v; fchs; fstp v`.
pub fn negate(v: usize) {
    p(
        v,
        crate::math::x87_run::neg_rounded(crate::scene::matrix3::extended(g(v))),
    );
}

/// The value at `v` brought toward 0 by the step at `step`, never past it; 0 (either
/// sign) or unordered left alone (`fldz; fld v; fcompp`, C3 then C0, as the originals).
pub fn ease_zero(v: usize, step: usize) {
    let x = g(v);
    if c3(x, 0) {
        return;
    }
    if c0(x, 0) {
        let y = x87::add(x, g(step));
        p(v, y);
        if !c0(y, 0) {
            p(v, 0);
        }
    } else {
        let y = x87::sub(x, g(step));
        p(v, y);
        if c0(y, 0) {
            p(v, 0);
        }
    }
}

/// A run of the original's x87 instructions (generated from IDA) over the game's
/// variables: `b` the game's base, other operands registers as named.
macro_rules! x87_run {
    ($($line:literal),+ ; $($name:ident = $value:expr),+) => {
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
pub(crate) use x87_run;

/// Three dwords copied as integers (`mov eax, [from]; mov [to], eax`, three times).
pub fn copy3(to: usize, from: usize) {
    for k in 0..3 {
        p(to + 4 * k, g(from + 4 * k));
    }
}

/// Three dwords set to `v`.
pub fn fill3(to: usize, v: u32) {
    for k in 0..3 {
        p(to + 4 * k, v);
    }
}

/// A clamped table lerp (the damage model's, e.g. 0x4AB61C .. 0x4AB680): the table at
/// `t` is (x0, y0, x1, y1); the value at `x` below x0 (or unordered) gives y0, below x1
/// gives y0 + (x - x0) / (x1 - x0) x (y1 - y0) (y0 when x1 - x0, unrounded, equals the
/// value at `zero`), else y1; into `out` (`fstp`).
pub fn table_lerp(x: usize, t: usize, zero: usize, out: usize) {
    if c0(g(x), g(t)) {
        p(out, x87::loaded(g(t + 4)));
        return;
    }
    if !c0(g(x), g(t + 8)) {
        p(out, x87::loaded(g(t + 0xC)));
        return;
    }
    let mut status = 0u16;
    x87_run!(
        "fld dword ptr [{t} + 8]",
        "fsub dword ptr [{t}]",
        "fcomp dword ptr [{z}]",
        "fnstsw word ptr [{s}]";
        t = a(t), z = a(zero), s = &raw mut status
    );
    if status & 0x4000 != 0 {
        p(out, x87::loaded(g(t + 4)));
        return;
    }
    x87_run!(
        "fld dword ptr [{x}]",
        "fsub dword ptr [{t}]",
        "fld dword ptr [{t} + 8]",
        "fsub dword ptr [{t}]",
        ".byte 0xDE, 0xF9", // fdivp st(1), st
        "fld dword ptr [{t} + 0xC]",
        "fsub dword ptr [{t} + 4]",
        ".byte 0xDE, 0xC9", // fmulp st(1), st
        "fadd dword ptr [{t} + 4]",
        "fstp dword ptr [{o}]";
        x = a(x), t = a(t), o = a(out)
    );
}
