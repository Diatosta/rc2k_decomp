//! `effects::scale_axes`: the two axes at `WindscreenFlakeTables.axes[0][0]` and
//! `WindscreenFlakeTables.axes[1][0]` scaled: by `WindscreenFlakeTables.radii[0]` into
//! `WindscreenFlakeTables.scaled_by_radius[0][0]` / `WindscreenFlakeTables.scaled_by_radius[1][0]`,
//! by `WindscreenFlakeTables.lengths[0]` / `WindscreenFlakeTables.lengths[1]` and the per-part
//! factors `WindscreenFlakeState.axis_factors[0]` .. into `WindscreenFlakeTables.scaled_axes[0][0]`
//! / `WindscreenFlakeTables.scaled_axes[1][0]`, and the first two times
//! `WindscreenFlakeState.halves[0]` .. into `WindscreenFlakeTables.half_axes[0][0]` /
//! `WindscreenFlakeTables.half_axes[1][0]`.
//!
//! The arithmetic is plain Rust on the x87 model (`math::x87_float`), over the game's variables.

pub fn scale_axes(fl: &WindscreenFlakes) {
    scale_axes_x87(&mut fl.tables.borrow_mut(), &fl.state.borrow());
}

use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::effects::windscreen_flakes_store::{WindscreenFlakeState, WindscreenFlakeTables};
use crate::math::x87_float as bh;

/// `scale_axes`'s arithmetic as plain Rust on the x87 model, in the same order.
fn scale_axes_x87(sq: &mut WindscreenFlakeTables, ss: &WindscreenFlakeState) {
    {
        let t0 = bh::ld(sq.axes[0][0]);
        let t1 = t0 * bh::ld(sq.radii[0]);
        sq.scaled_by_radius[0][0] = f32::from_bits(bh::st(t1));
        let t2 = bh::ld(sq.axes[0][1]);
        let t3 = t2 * bh::ld(sq.radii[0]);
        sq.scaled_by_radius[0][1] = f32::from_bits(bh::st(t3));
        let t4 = bh::ld(sq.axes[0][2]);
        let t5 = t4 * bh::ld(sq.radii[0]);
        sq.scaled_by_radius[0][2] = f32::from_bits(bh::st(t5));
        let t6 = bh::ld(sq.axes[1][0]);
        let t7 = t6 * bh::ld(sq.radii[0]);
        sq.scaled_by_radius[1][0] = f32::from_bits(bh::st(t7));
        let t8 = bh::ld(sq.axes[1][1]);
        let t9 = t8 * bh::ld(sq.radii[0]);
        sq.scaled_by_radius[1][1] = f32::from_bits(bh::st(t9));
        let t10 = bh::ld(sq.axes[1][2]);
        let t11 = t10 * bh::ld(sq.radii[0]);
        sq.scaled_by_radius[1][2] = f32::from_bits(bh::st(t11));
        let t12 = bh::ld(sq.axes[0][0]);
        let t13 = t12 * bh::ld(sq.lengths[0]);
        let t14 = t13 * bh::ld(ss.axis_factors[0]);
        sq.scaled_axes[0][0] = f32::from_bits(bh::st(t14));
        let t15 = bh::ld(sq.axes[0][1]);
        let t16 = t15 * bh::ld(sq.lengths[0]);
        let t17 = t16 * bh::ld(ss.axis_factors[1]);
        sq.scaled_axes[0][1] = f32::from_bits(bh::st(t17));
        let t18 = bh::ld(sq.axes[0][2]);
        let t19 = t18 * bh::ld(sq.lengths[0]);
        let t20 = t19 * bh::ld(ss.axis_factors[2]);
        sq.scaled_axes[0][2] = f32::from_bits(bh::st(t20));
        let t21 = bh::ld(sq.axes[1][0]);
        let t22 = t21 * bh::ld(sq.lengths[1]);
        let t23 = t22 * bh::ld(ss.axis_factors[3]);
        sq.scaled_axes[1][0] = f32::from_bits(bh::st(t23));
        let t24 = bh::ld(sq.axes[1][1]);
        let t25 = t24 * bh::ld(sq.lengths[1]);
        let t26 = t25 * bh::ld(ss.axis_factors[4]);
        sq.scaled_axes[1][1] = f32::from_bits(bh::st(t26));
        let t27 = bh::ld(sq.axes[1][2]);
        let t28 = t27 * bh::ld(sq.lengths[1]);
        let t29 = t28 * bh::ld(ss.axis_factors[5]);
        sq.scaled_axes[1][2] = f32::from_bits(bh::st(t29));
        let t30 = bh::ld(sq.scaled_by_radius[0][0]);
        let t31 = t30 * bh::ld(ss.halves[0]);
        sq.half_axes[0][0] = f32::from_bits(bh::st(t31));
        let t32 = bh::ld(sq.scaled_by_radius[0][1]);
        let t33 = t32 * bh::ld(ss.halves[1]);
        sq.half_axes[0][1] = f32::from_bits(bh::st(t33));
        let t34 = bh::ld(sq.scaled_by_radius[0][2]);
        let t35 = t34 * bh::ld(ss.halves[2]);
        sq.half_axes[0][2] = f32::from_bits(bh::st(t35));
        let t36 = bh::ld(sq.scaled_by_radius[1][0]);
        let t37 = t36 * bh::ld(ss.halves[3]);
        sq.half_axes[1][0] = f32::from_bits(bh::st(t37));
        let t38 = bh::ld(sq.scaled_by_radius[1][1]);
        let t39 = t38 * bh::ld(ss.halves[4]);
        sq.half_axes[1][1] = f32::from_bits(bh::st(t39));
        let t40 = bh::ld(sq.scaled_by_radius[1][2]);
        let t41 = t40 * bh::ld(ss.halves[5]);
        sq.half_axes[1][2] = f32::from_bits(bh::st(t41));
    }
}

#[cfg(test)]
mod tests;
