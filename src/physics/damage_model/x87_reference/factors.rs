//! The x87-shaped leaves of `damage_factors` as they were written over `X` (`ld` / `st`),
//! kept for the tests that compare the plain versions against them.

#![allow(clippy::too_many_arguments)]

use crate::math::x87_float::{X, compare, int, ld, st};
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
    let t0 = int(car.ride_height.bits() as i32);
    let t1 = ld(m.fixed_point_scale.bits());
    let t2 = t0 / t1;
    m.ride_height_share.set_bits(st(t2));
    let t3 = int(car.suspension_stiffness.bits() as i32);
    let t4 = ld(m.fixed_point_scale.bits());
    let t5 = t3 / t4;
    m.stiffness_share.set_bits(st(t5));
    let t6 = int(car.brake_balance.bits() as i32);
    let t7 = ld(m.fixed_point_scale.bits());
    let t8 = t6 / t7;
    m.brake_balance_share.set_bits(st(t8));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o3] = ([m + 0x35B3FC] * [m + 0x35B34C]) - [m + o2]`
pub fn wheel_stiffness_from_share(
    m: &DamageModel,
    o2: &impl Cell32,
    o3: &impl crate::session::Cell32,
) {
    let t0 = ld(m.stiffness_share.bits());
    let t1 = ld(m.gauge_top.bits());
    let t2 = t0 * t1;
    let t3 = ld(o2.bits());
    let t4 = t2 - t3;
    o3.set_bits(st(t4));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o2] = [m + 0x35B3F8] * [m + 0x35B34C]`
pub fn wheel_ride_height_from_share(m: &DamageModel, o2: &impl crate::session::Cell32) {
    let t0 = ld(m.ride_height_share.bits());
    let t1 = ld(m.gauge_top.bits());
    let t2 = t0 * t1;
    o2.set_bits(st(t2));
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
    let t0 = ld(m.gauge_top.bits());
    o1.set_bits(st(t0));
    let t1 = ld(m.gauge_low.bits());
    o3.set_bits(st(t1));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[ride_height_mean] = ((([wheel_ride_height] + [rc + 0x31A414]) + [rc + 0x31A418]) + [rc + 0x31A41C]) * [m + 0x35B364]`
/// `[stiffness_mean] = ((([wheel_stiffness] + [rc + 0x31A424]) + [rc + 0x31A428]) + [rc + 0x31A42C]) * [m + 0x35B364]`
/// `[m + 0x35B3E4] = [drive_kind]`
/// `[m + 0x35B36C] = |[engine_speed]|`
pub fn means_and_engine_speed(car: &Car, m: &DamageModel) {
    let t0 = ld(car.wheel_ride_height[0].bits());
    let t1 = ld(car.wheel_ride_height[1].bits());
    let t2 = t0 + t1;
    let t3 = ld(car.wheel_ride_height[2].bits());
    let t4 = t2 + t3;
    let t5 = ld(car.wheel_ride_height[3].bits());
    let t6 = t4 + t5;
    let t7 = ld(m.quarter.bits());
    let t8 = t6 * t7;
    car.ride_height_mean.set_bits(st(t8));
    let t9 = ld(car.wheel_stiffness[0].bits());
    let t10 = ld(car.wheel_stiffness[1].bits());
    let t11 = t9 + t10;
    let t12 = ld(car.wheel_stiffness[2].bits());
    let t13 = t11 + t12;
    let t14 = ld(car.wheel_stiffness[3].bits());
    let t15 = t13 + t14;
    let t16 = ld(m.quarter.bits());
    let t17 = t15 * t16;
    car.stiffness_mean.set_bits(st(t17));
    let t18 = ld(car.drive_kind.bits());
    m.drive_kind.set_bits(st(t18));
    let t19 = ld(car.engine_speed.bits());
    let t20 = t19.abs();
    m.scratch.set_bits(st(t20));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B398] = ([m + 0x35B36C] / [engine_speed_limit]) * [m + 0x35B34C]`
/// `[m + 0x35B36C] = |[rc + 0x31A184]|`
pub fn engine_speed_share_step(car: &Car, m: &DamageModel) {
    let t0 = ld(m.scratch.bits());
    let t1 = ld(car.engine_speed_limit.bits());
    let t2 = t0 / t1;
    let t3 = ld(m.gauge_top.bits());
    let t4 = t2 * t3;
    m.engine_speed_share.set_bits(st(t4));
    let t5 = ld(car.local_velocity[2].bits());
    let t6 = t5.abs();
    m.scratch.set_bits(st(t6));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B39C] = ([m + 0x35B36C] / [m + 0x35B378]) * [m + 0x35B34C]`
/// `[m + 0x35B3E0] = [rc + 0x31A930]`
/// `[m + 0x35B36C] = |([rc + 0x31A184] - [drive_speed_a])|`
pub fn speed_share_step(car: &Car, m: &DamageModel) {
    let t0 = ld(m.scratch.bits());
    let t1 = ld(m.speed_range.bits());
    let t2 = t0 / t1;
    let t3 = ld(m.gauge_top.bits());
    let t4 = t2 * t3;
    m.speed_share.set_bits(st(t4));
    let t5 = ld(car.point_wear_factor[4].bits());
    m.point_surface_copy.set_bits(st(t5));
    let t6 = ld(car.local_velocity[2].bits());
    let t7 = ld(car.drive_speed_a.bits());
    let t8 = t6 - t7;
    let t9 = t8.abs();
    m.scratch.set_bits(st(t9));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3A0] = [m + 0x35B36C] / [m + 0x35B378]`
/// `[m + 0x35B36C] = |([rc + 0x31A184] - [drive_speed_b])|`
pub fn drive_a_slip_step(car: &Car, m: &DamageModel) {
    let t0 = ld(m.scratch.bits());
    let t1 = ld(m.speed_range.bits());
    let t2 = t0 / t1;
    m.drive_a_slip_share.set_bits(st(t2));
    let t3 = ld(car.local_velocity[2].bits());
    let t4 = ld(car.drive_speed_b.bits());
    let t5 = t3 - t4;
    let t6 = t5.abs();
    m.scratch.set_bits(st(t6));
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
    let t0 = ld(m.scratch.bits());
    let t1 = ld(m.speed_range.bits());
    let t2 = t0 / t1;
    o2.set_bits(st(t2));
    let t3 = ld(o3.bits());
    let t4 = ld(car.local_velocity[2].bits());
    let t5 = t3 - t4;
    let t6 = ld(m.figures[31].bits());
    let t7 = t5 * t6;
    let t8 = ld(o6.bits());
    let t9 = t7 + t8;
    let t10 = t9.abs();
    m.scratch.set_bits(st(t10));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3AC] = [m + 0x35B36C] / [m + 0x35B378]`
/// `[m + 0x35B3F4] = 0`
pub fn finish_shares(m: &DamageModel) {
    let t0 = ld(m.scratch.bits());
    let t1 = ld(m.speed_range.bits());
    let t2 = t0 / t1;
    m.drive_b_extrapolated_share.set_bits(st(t2));
    let t3 = X(0.0);
    m.wheels_down.set_bits(st(t3));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3B0] = int [brake] / [m + 0x35B35C]`
pub fn brake_share_step(car: &Car, m: &DamageModel) {
    let t0 = int(car.brake.bits() as i32);
    let t1 = ld(m.fixed_point_scale.bits());
    let t2 = t0 / t1;
    m.brake_share.set_bits(st(t2));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `status [s + 0] = status`
pub fn engine_over_limit_status(car: &Car, m: &DamageModel) -> u16 {
    let t0 = ld(car.engine_speed_limit.bits());
    let t1 = ld(m.engine_limit_factor.bits());
    let t2 = t0 * t1;
    let t3 = ld(car.engine_speed.bits());
    compare(t2, t3)
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3C8] = (([contact_depth] * [contact_depth]) / [m + 0x35B394]) / [m + 0x35B378]`
/// `[m + 0x35B36C] = -[rc + 0x31A180] - [m + 0x35B624]`
pub fn hit_size_and_dive(car: &Car, m: &DamageModel) {
    let t0 = ld(car.contact_depth.bits());
    let t1 = ld(car.contact_depth.bits());
    let t2 = t0 * t1;
    let t3 = ld(m.hit_divisor.bits());
    let t4 = t2 / t3;
    let t5 = ld(m.speed_range.bits());
    let t6 = t4 / t5;
    m.hit_size.set_bits(st(t6));
    let t7 = ld(car.local_velocity[1].bits());
    let t8 = -t7;
    let t9 = ld(m.gauge_rates[21].bits());
    let t10 = t8 - t9;
    m.scratch.set_bits(st(t10));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B36C] = ([m + 0x35B36C] * [m + 0x35B36C]) * [m + 0x35B620]`
pub fn square_dive(m: &DamageModel) {
    let t0 = ld(m.scratch.bits());
    let t1 = t0 * t0;
    let t2 = ld(m.gauge_rates[20].bits());
    let t3 = t1 * t2;
    m.scratch.set_bits(st(t3));
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
    let t0 = ld(m.wheel_loads[0].bits());
    let t1 = ld(m.wheel_loads[1].bits());
    let t2 = t0 + t1;
    let t3 = ld(m.wheel_loads[2].bits());
    let t4 = t2 + t3;
    let t5 = ld(m.wheel_loads[3].bits());
    let t6 = t4 + t5;
    let t7 = ld(m.quarter.bits());
    let t8 = t6 * t7;
    m.wheel_load_mean.set_bits(st(t8));
    let t9 = ld(m.wheel_load_mean.bits());
    let t10 = ld(m.half.bits());
    let t11 = t9 * t10;
    m.scratch.set_bits(st(t11));
    let t12 = ld(m.wheel_loads[0].bits());
    let t13 = ld(m.half.bits());
    let t14 = t12 * t13;
    let t15 = ld(m.scratch.bits());
    let t16 = t14 + t15;
    m.wheel_loads[0].set_bits(st(t16));
    let t17 = ld(m.wheel_loads[1].bits());
    let t18 = ld(m.half.bits());
    let t19 = t17 * t18;
    let t20 = ld(m.scratch.bits());
    let t21 = t19 + t20;
    m.wheel_loads[1].set_bits(st(t21));
    let t22 = ld(m.wheel_loads[2].bits());
    let t23 = ld(m.half.bits());
    let t24 = t22 * t23;
    let t25 = ld(m.scratch.bits());
    let t26 = t24 + t25;
    m.wheel_loads[2].set_bits(st(t26));
    let t27 = ld(m.wheel_loads[3].bits());
    let t28 = ld(m.half.bits());
    let t29 = t27 * t28;
    let t30 = ld(m.scratch.bits());
    let t31 = t29 + t30;
    m.wheel_loads[3].set_bits(st(t31));
}
