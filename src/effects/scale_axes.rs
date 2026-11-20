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
    scale_axes_x87(&fl.tables, &fl.state);
}

use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::effects::windscreen_flakes_store::{WindscreenFlakeState, WindscreenFlakeTables};
use crate::math::cell_x87::CellX87;

/// `scale_axes`'s arithmetic as plain Rust on the x87 model, in the same order.
fn scale_axes_x87(sq: &WindscreenFlakeTables, ss: &WindscreenFlakeState) {
    {
        let t0 = sq.axes[0][0].ld();
        let t1 = t0 * sq.radii[0].ld();
        sq.scaled_by_radius[0][0].st(t1);
        let t2 = sq.axes[0][1].ld();
        let t3 = t2 * sq.radii[0].ld();
        sq.scaled_by_radius[0][1].st(t3);
        let t4 = sq.axes[0][2].ld();
        let t5 = t4 * sq.radii[0].ld();
        sq.scaled_by_radius[0][2].st(t5);
        let t6 = sq.axes[1][0].ld();
        let t7 = t6 * sq.radii[0].ld();
        sq.scaled_by_radius[1][0].st(t7);
        let t8 = sq.axes[1][1].ld();
        let t9 = t8 * sq.radii[0].ld();
        sq.scaled_by_radius[1][1].st(t9);
        let t10 = sq.axes[1][2].ld();
        let t11 = t10 * sq.radii[0].ld();
        sq.scaled_by_radius[1][2].st(t11);
        let t12 = sq.axes[0][0].ld();
        let t13 = t12 * sq.lengths[0].ld();
        let t14 = t13 * ss.axis_factors[0].ld();
        sq.scaled_axes[0][0].st(t14);
        let t15 = sq.axes[0][1].ld();
        let t16 = t15 * sq.lengths[0].ld();
        let t17 = t16 * ss.axis_factors[1].ld();
        sq.scaled_axes[0][1].st(t17);
        let t18 = sq.axes[0][2].ld();
        let t19 = t18 * sq.lengths[0].ld();
        let t20 = t19 * ss.axis_factors[2].ld();
        sq.scaled_axes[0][2].st(t20);
        let t21 = sq.axes[1][0].ld();
        let t22 = t21 * sq.lengths[1].ld();
        let t23 = t22 * ss.axis_factors[3].ld();
        sq.scaled_axes[1][0].st(t23);
        let t24 = sq.axes[1][1].ld();
        let t25 = t24 * sq.lengths[1].ld();
        let t26 = t25 * ss.axis_factors[4].ld();
        sq.scaled_axes[1][1].st(t26);
        let t27 = sq.axes[1][2].ld();
        let t28 = t27 * sq.lengths[1].ld();
        let t29 = t28 * ss.axis_factors[5].ld();
        sq.scaled_axes[1][2].st(t29);
        let t30 = sq.scaled_by_radius[0][0].ld();
        let t31 = t30 * ss.halves[0].ld();
        sq.half_axes[0][0].st(t31);
        let t32 = sq.scaled_by_radius[0][1].ld();
        let t33 = t32 * ss.halves[1].ld();
        sq.half_axes[0][1].st(t33);
        let t34 = sq.scaled_by_radius[0][2].ld();
        let t35 = t34 * ss.halves[2].ld();
        sq.half_axes[0][2].st(t35);
        let t36 = sq.scaled_by_radius[1][0].ld();
        let t37 = t36 * ss.halves[3].ld();
        sq.half_axes[1][0].st(t37);
        let t38 = sq.scaled_by_radius[1][1].ld();
        let t39 = t38 * ss.halves[4].ld();
        sq.half_axes[1][1].st(t39);
        let t40 = sq.scaled_by_radius[1][2].ld();
        let t41 = t40 * ss.halves[5].ld();
        sq.half_axes[1][2].st(t41);
    }
}

#[cfg(test)]
mod tests;
