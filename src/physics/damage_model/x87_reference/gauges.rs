//! The x87-shaped leaves of `damage_gauges` as they were written over `X` (`ld` / `st`),
//! kept for the tests that compare the plain versions against them.

#![allow(clippy::too_many_arguments)]

use crate::effects::cell_x87::CellX87;
use crate::math::x87_float::X;
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
    let t0 = o0.ld();
    let t1 = o1.ld();
    let t2 = t0 * t1;
    let t3 = o2.ld();
    let t4 = t2 + t3;
    o3.st(t4);
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
    let t0 = o0.ld();
    let t1 = o1.ld();
    let t2 = t0 * t1;
    let t3 = o2.ld();
    let t4 = t2 * t3;
    let t5 = m.gauge_step.ld();
    let t6 = t4 * t5;
    let t7 = o4.ld();
    let t8 = t6 + t7;
    o5.st(t8);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[v + 0] = ((([m + 0x35B398] * [m + 0x35B5A8]) * [m + 0x35B370]) * [t + 0]) + [v + 0]`
pub fn gear_wear_from_speed(m: &DamageModel, t: &impl Cell32, v: &impl Cell32) {
    let t0 = m.engine_speed_share.ld();
    let t1 = m.wear_rates[12].ld();
    let t2 = t0 * t1;
    let t3 = m.hundredth.ld();
    let t4 = t2 * t3;
    let t5 = t.ld();
    let t6 = t4 * t5;
    let t7 = v.ld();
    let t8 = t6 + t7;
    v.st(t8);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B43C] = 1 - [m + 0x35B400]`
/// `[brake_wear_a] = ((([m + 0x35B5F4] * [m + 0x35B400]) * [m + 0x35B3B0]) * [m + 0x35B38C]) + [brake_wear_a]`
pub fn brake_wear_step(car: &Car, m: &DamageModel) {
    let t0 = X(1.0);
    let t1 = m.brake_balance_share.ld();
    let t2 = t0 - t1;
    m.brake_balance_complement.st(t2);
    let t3 = m.gauge_rates[9].ld();
    let t4 = m.brake_balance_share.ld();
    let t5 = t3 * t4;
    let t6 = m.brake_share.ld();
    let t7 = t5 * t6;
    let t8 = m.gauge_step.ld();
    let t9 = t7 * t8;
    let t10 = car.brake_wear_a.ld();
    let t11 = t9 + t10;
    car.brake_wear_a.st(t11);
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
    let t0 = o0.ld();
    let t1 = o1.ld();
    let t2 = t0 * t1;
    let t3 = o2.ld();
    let t4 = t2 * t3;
    let t5 = o3.ld();
    let t6 = t4 + t5;
    o4.st(t6);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B488] = (([m + 0x35B39C] / ([m + 0x35B484] * [m + 0x35B380])) * [m + 0x35B34C]) * [m + 0x35B3F4]`
/// `[m + 0x35B48C] = (([m + 0x35B39C] / ([m + 0x35B484] * [m + 0x35B380])) * [m + 0x35B34C]) * [m + 0x35B3F4]`
pub fn part_hit_figures(m: &DamageModel) {
    let t0 = m.speed_share.ld();
    let t1 = m.wear_figure.ld();
    let t2 = m.metres_per_mile.ld();
    let t3 = t1 * t2;
    let t4 = t0 / t3;
    let t5 = m.gauge_top.ld();
    let t6 = t4 * t5;
    let t7 = m.wheels_down.ld();
    let t8 = t6 * t7;
    m.front_tyre_wear_rate.st(t8);
    m.rear_tyre_wear_rate.st(t8);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B488] = [m + 0x35B488] * [t + 0]`
/// `[m + 0x35B48C] = [m + 0x35B48C] * [u + 0]`
pub fn scale_part_hit_figures(m: &DamageModel, t: &impl Cell32, u: &impl Cell32) {
    let t0 = m.front_tyre_wear_rate.ld();
    let t1 = t.ld();
    let t2 = t0 * t1;
    m.front_tyre_wear_rate.st(t2);
    let t3 = m.rear_tyre_wear_rate.ld();
    let t4 = u.ld();
    let t5 = t3 * t4;
    m.rear_tyre_wear_rate.st(t5);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[engine_damage] = (([engine_heat] * [m + 0x35B608]) * [m + 0x35B370]) + [engine_damage]`
pub fn engine_damage_carry(car: &Car, m: &DamageModel) {
    let t0 = car.engine_heat.ld();
    let t1 = m.gauge_rates[14].ld();
    let t2 = t0 * t1;
    let t3 = m.hundredth.ld();
    let t4 = t2 * t3;
    let t5 = car.engine_damage.ld();
    let t6 = t4 + t5;
    car.engine_damage.st(t6);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[clutch_damage] = [clutch_damage] + [m + o1]`
pub fn add_rate_to_clutch(car: &Car, o1: &impl Cell32) {
    let t0 = car.clutch_damage.ld();
    let t1 = o1.ld();
    let t2 = t0 + t1;
    car.clutch_damage.st(t2);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[v + 0] = [v + 0] + [m + o1]`
pub fn add_rate_to_gauge(v: &impl Cell32, o1: &impl Cell32) {
    let t0 = v.ld();
    let t1 = o1.ld();
    let t2 = t0 + t1;
    v.st(t2);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3C8] = [m + 0x35B3C8] * [rc + 0x31C708]`
pub fn scale_hit_size(work: &RaceWork, m: &DamageModel) {
    let t0 = m.hit_size.ld();
    let t1 = work.patch_factors[1].ld();
    let t2 = t0 * t1;
    m.hit_size.st(t2);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[v + 0] = ([m + 0x35B3C8] * [m + 0x35B6CC]) + [v + 0]`
pub fn add_hit_to_gear_gauge(m: &DamageModel, v: &impl Cell32) {
    let t0 = m.hit_size.ld();
    let t1 = m.figures[21].ld();
    let t2 = t0 * t1;
    let t3 = v.ld();
    let t4 = t2 + t3;
    v.st(t4);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[lights_damage] = ((((([light_damage] + [rc + 0x31A2F0]) + [rc + 0x31A2F4]) + [rc + 0x31A2F8]) + [rc + 0x31A300]) + [rc + 0x31A2FC]) / [m + 0x35B344]`
/// `[brake_damage] = ([brake_wear_a] + [brake_wear_b]) * [m + 0x35B360]`
/// `[m + 0x35B440] = ([tyre_wears] + [rc + 0x31A330]) * [m + 0x35B360]`
/// `[m + 0x35B444] = ([rc + 0x31A334] + [rc + 0x31A338]) * [m + 0x35B360]`
/// `[tyre_wear] = ([m + 0x35B440] + [m + 0x35B444]) * [m + 0x35B360]`
/// (a long formula: checked by the tests' `reference_vectors`)
/// (a long formula: checked by the tests' `reference_vectors`)
/// `[overall_suspension_damage] = ((([suspension_damage] + [rc + 0x31A310]) + [rc + 0x31A314]) + [rc + 0x31A318]) * [m + 0x35B364]`
/// `[stiffness_mean] = ((([wheel_stiffness] + [rc + 0x31A424]) + [rc + 0x31A428]) + [rc + 0x31A42C]) * [m + 0x35B364]`
pub fn gauge_totals(car: &Car, m: &DamageModel) {
    let t0 = car.light_damage[0].ld();
    let t1 = car.light_damage[1].ld();
    let t2 = t0 + t1;
    let t3 = car.light_damage[2].ld();
    let t4 = t2 + t3;
    let t5 = car.light_damage[3].ld();
    let t6 = t4 + t5;
    let t7 = car.light_damage[5].ld();
    let t8 = t6 + t7;
    let t9 = car.light_damage[4].ld();
    let t10 = t8 + t9;
    let t11 = m.light_count_divisor.ld();
    let t12 = t10 / t11;
    car.lights_damage.st(t12);
    let t13 = car.brake_wear_a.ld();
    let t14 = car.brake_wear_b.ld();
    let t15 = t13 + t14;
    let t16 = m.half.ld();
    let t17 = t15 * t16;
    car.brake_damage.st(t17);
    let t18 = car.tyre_wears[0].ld();
    let t19 = car.tyre_wears[1].ld();
    let t20 = t18 + t19;
    let t21 = m.half.ld();
    let t22 = t20 * t21;
    m.front_tyre_wear.st(t22);
    let t23 = car.tyre_wears[2].ld();
    let t24 = car.tyre_wears[3].ld();
    let t25 = t23 + t24;
    let t26 = m.half.ld();
    let t27 = t25 * t26;
    m.rear_tyre_wear.st(t27);
    let t28 = m.front_tyre_wear.ld();
    let t29 = m.rear_tyre_wear.ld();
    let t30 = t28 + t29;
    let t31 = m.half.ld();
    let t32 = t30 * t31;
    car.tyre_wear.st(t32);
    let t33 = car.gear_wear[0].ld();
    let t34 = car.gear_wear[2].ld();
    let t35 = t33 + t34;
    let t36 = car.gear_wear[3].ld();
    let t37 = t35 + t36;
    let t38 = car.gear_wear[4].ld();
    let t39 = t37 + t38;
    let t40 = car.gear_wear[5].ld();
    let t41 = t39 + t40;
    let t42 = car.gear_wear[6].ld();
    let t43 = t41 + t42;
    let t44 = car.gear_wear[7].ld();
    let t45 = t43 + t44;
    let t46 = car.gear_wear[8].ld();
    let t47 = t45 + t46;
    let t48 = m.half.ld();
    let t49 = t47 * t48;
    let t50 = m.half.ld();
    let t51 = t49 * t50;
    let t52 = m.half.ld();
    let t53 = t51 * t52;
    car.gearbox_damage.st(t53);
    let t54 = car.wheel_hit_damage[0].ld();
    let t55 = car.wheel_hit_damage[1].ld();
    let t56 = t54 + t55;
    let t57 = car.wheel_hit_damage[2].ld();
    let t58 = t56 + t57;
    let t59 = car.wheel_hit_damage[3].ld();
    let t60 = t58 + t59;
    let t61 = car.body_hit_damage.ld();
    let t62 = t60 + t61;
    let t63 = car.body_hit_damage.ld();
    let t64 = t62 + t63;
    let t65 = car.body_hit_damage.ld();
    let t66 = t64 + t65;
    let t67 = car.body_hit_damage.ld();
    let t68 = t66 + t67;
    let t69 = car.body_hit_damage.ld();
    let t70 = t68 + t69;
    let t71 = car.body_hit_damage.ld();
    let t72 = t70 + t71;
    let t73 = m.tenth.ld();
    let t74 = t72 * t73;
    car.body_damage.st(t74);
    let t75 = car.suspension_damage[0].ld();
    let t76 = car.suspension_damage[1].ld();
    let t77 = t75 + t76;
    let t78 = car.suspension_damage[2].ld();
    let t79 = t77 + t78;
    let t80 = car.suspension_damage[3].ld();
    let t81 = t79 + t80;
    let t82 = m.quarter.ld();
    let t83 = t81 * t82;
    car.overall_suspension_damage.st(t83);
    let t84 = car.wheel_stiffness[0].ld();
    let t85 = car.wheel_stiffness[1].ld();
    let t86 = t84 + t85;
    let t87 = car.wheel_stiffness[2].ld();
    let t88 = t86 + t87;
    let t89 = car.wheel_stiffness[3].ld();
    let t90 = t88 + t89;
    let t91 = m.quarter.ld();
    let t92 = t90 * t91;
    car.stiffness_mean.st(t92);
}
