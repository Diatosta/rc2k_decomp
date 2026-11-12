//! Small float steps of `make_masks`' tree (IDA): the 2 x 2 turn a line gives,
//! three rows scaled to set lengths, and two cosine-eased steps.

use crate::effects::mask_store::Masks;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::math::x87;
use crate::session::{PlainDword, PlainFloat};

/// `effects::set_line_turn` (0x4BF432) and, `back`, `effects::set_line_turn_back` (0x4BF466):
/// from the line (+0, +4 to +0xC, +0x10: here `line`'s dwords 0, 1 to 3, 4) dx into
/// +0x396 and +0x3A6, minus dy (dy when `back`) into +0x3A2, its negation into +0x39A.
pub fn set_line_turn(masks: &Masks, line: &[PlainDword; 6], back: bool) {
    let dx = x87::sub(line[3].get(), line[0].get());
    let dy = if back {
        x87::sub(line[4].get(), line[1].get())
    } else {
        x87::negated(x87::sub(line[4].get(), line[1].get()))
    };
    masks.vars.turn[0].set(dx);
    masks.vars.turn[3].set(dy);
    masks.vars.turn[1].set(x87::negated(dy));
    masks.vars.turn[4].set(x87::loaded(dx));
}

/// `effects::scale_rows3` (0x4BF498): the three rows of three floats each made the
/// length the effects' row lengths say (`x87::scale_row`).
pub fn scale_rows3(rows: &[PlainFloat; 9], effects: &crate::effects::owner::Effects) {
    for (n, row) in rows.as_chunks::<3>().0.iter().enumerate() {
        let values = [row[0].get(), row[1].get(), row[2].get()];
        let length = effects.setup.mask_constants.row_lengths[n].get();
        for (cell, v) in row.iter().zip(x87::scale_row(values, length)) {
            cell.set(v);
        }
    }
}

/// `effects::smooth_steps` (0x4BE928): from a to b as floats, step n: t (b - a) times
/// a step factor (a float), then the first result a + t - t cos(pi (n plus 1) / the step count)
/// and the second the same with n over the second count.
pub fn smooth_steps(
    drops: &WindscreenDrops,
    n: u32,
    a: u32,
    b: u32,
    effects: &crate::effects::owner::Effects,
) -> (u32, u32) {
    drops.inputs.scratch.set(a);
    effects.setup.work.scratch.set(a);
    smooth_pair(n, a, b, effects)
}
/// `smooth_steps`' two results from step n, a and b, writing nothing.
pub fn smooth_pair(n: u32, a: u32, b: u32, effects: &crate::effects::owner::Effects) -> (u32, u32) {
    let setup = &effects.setup;
    {
        let t = x87::sub_mul(b, a, &setup.mask_constants.ease);
        let steps: [i32; 2] = setup
            .mask_constants
            .steps
            .each_ref()
            .map(|c| c.get() as i32);
        (
            x87::cosine_step(steps[0], n.wrapping_add(1) as i32, t, a).to_bits(),
            x87::cosine_step(steps[1], n as i32, t, a).to_bits(),
        )
    }
}

/// `effects::turn_pair` (0x4BF409): st(0) a, st(1) b turned by `g_marksLineTurn` (m0 .. m3):
/// st(0) a m3 + b m2, st(1) b m0 + a m1, unrounded (the original's `fld st(1)`, `fxch`s
/// and `faddp`s). Every register kept.
pub fn turn_pair(masks: &Masks, inputs: [[u8; 10]; 2]) -> [[u8; 10]; 2] {
    turn_pair_x87(&inputs, &masks.vars.turn)
}

/// `fld tbyte; fmul; fistp` for st(0) then st(1): their whole-number scalings.
fn scaled_ints(inputs: &[[u8; 10]; 2], k0: &PlainFloat, k1: &PlainFloat) -> (i32, i32) {
    scaled_ints_x87(inputs, k0, k1)
}

/// `fild v; fdiv k; fstp tbyte`.
fn int_over_80(v: i32, k: &PlainFloat) -> [u8; 10] {
    int_over_80_x87(v, k)
}

/// `effects::clamp_pair` (0x4BEF22): st(0) times `MaskConstants.pair_scales[0]` rounded (b), st(1) times
/// `MaskConstants.pair_scales[1]` (a); a held to 0 .. 0xFFFF and b to 0 .. 0x7FFF; then st(1) a over
/// `MaskConstants.pair_scales[2]` and st(0) b over `MaskConstants.pair_scales[3]`, unrounded.
pub fn clamp_pair(
    inputs: [[u8; 10]; 2],
    effects: &crate::effects::owner::Effects,
) -> [[u8; 10]; 2] {
    let scales = &effects.setup.mask_constants.pair_scales;
    let (b, a) = scaled_ints(&inputs, &scales[0], &scales[1]);
    let b = b.clamp(0, 0x7FFF);
    let a = a.clamp(0, 0xFFFF);
    [int_over_80(b, &scales[3]), int_over_80(a, &scales[2])]
}
// ---- the converted versions (`rc2k_core::effects::turns`) ----

use crate::effects::plain::ld_cell;
use crate::math::x87_float as bh;

/// `turn_pair`'s x87 block as plain Rust on the x87 model (16 instructions), same order and memory places
/// (`i` the two inputs, `m` the turn: the block reads its floats 0, 1, 3 and 4).
fn turn_pair_x87(i: &[[u8; 10]; 2], m: &[PlainFloat; 9]) -> [[u8; 10]; 2] {
    let t0 = bh::from_f80(i[1]);
    let t1 = bh::from_f80(i[0]);
    let t2 = t1 * ld_cell(&m[4]);
    let t3 = t0 * ld_cell(&m[3]);
    let t4 = t2 + t3;
    let t5 = t0 * ld_cell(&m[0]);
    let t6 = t1 * ld_cell(&m[1]);
    let t7 = t5 + t6;
    [bh::f80(t4), bh::f80(t7)]
}

/// `scaled_ints`'s x87 block as plain Rust on the x87 model (6 instructions), same order and memory places
/// (the two whole numbers it stores).
fn scaled_ints_x87(i: &[[u8; 10]; 2], k0: &PlainFloat, k1: &PlainFloat) -> (i32, i32) {
    let t0 = bh::from_f80(i[1]);
    let t1 = bh::from_f80(i[0]);
    let t2 = t1 * ld_cell(k0);
    let v0 = bh::to_int(t2);
    let t3 = t0 * ld_cell(k1);
    let v1 = bh::to_int(t3);
    (v0, v1)
}

/// `int_over_80`'s x87 block as plain Rust on the x87 model (3 instructions), same order and memory places.
fn int_over_80_x87(v: i32, k: &PlainFloat) -> [u8; 10] {
    let t0 = bh::int(v);
    let t1 = t0 / ld_cell(k);
    bh::f80(t1)
}

#[cfg(test)]
mod reference_vectors {
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::{Op, Spec, recorded};
    use crate::session::PlainFloat;

    /// The `N` bytes of the test buffer at `at`.
    fn bytes<const N: usize>(at: crate::effects::asm_test::Operand) -> [u8; N] {
        std::array::from_fn(|n| unsafe { (at + n).host().read() })
    }

    /// The test buffer's dword at `at` as a float cell.
    fn cell(at: crate::effects::asm_test::Operand) -> PlainFloat {
        PlainFloat::from_bits(u32::from_le_bytes(bytes::<4>(at)))
    }

    /// `data` into the test buffer at `at`.
    fn put(at: crate::effects::asm_test::Operand, data: &[u8]) {
        for (n, &b) in data.iter().enumerate() {
            unsafe { (at + n).host().write(b) };
        }
    }

    /// The two ten-byte inputs at the test buffer's `at` (0 and 0xA).
    fn pair(at: crate::effects::asm_test::Operand) -> [[u8; 10]; 2] {
        [bytes::<10>(at), bytes::<10>(at + 0xA)]
    }

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
        recorded("effects.turns.turn_pair_x87", &SPEC, |b| {
            // The turn's floats 0, 1, 3 and 4 (byte offsets 0, 4, 0xC, 0x10).
            let m: [PlainFloat; 9] = std::array::from_fn(|n| {
                if n < 5 {
                    cell(b[1] + 4 * n)
                } else {
                    cell(b[1])
                }
            });
            let out = super::turn_pair_x87(&pair(b[0]), &m);
            put(b[2], &out[0]);
            put(b[2] + 0xA, &out[1]);
        });
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
        recorded("effects.turns.scaled_ints_x87", &SPEC, |b| {
            let (v0, v1) = super::scaled_ints_x87(&pair(b[0]), &cell(b[1]), &cell(b[1] + 4));
            put(b[2], &v0.to_le_bytes());
            put(b[2] + 4, &v1.to_le_bytes());
        });
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
        recorded("effects.turns.int_over_80_x87", &SPEC, |b| {
            let v = i32::from_le_bytes(bytes::<4>(b[0]));
            put(b[2], &super::int_over_80_x87(v, &cell(b[1])));
        });
    }
}
