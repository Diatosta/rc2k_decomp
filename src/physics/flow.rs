//! The steps the physics' long functions share once their flow is in Rust (stream B,
//! 2026-10-18): the game's variables by IDA address, the x87 compares as the originals
//! read them (`fld; fcomp` / `fcompp` and the status word's C0 / C3), the holds and
//! eases their patterns repeat.

use crate::math::x87;
use crate::math::x87_float::compare;
use crate::math::x87_run::compare_full;
use crate::scene::x87_mem::{fld, fnstsw, fstp};

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
pub fn hold_low(v: impl crate::store::At, lim: impl crate::store::At) {
    if c0(g(v), g(lim)) {
        p(v, x87::loaded(g(lim)));
    }
}

/// `fld lim; fld v; fcompp; jb skip; fld lim; fstp v`: v lowered to lim unless below it
/// (or unordered).
pub fn hold_high(v: impl crate::store::At, lim: impl crate::store::At) {
    if !c0(g(v), g(lim)) {
        p(v, x87::loaded(g(lim)));
    }
}

/// `fld v; fchs; fstp v`.
pub fn negate(v: impl crate::store::At) {
    p(
        v,
        crate::math::x87_run::neg_rounded(crate::scene::matrix3::extended(g(v))),
    );
}

/// The value at `v` brought toward 0 by the step at `step`, never past it; 0 (either
/// sign) or unordered left alone (`fldz; fld v; fcompp`, C3 then C0, as the originals).
pub fn ease_zero(v: impl crate::store::At, step: impl crate::store::At) {
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

/// A clamped table lerp (the damage model's, e.g. 0x4AB61C .. 0x4AB680): the table at
/// `t` is (x0, y0, x1, y1); the value at `x` below x0 (or unordered) gives y0, below x1
/// gives y0 + (x - x0) / (x1 - x0) x (y1 - y0) (y0 when x1 - x0, unrounded, equals the
/// value at `zero`), else y1; into `out` (`fstp`).
pub fn table_lerp(
    x: impl crate::store::At,
    t: &[crate::session::Float; 4],
    zero: impl crate::store::At,
    out: impl crate::store::At,
) {
    if c0(g(x), t[0].bits()) {
        p(out, x87::loaded(t[1].bits()));
        return;
    }
    if !c0(g(x), t[2].bits()) {
        p(out, x87::loaded(t[3].bits()));
        return;
    }
    let mut status = 0u16;
    x87_1(
        (crate::pointers::game_address(t.as_ptr())) as usize,
        a(zero) as usize,
        crate::pointers::game_address(&raw mut status) as usize,
    );
    if status & 0x4000 != 0 {
        p(out, x87::loaded(t[1].bits()));
        return;
    }
    x87_2(
        a(x) as usize,
        (crate::pointers::game_address(t.as_ptr())) as usize,
        a(out) as usize,
    );
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `status [s + 0] = status`
fn x87_1(t: usize, z: usize, s: usize) {
    let t0 = fld(t, 8);
    let t1 = fld(t, 0);
    let t2 = t0 - t1;
    let t3 = fld(z, 0);
    let t4 = compare(t2, t3);
    fnstsw(s, 0, t4);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[o + 0] = ((([x + 0] - [t + 0]) / ([t + 8] - [t + 0])) * ([t + 0xC] - [t + 4])) + [t + 4]`
fn x87_2(x: usize, t: usize, o: usize) {
    let t0 = fld(x, 0);
    let t1 = fld(t, 0);
    let t2 = t0 - t1;
    let t3 = fld(t, 8);
    let t4 = fld(t, 0);
    let t5 = t3 - t4;
    let t6 = t2 / t5;
    let t7 = fld(t, 0xC);
    let t8 = fld(t, 4);
    let t9 = t7 - t8;
    let t10 = t6 * t9;
    let t11 = fld(t, 4);
    let t12 = t10 + t11;
    fstp(o, 0, t12);
}

/// The first three cells of `from` copied as integers (`mov eax, [from]; mov [to], eax`,
/// three times) into the first three of `to`: `copy3` for typed cells (a `Float` goes by
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

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

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
                plain: |a| x87_1(a[0], a[1], a[2]),
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
                plain: |a| x87_2(a[0], a[1], a[2]),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
