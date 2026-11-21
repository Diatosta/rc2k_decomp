//! Small float steps of `make_masks`' tree: the 2 x 2 turn a line gives, three rows scaled to set
//! lengths, and two cosine-eased steps.

use crate::effects::mask_store::Masks;
use crate::math::x87;

/// `effects::set_line_turn` (and with `back`, the turn back):
/// from the line (`line`'s dwords 0, 1 to 3, 4) dx into the turn's first and fifth cells, minus
/// dy (dy when `back`) into its fourth, its negation into its second.
pub fn set_line_turn(masks: &Masks, line: [u32; 6], back: bool) {
    let dx = x87::sub(line[3], line[0]);
    let dy = if back {
        x87::sub(line[4], line[1])
    } else {
        x87::negated(x87::sub(line[4], line[1]))
    };
    let mut vars = masks.vars.borrow_mut();
    vars.turn[0] = dx;
    vars.turn[3] = dy;
    vars.turn[1] = x87::negated(dy);
    vars.turn[4] = x87::loaded(dx);
}

/// `effects::scale_rows3`: the three rows of three floats each made the
/// length the effects' row lengths say (`x87::scale_row`).
pub fn scale_rows3(rows: &mut [f32; 9], effects: &crate::effects::owner::Effects) {
    for (n, row) in rows.as_chunks_mut::<3>().0.iter_mut().enumerate() {
        let values = [row[0], row[1], row[2]];
        let length = effects.setup.mask_constants.borrow().row_lengths[n];
        for (cell, v) in row.iter_mut().zip(x87::scale_row(values, length)) {
            *cell = v;
        }
    }
}

/// `effects::smooth_steps`: from a to b as floats, step n: t (b - a) times
/// a step factor (a float), then the first result a + t - t cos(pi (n plus 1) / the step count)
/// and the second the same with n over the second count.
pub fn smooth_steps(
    n: u32,
    a: f32,
    b: u32,
    effects: &crate::effects::owner::Effects,
) -> (u32, u32) {
    effects.setup.work.borrow_mut().scratch = a.to_bits();
    smooth_pair(n, a, b, effects)
}
/// `smooth_steps`' two results from step n, a and b, writing nothing.
pub fn smooth_pair(n: u32, a: f32, b: u32, effects: &crate::effects::owner::Effects) -> (u32, u32) {
    let setup = &effects.setup;
    {
        let t = x87::sub_mul(b, a, setup.mask_constants.borrow().ease);
        let steps: [i32; 2] = setup.mask_constants.borrow().steps.map(|c| c as i32);
        (
            x87::cosine_step(steps[0], n.wrapping_add(1) as i32, t, a).to_bits(),
            x87::cosine_step(steps[1], n as i32, t, a).to_bits(),
        )
    }
}

/// `effects::turn_pair`: the pair (a, b) turned by the line turn (m0 .. m3): a m3 + b m2,
/// b m0 + a m1, unrounded.
pub fn turn_pair(masks: &Masks, inputs: [[u8; 10]; 2]) -> [[u8; 10]; 2] {
    let turn = masks.vars.borrow().turn;
    turn_pair_x87(&inputs, &turn)
}

/// `turn_pair` with a copy of the turn's nine floats.
pub(super) fn turn_pair_with(turn: &[f32; 9], inputs: [[u8; 10]; 2]) -> [[u8; 10]; 2] {
    turn_pair_x87(&inputs, turn)
}

/// The whole-number scalings of the first and second values.
fn scaled_ints(inputs: &[[u8; 10]; 2], k0: f32, k1: f32) -> (i32, i32) {
    scaled_ints_x87(inputs, k0, k1)
}

/// A whole number over `k`, as an 80-bit value.
fn int_over_80(v: i32, k: f32) -> [u8; 10] {
    int_over_80_x87(v, k)
}

/// `effects::clamp_pair`: the first value times `MaskConstants.pair_scales[0]` rounded (b),
/// the second times `MaskConstants.pair_scales[1]` (a); a held to 0 .. 0xFFFF and b to
/// 0 .. 0x7FFF; then a over `MaskConstants.pair_scales[2]` and b over
/// `MaskConstants.pair_scales[3]`, unrounded.
pub fn clamp_pair(
    inputs: [[u8; 10]; 2],
    effects: &crate::effects::owner::Effects,
) -> [[u8; 10]; 2] {
    let scales = effects.setup.mask_constants.borrow().pair_scales;
    let (b, a) = scaled_ints(&inputs, scales[0], scales[1]);
    let b = b.clamp(0, 0x7FFF);
    let a = a.clamp(0, 0xFFFF);
    [int_over_80(b, scales[3]), int_over_80(a, scales[2])]
}
// ---- the plain versions (`rc2k_core::effects::turns`) ----

use crate::math::x87_float as bh;

/// `turn_pair`'s arithmetic as plain Rust on the x87 model, in the same order
/// (`i` the two inputs, `m` the turn: the block reads its floats 0, 1, 3 and 4).
fn turn_pair_x87(i: &[[u8; 10]; 2], m: &[f32; 9]) -> [[u8; 10]; 2] {
    let t0 = bh::from_f80(i[1]);
    let t1 = bh::from_f80(i[0]);
    let t2 = t1 * bh::ld(m[4]);
    let t3 = t0 * bh::ld(m[3]);
    let t4 = t2 + t3;
    let t5 = t0 * bh::ld(m[0]);
    let t6 = t1 * bh::ld(m[1]);
    let t7 = t5 + t6;
    [bh::f80(t4), bh::f80(t7)]
}

/// `scaled_ints`'s arithmetic as plain Rust on the x87 model, in the same order
/// (the two whole numbers it stores).
fn scaled_ints_x87(
    i: &[[u8; 10]; 2],
    k0: impl crate::math::x87::Flt,
    k1: impl crate::math::x87::Flt,
) -> (i32, i32) {
    let t0 = bh::from_f80(i[1]);
    let t1 = bh::from_f80(i[0]);
    let t2 = t1 * bh::ld(k0.value());
    let v0 = bh::to_int(t2);
    let t3 = t0 * bh::ld(k1.value());
    let v1 = bh::to_int(t3);
    (v0, v1)
}

/// `int_over_80`'s arithmetic as plain Rust on the x87 model, in the same order.
fn int_over_80_x87(v: i32, k: impl crate::math::x87::Flt) -> [u8; 10] {
    let t0 = bh::int(v);
    let t1 = t0 / bh::ld(k.value());
    bh::f80(t1)
}

#[cfg(test)]
mod tests;
