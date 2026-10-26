//! Small float steps of `make_masks`' tree (IDA, 2026-10-17): the 2 x 2 turn a line gives,
//! three rows scaled to set lengths, and two cosine-eased steps.

use crate::errors::{BadInput, Refused};
use crate::marks::mask_constants::MASK_CONSTANTS;
use crate::marks::mask_vars::MASKS;
use crate::marks::work_store::MARKS_WORK;
use crate::math::x87;
use crate::pointers::GamePtr;

fn get(at: u32) -> Result<u32, BadInput> {
    GamePtr::new(at).read("a game field")
}

/// `marks::set_line_turn` (0x4BF432) and, `back`, `marks::set_line_turn_back` (0x4BF466):
/// from the line at esi (+0, +4 to +0xC, +0x10) dx into +0x396 and +0x3A6, minus dy (dy
/// when `back`) into +0x3A2, its negation into +0x39A. Every register kept.
pub fn set_line_turn(line: u32, back: bool) -> Result<(), Refused> {
    {
        let dx = x87::sub(get(line + 0xC)?, get(line)?);
        let dy = if back {
            x87::sub(get(line + 0x10)?, get(line + 4)?)
        } else {
            x87::negated_sub(get(line + 0x10)?, get(line + 4)?)
        };
        MASKS.turn[0].set_bits(dx);
        MASKS.turn[3].set_bits(dy);
        MASKS.turn[1].set_bits(x87::negated(dy));
        MASKS.turn[4].set_bits(x87::loaded(dx));
    }
    Ok(())
}

/// `marks::scale_rows3` (0x4BF498): the three rows of three floats at esi each made the
/// length `flt_89CA24` .. `flt_89CA2C` says (`x87::scale_row`). Every register kept.
pub fn scale_rows3(rows: u32) -> Result<(), BadInput> {
    crate::race::trap::span(rows, 36, "a mark's three rows")?;
    // SAFETY: the nine floats lie in one live region (checked above).
    for n in 0..3 {
        x87::scale_row(
            crate::pointers::host(rows + 12 * n) as *mut u32,
            MASK_CONSTANTS.row_lengths[n as usize].bits(),
        );
    }
    Ok(())
}

/// `marks::smooth_steps` (0x4BE928): from a to b as floats, step n: t (b - a) times
/// `flt_89C9C0` (a float), then the first result (the original's ecx) a + t - t
/// cos(pi (n plus 1) / `dword_89C9C4`) and the second (its ebx) the same with n over `dword_89C9C8`.
/// `dword_764F40` and `dword_71C880` left holding a.
pub fn smooth_steps(n: u32, a: u32, b: u32) -> (u32, u32) {
    crate::marks::shape_points::SHAPE_INPUTS.scratch.set(a);
    MARKS_WORK.scratch.set(a);
    smooth_pair(n, a, b)
}
/// `smooth_steps`' two results (ecx, ebx) from step n, a and b, writing nothing.
pub fn smooth_pair(n: u32, a: u32, b: u32) -> (u32, u32) {
    {
        let t = x87::sub_mul(b, a, MASK_CONSTANTS.ease.bits());
        let steps: [i32; 2] = MASK_CONSTANTS.steps.each_ref().map(|c| c.get() as i32);
        (
            x87::cosine_step(steps[0], n.wrapping_add(1) as i32, t, a),
            x87::cosine_step(steps[1], n as i32, t, a),
        )
    }
}

/// `marks::turn_pair` (0x4BF409): st(0) a, st(1) b turned by `g_marksLineTurn` (m0 .. m3):
/// st(0) a m3 + b m2, st(1) b m0 + a m1, unrounded (the original's `fld st(1)`, `fxch`s
/// and `faddp`s). Every register kept.
pub fn turn_pair(inputs: [[u8; 10]; 2]) -> [[u8; 10]; 2] {
    let mut out = [[0u8; 10]; 2];
    let m = crate::pointers::game_address(MASKS.turn.as_ptr());
    turn_pair_x87(
        crate::pointers::host_number(inputs.as_ptr()),
        m as usize,
        crate::pointers::host_number(out.as_mut_ptr()),
    );
    out
}

/// `fld tbyte; fmul; fistp` for st(0) then st(1): their whole-number scalings.
fn scaled_ints(inputs: &[[u8; 10]; 2], k: *const u32) -> (i32, i32) {
    let mut v = [0i32; 2];
    scaled_ints_x87(
        crate::pointers::host_number(inputs.as_ptr()),
        crate::pointers::host_number(k),
        crate::pointers::host_number(v.as_mut_ptr()),
    );
    (v[0], v[1])
}

/// `fild v; fdiv k; fstp tbyte`.
fn int_over_80(v: i32, k: *const u32) -> [u8; 10] {
    let mut out = [0u8; 10];
    int_over_80_x87(
        crate::pointers::host_number(&raw const v),
        crate::pointers::host_number(k),
        crate::pointers::host_number(out.as_mut_ptr()),
    );
    out
}

/// `marks::clamp_pair` (0x4BEF22): st(0) times `flt_89C9FC` rounded (b), st(1) times
/// `flt_89CA00` (a); a held to 0 .. 0xFFFF and b to 0 .. 0x7FFF; then st(1) a over
/// `flt_89CA04` and st(0) b over `flt_89CA08`, unrounded.
pub fn clamp_pair(inputs: [[u8; 10]; 2]) -> [[u8; 10]; 2] {
    let k = MASK_CONSTANTS.pair_scales.as_ptr() as *const u32;
    let (b, a) = scaled_ints(&inputs, k);
    let b = b.clamp(0, 0x7FFF);
    let a = a.clamp(0, 0xFFFF);
    unsafe { [int_over_80(b, k.add(3)), int_over_80(a, k.add(2))] }
}
// ---- the converted versions (`rc2k_core::marks::turns`) ----

use crate::marks::plain::{ld, ld80, ldi, st80, sti};
use crate::math::x87_float as bh;

/// `turn_pair`'s x87 block as plain Rust on the x87 model (16 instructions), same order and memory places.
fn turn_pair_x87(i: usize, m: usize, o: usize) {
    let t0 = bh::from_f80(ld80(i, 0xA));
    let t1 = bh::from_f80(ld80(i, 0));
    let t2 = t1 * ld(m, 0x10);
    let t3 = t0 * ld(m, 0xC);
    let t4 = t2 + t3;
    let t5 = t0 * ld(m, 0);
    let t6 = t1 * ld(m, 0x4);
    let t7 = t5 + t6;
    st80(o, 0, t4);
    st80(o, 0xA, t7);
}

/// `scaled_ints`'s x87 block as plain Rust on the x87 model (6 instructions), same order and memory places.
fn scaled_ints_x87(i: usize, k: usize, v: usize) {
    let t0 = bh::from_f80(ld80(i, 0xA));
    let t1 = bh::from_f80(ld80(i, 0));
    let t2 = t1 * ld(k, 0);
    sti(v, 0, t2);
    let t3 = t0 * ld(k, 0x4);
    sti(v, 0x4, t3);
}

/// `int_over_80`'s x87 block as plain Rust on the x87 model (3 instructions), same order and memory places.
fn int_over_80_x87(v: usize, k: usize, o: usize) {
    let t0 = ldi(v, 0);
    let t1 = t0 / ld(k, 0);
    st80(o, 0, t1);
}

#[cfg(test)]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn turn_pair_x87(i: usize, m: usize, o: usize) {
        unsafe {
            core::arch::asm!(
                    "fld tbyte ptr [{i} + 10]",
            "fld tbyte ptr [{i}]",
            ".byte 0xD9, 0xC1",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{m} + 0x10]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{m} + 0xC]",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{m}]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{m} + 4]",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xC9",
            "fstp tbyte ptr [{o}]",
            "fstp tbyte ptr [{o} + 10]",
            i = in(reg) i,
            m = in(reg) m,
            o = in(reg) o,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) fn scaled_ints_x87(i: usize, k: usize, v: usize) {
        unsafe {
            core::arch::asm!(
                    "fld tbyte ptr [{i} + 10]",
            "fld tbyte ptr [{i}]",
            "fmul dword ptr [{k}]",
            "fistp dword ptr [{v}]",
            "fmul dword ptr [{k} + 4]",
            "fistp dword ptr [{v} + 4]",
            i = in(reg) i,
            k = in(reg) k,
            v = in(reg) v,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) fn int_over_80_x87(v: usize, k: usize, o: usize) {
        unsafe {
            core::arch::asm!(
                    "fild dword ptr [{v}]",
            "fdiv dword ptr [{k}]",
            "fstp tbyte ptr [{o}]",
            v = in(reg) v,
            k = in(reg) k,
            o = in(reg) o,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }
}

#[cfg(test)]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::marks::asm_test::Kind::*;
    use crate::marks::asm_test::{Op, Spec, differential};

    #[test]
    fn turn_pair_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x0,
                    hi: 0x14,
                    reads: &[(0x0, F80), (0xA, F80)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x14,
                    reads: &[(0x0, F32), (0x4, F32), (0xC, F32), (0x10, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x14,
                    reads: &[],
                    writes: &[(0x0, F80), (0xA, F80)],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::turn_pair_x87(b[0], b[1], b[2]),
            |b| super::turn_pair_x87(b[0], b[1], b[2]),
        );
    }

    #[test]
    fn scaled_ints_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x0,
                    hi: 0x14,
                    reads: &[(0x0, F80), (0xA, F80)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x8,
                    reads: &[(0x0, F32), (0x4, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x8,
                    reads: &[],
                    writes: &[(0x0, I32), (0x4, I32)],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::scaled_ints_x87(b[0], b[1], b[2]),
            |b| super::scaled_ints_x87(b[0], b[1], b[2]),
        );
    }

    #[test]
    fn int_over_80_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x0,
                    hi: 0x4,
                    reads: &[(0x0, I32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x4,
                    reads: &[(0x0, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0xA,
                    reads: &[],
                    writes: &[(0x0, F80)],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::int_over_80_x87(b[0], b[1], b[2]),
            |b| super::int_over_80_x87(b[0], b[1], b[2]),
        );
    }
}
