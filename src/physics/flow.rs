//! The steps the physics' long functions share once their flow is in Rust the game's variables by IDA address, the x87 compares as the originals
//! read them (`fld; fcomp` / `fcompp` and the status word's C0 / C3), the holds and
//! eases their patterns repeat.

use crate::effects::cell_x87::CellX87;
use crate::math::x87;
use crate::math::x87::compare_full;
use crate::math::x87_float::{compare, ld, st};
use crate::session::Float;

/// The bits of 0.0. The game keeps one such constant in `race.course.numbers.zero`
/// (`numbers.zero`, 0.0); nothing in the program writes that cell, so the
/// physics reads this constant instead of going through the race for it.
pub const ZERO_BITS: u32 = 0;

/// The game's dword in the cell `x`.
pub fn g(x: &impl crate::session::Cell32) -> u32 {
    x.bits()
}

/// Writes the game's dword into the cell `x`.
pub fn p(x: &impl crate::session::Cell32, v: u32) {
    x.set_bits(v);
}

/// `fld a; fcomp b`: C0, a below b or unordered.
pub fn c0(a: impl x87::Flt, b: impl x87::Flt) -> bool {
    compare_full(a.bits(), b.bits()) & 0x100 != 0
}

/// `fld a; fcomp b`: C3, a equal to b or unordered.
pub fn c3(a: impl x87::Flt, b: impl x87::Flt) -> bool {
    compare_full(a.bits(), b.bits()) & 0x4000 != 0
}

/// The float `a` is below `b`, or one of them is not a number (the x87's C0 after
/// comparing them); `!below_or_unordered(a, b)` is "`a` is at least `b`".
pub fn below_or_unordered(a: impl x87::Flt, b: impl x87::Flt) -> bool {
    c0(a, b)
}

/// The float `a` equals `b`, or one of them is not a number (the x87's C3 after comparing
/// them).
pub fn equal_or_unordered(a: impl x87::Flt, b: impl x87::Flt) -> bool {
    c3(a, b)
}

/// The size of the float `a` is below `b`, or one of them is not a number (`fabs` then
/// the comparison's C0).
pub fn abs_below_or_unordered(a: u32, b: u32) -> bool {
    crate::math::x87::abs_compare(a, b) & 0x100 != 0
}

/// `fld lim; fld v; fcompp; jnb skip; fld lim; fstp v`: v raised to lim when below it
/// (or unordered).
pub fn hold_low(v: &impl crate::session::Cell32, lim: &impl crate::session::Cell32) {
    hold_low_bits(v, g(lim));
}

/// `hold_low` against a limit given as a value (its bits).
pub fn hold_low_bits(v: &impl crate::session::Cell32, lim: u32) {
    if c0(g(v), lim) {
        p(v, x87::loaded(lim).to_bits());
    }
}

/// `fld lim; fld v; fcompp; jb skip; fld lim; fstp v`: v lowered to lim unless below it
/// (or unordered).
pub fn hold_high(v: &impl crate::session::Cell32, lim: &impl crate::session::Cell32) {
    if !c0(g(v), g(lim)) {
        p(v, x87::loaded(g(lim)).to_bits());
    }
}

/// `fld v; fchs; fstp v`.
pub fn negate(v: &impl crate::session::Cell32) {
    p(
        v,
        crate::math::x87::neg_rounded(crate::scene::matrix3::extended(g(v))).to_bits(),
    );
}

/// The value at `v` brought toward 0 by the step at `step`, never past it; 0 (either
/// sign) or unordered left alone (`fldz; fld v; fcompp`, C3 then C0, as the originals).
pub fn ease_zero(v: &impl crate::session::Cell32, step: &impl crate::session::Cell32) {
    let x = g(v);
    if c3(x, 0) {
        return;
    }
    if c0(x, 0) {
        let y = x87::add(x, g(step));
        p(v, y.to_bits());
        if !c0(y.to_bits(), 0) {
            p(v, 0);
        }
    } else {
        let y = x87::sub(x, g(step));
        p(v, y.to_bits());
        if c0(y.to_bits(), 0) {
            p(v, 0);
        }
    }
}

/// A clamped table lerp (the damage model's, e.g. 0x4AB61C .. 0x4AB680): the table at
/// `t` is (x0, y0, x1, y1); the value at `x` below x0 (or unordered) gives y0, below x1
/// gives y0 + (x - x0) / (x1 - x0) x (y1 - y0) (y0 when x1 - x0, unrounded, equals the
/// value at `zero`), else y1; into `out` (`fstp`).
pub fn table_lerp(
    x: &impl crate::session::Cell32,
    t: &[crate::session::Float; 4],
    zero: &impl crate::session::Cell32,
    out: &impl crate::session::Cell32,
) {
    if c0(g(x), t[0].bits()) {
        p(out, x87::loaded(&t[1]).to_bits());
        return;
    }
    if !c0(g(x), t[2].bits()) {
        p(out, x87::loaded(&t[3]).to_bits());
        return;
    }
    if x87_1(t, g(zero)) & 0x4000 != 0 {
        p(out, x87::loaded(&t[1]).to_bits());
        return;
    }
    p(out, x87_2(g(x), t));
}

// ---- the arithmetic that was x87 assembly ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `status [s + 0] = status`
fn x87_1(t: &[Float; 4], z: u32) -> u16 {
    let t0 = t[2].ld();
    let t1 = t[0].ld();
    let t2 = t0 - t1;
    let t3 = ld(z);
    compare(t2, t3)
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[o + 0] = ((([x + 0] - [t + 0]) / ([t + 8] - [t + 0])) * ([t + 0xC] - [t + 4])) + [t + 4]`
fn x87_2(x: u32, t: &[Float; 4]) -> u32 {
    let t0 = ld(x);
    let t1 = t[0].ld();
    let t2 = t0 - t1;
    let t3 = t[2].ld();
    let t4 = t[0].ld();
    let t5 = t3 - t4;
    let t6 = t2 / t5;
    let t7 = t[3].ld();
    let t8 = t[1].ld();
    let t9 = t7 - t8;
    let t10 = t6 * t9;
    let t11 = t[1].ld();
    let t12 = t10 + t11;
    st(t12)
}

/// The first three cells of `from` copied as integers (three dword moves) into the first three of `to`: `copy3` for typed cells (a `Float` goes by
/// its bits).
pub fn copy_cells<A: crate::session::Cell32, B: crate::session::Cell32>(to: &[A], from: &[B]) {
    for k in 0..3 {
        to[k].set_bits(from[k].bits());
    }
}

/// The first three cells of `to` set to the bits `v` (`fill3` for typed cells).
pub fn fill_cells<A: crate::session::Cell32>(to: &[A], v: u32) {
    for cell in &to[..3] {
        cell.set_bits(v);
    }
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};
    use crate::scene::x87_mem::{Base, HostCell, view};
    use crate::session::Cell32;

    // The original's instructions, one function per run.
    reference!(ref_0(t, z, s) "fld dword ptr [{t} + 8]", "fsub dword ptr [{t}]", "fcomp dword ptr [{z}]", "fnstsw word ptr [{s}]");
    reference!(ref_1(x, t, o) "fld dword ptr [{x}]", "fsub dword ptr [{t}]", "fld dword ptr [{t} + 8]", "fsub dword ptr [{t}]", ".byte 0xDE, 0xF9", "fld dword ptr [{t} + 0xC]", "fsub dword ptr [{t} + 4]", ".byte 0xDE, 0xC9", "fadd dword ptr [{t} + 4]", "fstp dword ptr [{o}]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "flow line 127",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F), (8, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::H)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1], a[2]) },
                plain: |a| a[2].set_word(0, x87_1(view(a[0], 0), HostCell(a[1]).bits())),
            },
            Block {
                name: "flow line 138",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::F), (4, In::F), (8, In::F), (0xC, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1], a[2]) },
                plain: |a| HostCell(a[2]).set_bits(x87_2(HostCell(a[0]).bits(), view(a[1], 0))),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
