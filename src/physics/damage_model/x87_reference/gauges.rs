//! The x87-shaped leaves of `damage_gauges` as they were written over `X` (`ld` / `st`),
//! kept for the tests that compare the plain versions against them.

#![allow(clippy::too_many_arguments)]

use crate::math::x87_float::{X, ld, st};
use crate::physics::car_store::Car;
use crate::physics::store::DamageModel;
use crate::race::work_store::RaceWork;
use crate::session::Cell32;

// ---- the arithmetic that was x87 assembly ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o3] = ([m + o0] * [m + o1]) + [rc + o2]`
pub fn add_product(
    o0: &impl Cell32,
    o1: &impl Cell32,
    o2: &impl crate::session::Cell32,
    o3: &impl crate::session::Cell32,
) {
    let t0 = ld(o0.bits());
    let t1 = ld(o1.bits());
    let t2 = t0 * t1;
    let t3 = ld(o2.bits());
    let t4 = t2 + t3;
    o3.set_bits(st(t4));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o5] = ((([m + o0] * [m + o1]) * [m + o2]) * [m + 0x35B38C]) + [rc + o4]`
pub fn add_scaled_triple(
    m: &DamageModel,
    o0: &impl Cell32,
    o1: &impl Cell32,
    o2: &impl Cell32,
    o4: &impl crate::session::Cell32,
    o5: &impl crate::session::Cell32,
) {
    let t0 = ld(o0.bits());
    let t1 = ld(o1.bits());
    let t2 = t0 * t1;
    let t3 = ld(o2.bits());
    let t4 = t2 * t3;
    let t5 = ld(m.gauge_step.bits());
    let t6 = t4 * t5;
    let t7 = ld(o4.bits());
    let t8 = t6 + t7;
    o5.set_bits(st(t8));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[v + 0] = ((([m + 0x35B398] * [m + 0x35B5A8]) * [m + 0x35B370]) * [t + 0]) + [v + 0]`
pub fn gear_wear_from_speed(m: &DamageModel, t: &impl Cell32, v: &impl Cell32) {
    let t0 = ld(m.engine_speed_share.bits());
    let t1 = ld(m.wear_rates[12].bits());
    let t2 = t0 * t1;
    let t3 = ld(m.hundredth.bits());
    let t4 = t2 * t3;
    let t5 = ld(t.bits());
    let t6 = t4 * t5;
    let t7 = ld(v.bits());
    let t8 = t6 + t7;
    v.set_bits(st(t8));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B43C] = 1 - [m + 0x35B400]`
/// `[brake_wear_a] = ((([m + 0x35B5F4] * [m + 0x35B400]) * [m + 0x35B3B0]) * [m + 0x35B38C]) + [brake_wear_a]`
pub fn brake_wear_step(car: &Car, m: &DamageModel) {
    let t0 = X(1.0);
    let t1 = ld(m.brake_balance_share.bits());
    let t2 = t0 - t1;
    m.brake_balance_complement.set_bits(st(t2));
    let t3 = ld(m.gauge_rates[9].bits());
    let t4 = ld(m.brake_balance_share.bits());
    let t5 = t3 * t4;
    let t6 = ld(m.brake_share.bits());
    let t7 = t5 * t6;
    let t8 = ld(m.gauge_step.bits());
    let t9 = t7 * t8;
    let t10 = ld(car.brake_wear_a.bits());
    let t11 = t9 + t10;
    car.brake_wear_a.set_bits(st(t11));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o4] = (([m + o0] * [m + o1]) * [m + o2]) + [rc + o3]`
pub fn add_triple_product(
    o0: &impl Cell32,
    o1: &impl Cell32,
    o2: &impl Cell32,
    o3: &impl crate::session::Cell32,
    o4: &impl crate::session::Cell32,
) {
    let t0 = ld(o0.bits());
    let t1 = ld(o1.bits());
    let t2 = t0 * t1;
    let t3 = ld(o2.bits());
    let t4 = t2 * t3;
    let t5 = ld(o3.bits());
    let t6 = t4 + t5;
    o4.set_bits(st(t6));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B488] = (([m + 0x35B39C] / ([m + 0x35B484] * [m + 0x35B380])) * [m + 0x35B34C]) * [m + 0x35B3F4]`
/// `[m + 0x35B48C] = (([m + 0x35B39C] / ([m + 0x35B484] * [m + 0x35B380])) * [m + 0x35B34C]) * [m + 0x35B3F4]`
pub fn part_hit_figures(m: &DamageModel) {
    let t0 = ld(m.speed_share.bits());
    let t1 = ld(m.wear_figure.bits());
    let t2 = ld(m.metres_per_mile.bits());
    let t3 = t1 * t2;
    let t4 = t0 / t3;
    let t5 = ld(m.gauge_top.bits());
    let t6 = t4 * t5;
    let t7 = ld(m.wheels_down.bits());
    let t8 = t6 * t7;
    m.front_tyre_wear_rate.set_bits(st(t8));
    m.rear_tyre_wear_rate.set_bits(st(t8));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B488] = [m + 0x35B488] * [t + 0]`
/// `[m + 0x35B48C] = [m + 0x35B48C] * [u + 0]`
pub fn scale_part_hit_figures(m: &DamageModel, t: &impl Cell32, u: &impl Cell32) {
    let t0 = ld(m.front_tyre_wear_rate.bits());
    let t1 = ld(t.bits());
    let t2 = t0 * t1;
    m.front_tyre_wear_rate.set_bits(st(t2));
    let t3 = ld(m.rear_tyre_wear_rate.bits());
    let t4 = ld(u.bits());
    let t5 = t3 * t4;
    m.rear_tyre_wear_rate.set_bits(st(t5));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[engine_damage] = (([engine_heat] * [m + 0x35B608]) * [m + 0x35B370]) + [engine_damage]`
pub fn engine_damage_carry(car: &Car, m: &DamageModel) {
    let t0 = ld(car.engine_heat.bits());
    let t1 = ld(m.gauge_rates[14].bits());
    let t2 = t0 * t1;
    let t3 = ld(m.hundredth.bits());
    let t4 = t2 * t3;
    let t5 = ld(car.engine_damage.bits());
    let t6 = t4 + t5;
    car.engine_damage.set_bits(st(t6));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[clutch_damage] = [clutch_damage] + [m + o1]`
pub fn add_rate_to_clutch(car: &Car, o1: &impl Cell32) {
    let t0 = ld(car.clutch_damage.bits());
    let t1 = ld(o1.bits());
    let t2 = t0 + t1;
    car.clutch_damage.set_bits(st(t2));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[v + 0] = [v + 0] + [m + o1]`
pub fn add_rate_to_gauge(v: &impl Cell32, o1: &impl Cell32) {
    let t0 = ld(v.bits());
    let t1 = ld(o1.bits());
    let t2 = t0 + t1;
    v.set_bits(st(t2));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3C8] = [m + 0x35B3C8] * [rc + 0x31C708]`
pub fn scale_hit_size(work: &RaceWork, m: &DamageModel) {
    let t0 = ld(m.hit_size.bits());
    let t1 = ld(work.patch_factors[1].bits());
    let t2 = t0 * t1;
    m.hit_size.set_bits(st(t2));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[v + 0] = ([m + 0x35B3C8] * [m + 0x35B6CC]) + [v + 0]`
pub fn add_hit_to_gear_gauge(m: &DamageModel, v: &impl Cell32) {
    let t0 = ld(m.hit_size.bits());
    let t1 = ld(m.figures[21].bits());
    let t2 = t0 * t1;
    let t3 = ld(v.bits());
    let t4 = t2 + t3;
    v.set_bits(st(t4));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[lights_damage] = ((((([light_damage] + [rc + 0x31A2F0]) + [rc + 0x31A2F4]) + [rc + 0x31A2F8]) + [rc + 0x31A300]) + [rc + 0x31A2FC]) / [m + 0x35B344]`
/// `[brake_damage] = ([brake_wear_a] + [brake_wear_b]) * [m + 0x35B360]`
/// `[m + 0x35B440] = ([tyre_wears] + [rc + 0x31A330]) * [m + 0x35B360]`
/// `[m + 0x35B444] = ([rc + 0x31A334] + [rc + 0x31A338]) * [m + 0x35B360]`
/// `[tyre_wear] = ([m + 0x35B440] + [m + 0x35B444]) * [m + 0x35B360]`
/// (a long formula: see the instructions in the tests' `asm_reference`)
/// (a long formula: see the instructions in the tests' `asm_reference`)
/// `[overall_suspension_damage] = ((([suspension_damage] + [rc + 0x31A310]) + [rc + 0x31A314]) + [rc + 0x31A318]) * [m + 0x35B364]`
/// `[stiffness_mean] = ((([wheel_stiffness] + [rc + 0x31A424]) + [rc + 0x31A428]) + [rc + 0x31A42C]) * [m + 0x35B364]`
pub fn gauge_totals(car: &Car, m: &DamageModel) {
    let t0 = ld(car.light_damage[0].bits());
    let t1 = ld(car.light_damage[1].bits());
    let t2 = t0 + t1;
    let t3 = ld(car.light_damage[2].bits());
    let t4 = t2 + t3;
    let t5 = ld(car.light_damage[3].bits());
    let t6 = t4 + t5;
    let t7 = ld(car.light_damage[5].bits());
    let t8 = t6 + t7;
    let t9 = ld(car.light_damage[4].bits());
    let t10 = t8 + t9;
    let t11 = ld(m.light_count_divisor.bits());
    let t12 = t10 / t11;
    car.lights_damage.set_bits(st(t12));
    let t13 = ld(car.brake_wear_a.bits());
    let t14 = ld(car.brake_wear_b.bits());
    let t15 = t13 + t14;
    let t16 = ld(m.half.bits());
    let t17 = t15 * t16;
    car.brake_damage.set_bits(st(t17));
    let t18 = ld(car.tyre_wears[0].bits());
    let t19 = ld(car.tyre_wears[1].bits());
    let t20 = t18 + t19;
    let t21 = ld(m.half.bits());
    let t22 = t20 * t21;
    m.front_tyre_wear.set_bits(st(t22));
    let t23 = ld(car.tyre_wears[2].bits());
    let t24 = ld(car.tyre_wears[3].bits());
    let t25 = t23 + t24;
    let t26 = ld(m.half.bits());
    let t27 = t25 * t26;
    m.rear_tyre_wear.set_bits(st(t27));
    let t28 = ld(m.front_tyre_wear.bits());
    let t29 = ld(m.rear_tyre_wear.bits());
    let t30 = t28 + t29;
    let t31 = ld(m.half.bits());
    let t32 = t30 * t31;
    car.tyre_wear.set_bits(st(t32));
    let t33 = ld(car.gear_wear[0].bits());
    let t34 = ld(car.gear_wear[2].bits());
    let t35 = t33 + t34;
    let t36 = ld(car.gear_wear[3].bits());
    let t37 = t35 + t36;
    let t38 = ld(car.gear_wear[4].bits());
    let t39 = t37 + t38;
    let t40 = ld(car.gear_wear[5].bits());
    let t41 = t39 + t40;
    let t42 = ld(car.gear_wear[6].bits());
    let t43 = t41 + t42;
    let t44 = ld(car.gear_wear[7].bits());
    let t45 = t43 + t44;
    let t46 = ld(car.gear_wear[8].bits());
    let t47 = t45 + t46;
    let t48 = ld(m.half.bits());
    let t49 = t47 * t48;
    let t50 = ld(m.half.bits());
    let t51 = t49 * t50;
    let t52 = ld(m.half.bits());
    let t53 = t51 * t52;
    car.gearbox_damage.set_bits(st(t53));
    let t54 = ld(car.wheel_hit_damage[0].bits());
    let t55 = ld(car.wheel_hit_damage[1].bits());
    let t56 = t54 + t55;
    let t57 = ld(car.wheel_hit_damage[2].bits());
    let t58 = t56 + t57;
    let t59 = ld(car.wheel_hit_damage[3].bits());
    let t60 = t58 + t59;
    let t61 = ld(car.body_hit_damage.bits());
    let t62 = t60 + t61;
    let t63 = ld(car.body_hit_damage.bits());
    let t64 = t62 + t63;
    let t65 = ld(car.body_hit_damage.bits());
    let t66 = t64 + t65;
    let t67 = ld(car.body_hit_damage.bits());
    let t68 = t66 + t67;
    let t69 = ld(car.body_hit_damage.bits());
    let t70 = t68 + t69;
    let t71 = ld(car.body_hit_damage.bits());
    let t72 = t70 + t71;
    let t73 = ld(m.tenth.bits());
    let t74 = t72 * t73;
    car.body_damage.set_bits(st(t74));
    let t75 = ld(car.suspension_damage[0].bits());
    let t76 = ld(car.suspension_damage[1].bits());
    let t77 = t75 + t76;
    let t78 = ld(car.suspension_damage[2].bits());
    let t79 = t77 + t78;
    let t80 = ld(car.suspension_damage[3].bits());
    let t81 = t79 + t80;
    let t82 = ld(m.quarter.bits());
    let t83 = t81 * t82;
    car.overall_suspension_damage.set_bits(st(t83));
    let t84 = ld(car.wheel_stiffness[0].bits());
    let t85 = ld(car.wheel_stiffness[1].bits());
    let t86 = t84 + t85;
    let t87 = ld(car.wheel_stiffness[2].bits());
    let t88 = t86 + t87;
    let t89 = ld(car.wheel_stiffness[3].bits());
    let t90 = t88 + t89;
    let t91 = ld(m.quarter.bits());
    let t92 = t90 * t91;
    car.stiffness_mean.set_bits(st(t92));
}
