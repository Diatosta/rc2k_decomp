//! The x87-shaped leaves of `damage_factors` as they were written over `X` (`ld` / `st`),
//! kept for the tests that compare the plain versions against them.

#![allow(clippy::too_many_arguments)]

use crate::effects::cell_x87::CellX87;
use crate::math::x87_float::{X, compare, int};
use crate::physics::car_store::Car;
use crate::physics::store::DamageModel;
use crate::session::Cell32;

// ---- the arithmetic that was x87 assembly ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3F8] = int [ride_height] / [m + 0x35B35C]`
/// `[m + 0x35B3FC] = int [suspension_stiffness] / [m + 0x35B35C]`
/// `[m + 0x35B400] = int [brake_balance] / [m + 0x35B35C]`
pub fn scale_car_figures(car: &Car, m: &DamageModel) {
    let t0 = int(crate::physics::flow::whole(&car.ride_height));
    let t1 = m.fixed_point_scale.ld();
    let t2 = t0 / t1;
    m.ride_height_share.st(t2);
    let t3 = int(crate::physics::flow::whole(&car.suspension_stiffness));
    let t4 = m.fixed_point_scale.ld();
    let t5 = t3 / t4;
    m.stiffness_share.st(t5);
    let t6 = int(crate::physics::flow::whole(&car.brake_balance));
    let t7 = m.fixed_point_scale.ld();
    let t8 = t6 / t7;
    m.brake_balance_share.st(t8);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o3] = ([m + 0x35B3FC] * [m + 0x35B34C]) - [m + o2]`
pub fn wheel_stiffness_from_share(
    m: &DamageModel,
    o2: &impl Cell32,
    o3: &impl crate::session::Cell32,
) {
    let t0 = m.stiffness_share.ld();
    let t1 = m.gauge_top.ld();
    let t2 = t0 * t1;
    let t3 = o2.ld();
    let t4 = t2 - t3;
    o3.st(t4);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o2] = [m + 0x35B3F8] * [m + 0x35B34C]`
pub fn wheel_ride_height_from_share(m: &DamageModel, o2: &impl crate::session::Cell32) {
    let t0 = m.ride_height_share.ld();
    let t1 = m.gauge_top.ld();
    let t2 = t0 * t1;
    o2.st(t2);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o1] = [m + 0x35B34C]`
/// `[rc + o3] = [m + 0x35B350]`
pub fn fail_wheel(
    m: &DamageModel,
    o1: &impl crate::session::Cell32,
    o3: &impl crate::session::Cell32,
) {
    let t0 = m.gauge_top.ld();
    o1.st(t0);
    let t1 = m.gauge_low.ld();
    o3.st(t1);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[ride_height_mean] = ((([wheel_ride_height] + [rc + 0x31A414]) + [rc + 0x31A418]) + [rc + 0x31A41C]) * [m + 0x35B364]`
/// `[stiffness_mean] = ((([wheel_stiffness] + [rc + 0x31A424]) + [rc + 0x31A428]) + [rc + 0x31A42C]) * [m + 0x35B364]`
/// `[m + 0x35B3E4] = [drive_kind]`
/// `[m + 0x35B36C] = |[engine_speed]|`
pub fn means_and_engine_speed(car: &Car, m: &DamageModel) {
    let t0 = car.wheel_ride_height[0].ld();
    let t1 = car.wheel_ride_height[1].ld();
    let t2 = t0 + t1;
    let t3 = car.wheel_ride_height[2].ld();
    let t4 = t2 + t3;
    let t5 = car.wheel_ride_height[3].ld();
    let t6 = t4 + t5;
    let t7 = m.quarter.ld();
    let t8 = t6 * t7;
    car.ride_height_mean.st(t8);
    let t9 = car.wheel_stiffness[0].ld();
    let t10 = car.wheel_stiffness[1].ld();
    let t11 = t9 + t10;
    let t12 = car.wheel_stiffness[2].ld();
    let t13 = t11 + t12;
    let t14 = car.wheel_stiffness[3].ld();
    let t15 = t13 + t14;
    let t16 = m.quarter.ld();
    let t17 = t15 * t16;
    car.stiffness_mean.st(t17);
    let t18 = car.drive_kind.ld();
    m.drive_kind.st(t18);
    let t19 = car.engine_speed.ld();
    let t20 = t19.abs();
    m.scratch.st(t20);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B398] = ([m + 0x35B36C] / [engine_speed_limit]) * [m + 0x35B34C]`
/// `[m + 0x35B36C] = |[rc + 0x31A184]|`
pub fn engine_speed_share_step(car: &Car, m: &DamageModel) {
    let t0 = m.scratch.ld();
    let t1 = car.engine_speed_limit.ld();
    let t2 = t0 / t1;
    let t3 = m.gauge_top.ld();
    let t4 = t2 * t3;
    m.engine_speed_share.st(t4);
    let t5 = car.local_velocity[2].ld();
    let t6 = t5.abs();
    m.scratch.st(t6);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B39C] = ([m + 0x35B36C] / [m + 0x35B378]) * [m + 0x35B34C]`
/// `[m + 0x35B3E0] = [rc + 0x31A930]`
/// `[m + 0x35B36C] = |([rc + 0x31A184] - [drive_speed_a])|`
pub fn speed_share_step(car: &Car, m: &DamageModel) {
    let t0 = m.scratch.ld();
    let t1 = m.speed_range.ld();
    let t2 = t0 / t1;
    let t3 = m.gauge_top.ld();
    let t4 = t2 * t3;
    m.speed_share.st(t4);
    let t5 = car.point_wear_factor[4].ld();
    m.point_surface_copy.st(t5);
    let t6 = car.local_velocity[2].ld();
    let t7 = car.drive_speed_a.ld();
    let t8 = t6 - t7;
    let t9 = t8.abs();
    m.scratch.st(t9);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3A0] = [m + 0x35B36C] / [m + 0x35B378]`
/// `[m + 0x35B36C] = |([rc + 0x31A184] - [drive_speed_b])|`
pub fn drive_a_slip_step(car: &Car, m: &DamageModel) {
    let t0 = m.scratch.ld();
    let t1 = m.speed_range.ld();
    let t2 = t0 / t1;
    m.drive_a_slip_share.st(t2);
    let t3 = car.local_velocity[2].ld();
    let t4 = car.drive_speed_b.ld();
    let t5 = t3 - t4;
    let t6 = t5.abs();
    m.scratch.st(t6);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + o2] = [m + 0x35B36C] / [m + 0x35B378]`
/// `[m + 0x35B36C] = |((([rc + o3] - [rc + 0x31A184]) * [m + 0x35B6F4]) + [rc + o6])|`
pub fn drive_extrapolation_step(
    car: &Car,
    m: &DamageModel,
    o2: &impl Cell32,
    o3: &impl crate::session::Cell32,
    o6: &impl crate::session::Cell32,
) {
    let t0 = m.scratch.ld();
    let t1 = m.speed_range.ld();
    let t2 = t0 / t1;
    o2.st(t2);
    let t3 = o3.ld();
    let t4 = car.local_velocity[2].ld();
    let t5 = t3 - t4;
    let t6 = m.figures[31].ld();
    let t7 = t5 * t6;
    let t8 = o6.ld();
    let t9 = t7 + t8;
    let t10 = t9.abs();
    m.scratch.st(t10);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3AC] = [m + 0x35B36C] / [m + 0x35B378]`
/// `[m + 0x35B3F4] = 0`
pub fn finish_shares(m: &DamageModel) {
    let t0 = m.scratch.ld();
    let t1 = m.speed_range.ld();
    let t2 = t0 / t1;
    m.drive_b_extrapolated_share.st(t2);
    let t3 = X(0.0);
    m.wheels_down.st(t3);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3B0] = int [brake] / [m + 0x35B35C]`
pub fn brake_share_step(car: &Car, m: &DamageModel) {
    let t0 = int(crate::physics::flow::whole(&car.brake));
    let t1 = m.fixed_point_scale.ld();
    let t2 = t0 / t1;
    m.brake_share.st(t2);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `status [s + 0] = status`
pub fn engine_over_limit_status(car: &Car, m: &DamageModel) -> u16 {
    let t0 = car.engine_speed_limit.ld();
    let t1 = m.engine_limit_factor.ld();
    let t2 = t0 * t1;
    let t3 = car.engine_speed.ld();
    compare(t2, t3)
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3C8] = (([contact_depth] * [contact_depth]) / [m + 0x35B394]) / [m + 0x35B378]`
/// `[m + 0x35B36C] = -[rc + 0x31A180] - [m + 0x35B624]`
pub fn hit_size_and_dive(car: &Car, m: &DamageModel) {
    let t0 = car.contact_depth.ld();
    let t1 = car.contact_depth.ld();
    let t2 = t0 * t1;
    let t3 = m.hit_divisor.ld();
    let t4 = t2 / t3;
    let t5 = m.speed_range.ld();
    let t6 = t4 / t5;
    m.hit_size.st(t6);
    let t7 = car.local_velocity[1].ld();
    let t8 = -t7;
    let t9 = m.gauge_rates[21].ld();
    let t10 = t8 - t9;
    m.scratch.st(t10);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B36C] = ([m + 0x35B36C] * [m + 0x35B36C]) * [m + 0x35B620]`
pub fn square_dive(m: &DamageModel) {
    let t0 = m.scratch.ld();
    let t1 = t0 * t0;
    let t2 = m.gauge_rates[20].ld();
    let t3 = t1 * t2;
    m.scratch.st(t3);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3DC] = ((([m + 0x35B3CC] + [m + 0x35B3D0]) + [m + 0x35B3D4]) + [m + 0x35B3D8]) * [m + 0x35B364]`
/// `[m + 0x35B36C] = [m + 0x35B3DC] * [m + 0x35B360]`
/// `[m + 0x35B3CC] = ([m + 0x35B3CC] * [m + 0x35B360]) + [m + 0x35B36C]`
/// `[m + 0x35B3D0] = ([m + 0x35B3D0] * [m + 0x35B360]) + [m + 0x35B36C]`
/// `[m + 0x35B3D4] = ([m + 0x35B3D4] * [m + 0x35B360]) + [m + 0x35B36C]`
/// `[m + 0x35B3D8] = ([m + 0x35B3D8] * [m + 0x35B360]) + [m + 0x35B36C]`
pub fn blend_wheel_loads(m: &DamageModel) {
    let t0 = m.wheel_loads[0].ld();
    let t1 = m.wheel_loads[1].ld();
    let t2 = t0 + t1;
    let t3 = m.wheel_loads[2].ld();
    let t4 = t2 + t3;
    let t5 = m.wheel_loads[3].ld();
    let t6 = t4 + t5;
    let t7 = m.quarter.ld();
    let t8 = t6 * t7;
    m.wheel_load_mean.st(t8);
    let t9 = m.wheel_load_mean.ld();
    let t10 = m.half.ld();
    let t11 = t9 * t10;
    m.scratch.st(t11);
    let t12 = m.wheel_loads[0].ld();
    let t13 = m.half.ld();
    let t14 = t12 * t13;
    let t15 = m.scratch.ld();
    let t16 = t14 + t15;
    m.wheel_loads[0].st(t16);
    let t17 = m.wheel_loads[1].ld();
    let t18 = m.half.ld();
    let t19 = t17 * t18;
    let t20 = m.scratch.ld();
    let t21 = t19 + t20;
    m.wheel_loads[1].st(t21);
    let t22 = m.wheel_loads[2].ld();
    let t23 = m.half.ld();
    let t24 = t22 * t23;
    let t25 = m.scratch.ld();
    let t26 = t24 + t25;
    m.wheel_loads[2].st(t26);
    let t27 = m.wheel_loads[3].ld();
    let t28 = m.half.ld();
    let t29 = t27 * t28;
    let t30 = m.scratch.ld();
    let t31 = t29 + t30;
    m.wheel_loads[3].st(t31);
}
