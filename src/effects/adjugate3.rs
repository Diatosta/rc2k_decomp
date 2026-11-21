//! `effects::adjugate3`: the adjugate of the 3 x 3 matrix (rows 12 bytes apart) into the
//! result: each entry a difference of two products, rounded as written (so an output over
//! the input reads back what was written).
//!
//! The arithmetic is plain Rust on the x87 model (`math::x87_float`).

/// `s` the matrix, `d` the result (another array).
pub fn adjugate3(s: &[f32; 9], d: &mut [f32; 9]) {
    adjugate3_x87(s, d);
}

use crate::math::x87_float::{ld, stf};

/// `adjugate3`'s x87 block as plain Rust on the x87 model, in the same order and over the
/// same memory places.
fn adjugate3_x87(s: &[f32; 9], d: &mut [f32; 9]) {
    let t0 = ld(s[4]);
    let t1 = t0 * ld(s[8]);
    let t2 = ld(s[5]);
    let t3 = t2 * ld(s[7]);
    let t4 = t1 - t3;
    d[0] = stf(t4);
    let t5 = ld(s[2]);
    let t6 = t5 * ld(s[7]);
    let t7 = ld(s[1]);
    let t8 = t7 * ld(s[8]);
    let t9 = t6 - t8;
    d[1] = stf(t9);
    let t10 = ld(s[1]);
    let t11 = t10 * ld(s[5]);
    let t12 = ld(s[2]);
    let t13 = t12 * ld(s[4]);
    let t14 = t11 - t13;
    d[2] = stf(t14);
    let t15 = ld(s[5]);
    let t16 = t15 * ld(s[6]);
    let t17 = ld(s[3]);
    let t18 = t17 * ld(s[8]);
    let t19 = t16 - t18;
    d[3] = stf(t19);
    let t20 = ld(s[0]);
    let t21 = t20 * ld(s[8]);
    let t22 = ld(s[2]);
    let t23 = t22 * ld(s[6]);
    let t24 = t21 - t23;
    d[4] = stf(t24);
    let t25 = ld(s[2]);
    let t26 = t25 * ld(s[3]);
    let t27 = ld(s[0]);
    let t28 = t27 * ld(s[5]);
    let t29 = t26 - t28;
    d[5] = stf(t29);
    let t30 = ld(s[3]);
    let t31 = t30 * ld(s[7]);
    let t32 = ld(s[4]);
    let t33 = t32 * ld(s[6]);
    let t34 = t31 - t33;
    d[6] = stf(t34);
    let t35 = ld(s[1]);
    let t36 = t35 * ld(s[6]);
    let t37 = ld(s[0]);
    let t38 = t37 * ld(s[7]);
    let t39 = t36 - t38;
    d[7] = stf(t39);
    let t40 = ld(s[0]);
    let t41 = t40 * ld(s[4]);
    let t42 = ld(s[1]);
    let t43 = t42 * ld(s[3]);
    let t44 = t41 - t43;
    d[8] = stf(t44);
}

#[cfg(test)]
mod tests;
