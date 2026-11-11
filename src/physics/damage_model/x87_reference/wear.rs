//! The x87-shaped leaves of `damage_wear` as they were written over `X` (`ld` / `st`),
//! kept for the tests that compare the plain versions against them.

#![allow(clippy::too_many_arguments)]

use crate::math::x87_float::{X, int, ld, st};
use crate::physics::store::DamageModel;
use crate::session::Cell32;

// ---- the arithmetic that was x87 assembly ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o4] = (([m + o0] * [m + o1]) * [m + 0x35B390]) + [rc + o3]`
pub fn add_wear(
    m: &DamageModel,
    o0: &impl Cell32,
    o1: &impl Cell32,
    o3: &impl crate::session::Cell32,
    o4: &impl crate::session::Cell32,
) {
    let t0 = ld(o0.bits());
    let t1 = ld(o1.bits());
    let t2 = t0 * t1;
    let t3 = ld(m.wear_step.bits());
    let t4 = t2 * t3;
    let t5 = ld(o3.bits());
    let t6 = t4 + t5;
    o4.set_bits(st(t6));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o3] = -([m + o0] * [m + 0x35B390]) + [rc + o2]`
pub fn subtract_rate(
    m: &DamageModel,
    o0: &impl Cell32,
    o2: &impl crate::session::Cell32,
    o3: &impl crate::session::Cell32,
) {
    let t0 = ld(o0.bits());
    let t1 = ld(m.wear_step.bits());
    let t2 = t0 * t1;
    let t3 = -t2;
    let t4 = ld(o2.bits());
    let t5 = t3 + t4;
    o3.set_bits(st(t5));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o4] = -(([m + o0] * [m + o1]) * [m + 0x35B390]) + [rc + o3]`
pub fn subtract_wear(
    m: &DamageModel,
    o0: &impl Cell32,
    o1: &impl Cell32,
    o3: &impl crate::session::Cell32,
    o4: &impl crate::session::Cell32,
) {
    let t0 = ld(o0.bits());
    let t1 = ld(o1.bits());
    let t2 = t0 * t1;
    let t3 = ld(m.wear_step.bits());
    let t4 = t2 * t3;
    let t5 = -t4;
    let t6 = ld(o3.bits());
    let t7 = t5 + t6;
    o4.set_bits(st(t7));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B464] = [m + 0x35B398] - [m + 0x35B460]`
pub fn lerp_gap(m: &DamageModel) {
    let t0 = ld(m.engine_speed_share.bits());
    let t1 = ld(m.cooling_capacity.bits());
    let t2 = t0 - t1;
    m.overheat_excess.set_bits(st(t2));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + o1] = [m + o0] * [m + o0]`
pub fn square_into(o0: &impl Cell32, o1: &impl Cell32) {
    let t0 = ld(o0.bits());
    let t1 = t0 * t0;
    o1.set_bits(st(t1));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B454] = (((([m + 0x35B3E0] * [m + 0x35B39C]) * [m + 0x35B6F8]) * [m + 0x35B44C]) * [m + 0x35B468]) * [m + 0x35B3F4]`
/// `[m + 0x35B450] = (([m + 0x35B3E0] * [m + 0x35B39C]) * [m + 0x35B6F8]) * [m + 0x35B3F4]`
/// `[m + 0x35B458] = ((([m + 0x35B3E0] * [m + 0x35B39C]) * [m + 0x35B6F8]) * [m + 0x35B3F4]) * [m + 0x35B448]`
pub fn hit_figures(m: &DamageModel) {
    let t0 = ld(m.point_surface_copy.bits());
    let t1 = ld(m.speed_share.bits());
    let t2 = t0 * t1;
    let t3 = ld(m.figures[32].bits());
    let t4 = t2 * t3;
    let t5 = ld(m.stiffness_body_factor.bits());
    let t6 = t4 * t5;
    let t7 = ld(m.ride_height_body_factor.bits());
    let t8 = t6 * t7;
    let t9 = ld(m.wheels_down.bits());
    let t10 = t8 * t9;
    m.body_roughness_wear.set_bits(st(t10));
    let t11 = ld(m.point_surface_copy.bits());
    let t12 = ld(m.speed_share.bits());
    let t13 = t11 * t12;
    let t14 = ld(m.figures[32].bits());
    let t15 = t13 * t14;
    let t16 = ld(m.wheels_down.bits());
    let t17 = t15 * t16;
    m.roughness_wear_base.set_bits(st(t17));
    let t18 = ld(m.stiffness_suspension_factor.bits());
    let t19 = t17 * t18;
    m.suspension_roughness_wear.set_bits(st(t19));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B484] = ((int [f + 0] * (1 - [m + 0x35B3F0])) + (int [s + 0] * [m + 0x35B3F0])) * [m + 0x35B8BC]`
pub fn blend_wear_figures(f: u32, m: &DamageModel, s: u32) {
    let t0 = int(f as i32);
    let t1 = X(1.0);
    let t2 = ld(m.surface_blend.bits());
    let t3 = t1 - t2;
    let t4 = t0 * t3;
    let t5 = int(s as i32);
    let t6 = ld(m.surface_blend.bits());
    let t7 = t5 * t6;
    let t8 = t4 + t7;
    let t9 = ld(m.wear_figure_scale.bits());
    let t10 = t8 * t9;
    m.wear_figure.set_bits(st(t10));
}
