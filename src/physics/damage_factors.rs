//! `physics::damage_factors` (0x4AB5E6): the damage model's figures for this step
//! (`physics/damage_model.rs`).
//!
//! From the car block it works out, in `DAMAGE`: the ride-height, stiffness and brake
//! shares; per wheel the stiffness and ride height that follow the suspension damage (a
//! wheel at the top of the gauge is worn out); the engine's and the car's speed shares
//! against their limits; the switches `wheels_down`, `brake_share` and `flt_75B3C0` (the
//! engine past its limit); the copies of the gear, tyre kind and surface; the side hit
//! (`dword_75B3C4`, 1 .. 4) and its size; the four wheels' loads; and the hit time.
//!
//! Caller: `physics::damage_step`.

use crate::math::x87::mul;
use crate::math::x87_float::{X, compare, int, ld, st};
use crate::physics::damage_model::ONE;
use crate::physics::flow::{
    abs_below_or_unordered, below_or_unordered, equal_or_unordered, hold_high, hold_low, table_lerp,
};
use crate::physics::store::DamageModel;
use crate::race::owner_physics::Physics;
use crate::session::Cell32;

/// The damage model's figures for this step.
pub fn steps(physics: &Physics) {
    scale_car_figures(&physics.damage);
    // Per wheel: the stiffness and ride height follow the suspension damage; at the top
    // of the gauge the wheel is worn out.
    for wheel in 0..4 {
        let damage = &crate::race::owner::race().shared.car.suspension_damage[wheel];
        let stiffness = &crate::race::owner::race().shared.car.wheel_stiffness[wheel];
        let ride_height = &crate::race::owner::race().shared.car.wheel_ride_height[wheel];
        let lerp = &physics.damage.wheel_stiffness_lerp[wheel];
        table_lerp(
            damage,
            &physics.damage.lerp_tables[2],
            &physics.damage.gauge_low,
            lerp,
        );
        wheel_stiffness_from_share(&physics.damage, lerp, stiffness);
        hold_low(stiffness, &physics.damage.gauge_low);
        hold_high(stiffness, &physics.damage.gauge_top);
        wheel_ride_height_from_share(&physics.damage, ride_height);
        if equal_or_unordered(damage.bits(), physics.damage.gauge_top.bits()) {
            fail_wheel(&physics.damage, stiffness, ride_height);
        }
    }
    means_and_engine_speed(&physics.damage);
    hold_high(
        &physics.damage.scratch,
        &crate::race::owner::race().shared.car.engine_speed_limit,
    );
    engine_speed_share_step(&physics.damage);
    // The speed gaps are each held to `flt_75b378` before their share is taken.
    let speed_range = &physics.damage.flt_75b378;
    hold_high(&physics.damage.scratch, speed_range);
    speed_share_step(&physics.damage);
    hold_high(&physics.damage.scratch, speed_range);
    drive_a_slip_step(&physics.damage);
    hold_high(&physics.damage.scratch, speed_range);
    drive_extrapolation_step(
        &physics.damage,
        &physics.damage.drive_b_slip_share,
        &crate::race::owner::race().shared.car.drive_speed_a,
        &crate::race::owner::race().shared.car.drive_speed_a,
    );
    hold_high(&physics.damage.scratch, speed_range);
    drive_extrapolation_step(
        &physics.damage,
        &physics.damage.drive_a_extrapolated_share,
        &crate::race::owner::race().shared.car.drive_speed_b,
        &crate::race::owner::race().shared.car.drive_speed_b,
    );
    hold_high(&physics.damage.scratch, speed_range);
    finish_shares(&physics.damage);
    if crate::race::owner::race().shared.car.all_wheels_off.get() == 0 {
        mark_wheels_down(&physics.damage);
    }
    clear_cell(&physics.damage.brake_share);
    if !abs_below_or_unordered(
        crate::race::owner::race().shared.car.local_velocity[2].bits(),
        physics.damage.grade_high.bits(),
    ) && (crate::race::owner::race()
        .shared
        .car
        .brake
        .get()
        .wrapping_sub(0x10) as i32)
        >= 0
    {
        brake_share_step(&physics.damage);
    }
    // 1 when the engine's speed is above the limit times `flt_75B37C`.
    let engine_over_limit = engine_over_limit_status(&physics.damage) & 0x100 != 0;
    physics
        .damage
        .flt_75b3c0
        .set_bits(if engine_over_limit { ONE } else { 0 });
    physics
        .damage
        .dword_75b3b8
        .set_bits(crate::race::owner::race().shared.car.gear_changed.get());
    physics
        .damage
        .dword_75b3bc
        .set_bits(crate::race::owner::race().shared.car.gear.get());
    physics
        .damage
        .dword_75b3e8
        .set_bits(crate::race::owner::race().shared.car.tyre_kind.get());
    let surface = crate::race::owner::race().shared.car.wheel_surface_a[4].get();
    physics.damage.dword_75b3ec.set_bits(surface);
    clear_cell(&physics.damage.flt_75b3b4);
    if surface == 5 {
        physics.damage.flt_75b3b4.set_bits(ONE);
    }
    physics
        .damage
        .dword_75b3f0
        .set_bits(crate::race::owner::race().shared.car.surface_blend.bits());
    physics.damage.dword_75b3c4.set_bits(side_hit(physics));
    hit_size_and_dive(&physics.damage);
    hold_low(&physics.damage.scratch, &physics.damage.gauge_low);
    hold_high(&physics.damage.scratch, &physics.damage.flt_75b354);
    square_dive(&physics.damage);
    // A wheel that has just touched down carries the dive's load.
    for wheel in 0..4 {
        physics.damage.wheel_loads[wheel].set_bits(0);
        if crate::race::owner::race().shared.car.wheel_squeeze_last[wheel].bits() == 0
            && crate::race::owner::race().shared.car.wheel_squeeze[wheel].bits() != 0
        {
            physics.damage.wheel_loads[wheel].set_bits(mul(
                crate::race::owner::race().shared.car.wheel_squeeze[wheel].bits(),
                physics.damage.scratch.bits(),
            ));
        }
    }
    blend_wheel_loads(&physics.damage);
    if physics.damage.flt_75b3c8.bits() != 0
        && crate::race::owner::race().shared.car.hit_cooldown.get() != 0
    {
        physics.damage.flt_75b3c8.set_bits(0);
        crate::race::owner::race().shared.car.hit_cooldown.set(0x64);
    }
}

/// Where the car was hit: 0 for no contact, else 1 .. 4 by the side of the contact point.
fn side_hit(physics: &Physics) -> u32 {
    if crate::race::owner::race().shared.car.contact_depth.bits() == 0 {
        return 0;
    }
    let mut hit = 1;
    if !below_or_unordered(
        crate::race::owner::race().shared.car.contact_point[0].bits(),
        physics.damage.gauge_low.bits(),
    ) {
        hit += 1;
    }
    if below_or_unordered(
        crate::race::owner::race().shared.car.contact_point[2].bits(),
        physics.damage.gauge_low.bits(),
    ) {
        hit += 2;
    }
    hit
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3F8] = int [rc + 0x31A008] / [m + 0x35B35C]`
/// `[m + 0x35B3FC] = int [rc + 0x31A00C] / [m + 0x35B35C]`
/// `[m + 0x35B400] = int [rc + 0x319FF8] / [m + 0x35B35C]`
fn scale_car_figures(m: &DamageModel) {
    let t0 = int(crate::race::owner::race().shared.car.ride_height.bits() as i32);
    let t1 = ld(m.fixed_point_scale.bits());
    let t2 = t0 / t1;
    m.ride_height_share.set_bits(st(t2));
    let t3 = int(crate::race::owner::race()
        .shared
        .car
        .suspension_stiffness
        .bits() as i32);
    let t4 = ld(m.fixed_point_scale.bits());
    let t5 = t3 / t4;
    m.stiffness_share.set_bits(st(t5));
    let t6 = int(crate::race::owner::race().shared.car.brake_balance.bits() as i32);
    let t7 = ld(m.fixed_point_scale.bits());
    let t8 = t6 / t7;
    m.brake_balance_share.set_bits(st(t8));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o3] = ([m + 0x35B3FC] * [m + 0x35B34C]) - [m + o2]`
fn wheel_stiffness_from_share(m: &DamageModel, o2: &impl Cell32, o3: &impl crate::session::Cell32) {
    let t0 = ld(m.stiffness_share.bits());
    let t1 = ld(m.gauge_top.bits());
    let t2 = t0 * t1;
    let t3 = ld(o2.bits());
    let t4 = t2 - t3;
    o3.set_bits(st(t4));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o2] = [m + 0x35B3F8] * [m + 0x35B34C]`
fn wheel_ride_height_from_share(m: &DamageModel, o2: &impl crate::session::Cell32) {
    let t0 = ld(m.ride_height_share.bits());
    let t1 = ld(m.gauge_top.bits());
    let t2 = t0 * t1;
    o2.set_bits(st(t2));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o1] = [m + 0x35B34C]`
/// `[rc + o3] = [m + 0x35B350]`
fn fail_wheel(m: &DamageModel, o1: &impl crate::session::Cell32, o3: &impl crate::session::Cell32) {
    let t0 = ld(m.gauge_top.bits());
    o1.set_bits(st(t0));
    let t1 = ld(m.gauge_low.bits());
    o3.set_bits(st(t1));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A434] = ((([rc + 0x31A410] + [rc + 0x31A414]) + [rc + 0x31A418]) + [rc + 0x31A41C]) * [m + 0x35B364]`
/// `[rc + 0x31A430] = ((([rc + 0x31A420] + [rc + 0x31A424]) + [rc + 0x31A428]) + [rc + 0x31A42C]) * [m + 0x35B364]`
/// `[m + 0x35B3E4] = [rc + 0x31A018]`
/// `[m + 0x35B36C] = |[rc + 0x31A1B0]|`
fn means_and_engine_speed(m: &DamageModel) {
    let t0 = ld(crate::race::owner::race().shared.car.wheel_ride_height[0].bits());
    let t1 = ld(crate::race::owner::race().shared.car.wheel_ride_height[1].bits());
    let t2 = t0 + t1;
    let t3 = ld(crate::race::owner::race().shared.car.wheel_ride_height[2].bits());
    let t4 = t2 + t3;
    let t5 = ld(crate::race::owner::race().shared.car.wheel_ride_height[3].bits());
    let t6 = t4 + t5;
    let t7 = ld(m.flt_75b364.bits());
    let t8 = t6 * t7;
    crate::race::owner::race()
        .shared
        .car
        .ride_height_mean
        .set_bits(st(t8));
    let t9 = ld(crate::race::owner::race().shared.car.wheel_stiffness[0].bits());
    let t10 = ld(crate::race::owner::race().shared.car.wheel_stiffness[1].bits());
    let t11 = t9 + t10;
    let t12 = ld(crate::race::owner::race().shared.car.wheel_stiffness[2].bits());
    let t13 = t11 + t12;
    let t14 = ld(crate::race::owner::race().shared.car.wheel_stiffness[3].bits());
    let t15 = t13 + t14;
    let t16 = ld(m.flt_75b364.bits());
    let t17 = t15 * t16;
    crate::race::owner::race()
        .shared
        .car
        .stiffness_mean
        .set_bits(st(t17));
    let t18 = ld(crate::race::owner::race().shared.car.drive_kind.bits());
    m.flt_75b3e4.set_bits(st(t18));
    let t19 = ld(crate::race::owner::race().shared.car.engine_speed.bits());
    let t20 = t19.abs();
    m.scratch.set_bits(st(t20));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B398] = ([m + 0x35B36C] / [rc + 0x31A1B4]) * [m + 0x35B34C]`
/// `[m + 0x35B36C] = |[rc + 0x31A184]|`
fn engine_speed_share_step(m: &DamageModel) {
    let t0 = ld(m.scratch.bits());
    let t1 = ld(crate::race::owner::race()
        .shared
        .car
        .engine_speed_limit
        .bits());
    let t2 = t0 / t1;
    let t3 = ld(m.gauge_top.bits());
    let t4 = t2 * t3;
    m.engine_speed_share.set_bits(st(t4));
    let t5 = ld(crate::race::owner::race().shared.car.local_velocity[2].bits());
    let t6 = t5.abs();
    m.scratch.set_bits(st(t6));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B39C] = ([m + 0x35B36C] / [m + 0x35B378]) * [m + 0x35B34C]`
/// `[m + 0x35B3E0] = [rc + 0x31A930]`
/// `[m + 0x35B36C] = |([rc + 0x31A184] - [rc + 0x31A190])|`
fn speed_share_step(m: &DamageModel) {
    let t0 = ld(m.scratch.bits());
    let t1 = ld(m.flt_75b378.bits());
    let t2 = t0 / t1;
    let t3 = ld(m.gauge_top.bits());
    let t4 = t2 * t3;
    m.speed_share.set_bits(st(t4));
    let t5 = ld(crate::race::owner::race().shared.car.point_surface_2c[4].bits());
    m.flt_75b3e0.set_bits(st(t5));
    let t6 = ld(crate::race::owner::race().shared.car.local_velocity[2].bits());
    let t7 = ld(crate::race::owner::race().shared.car.drive_speed_a.bits());
    let t8 = t6 - t7;
    let t9 = t8.abs();
    m.scratch.set_bits(st(t9));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3A0] = [m + 0x35B36C] / [m + 0x35B378]`
/// `[m + 0x35B36C] = |([rc + 0x31A184] - [rc + 0x31A194])|`
fn drive_a_slip_step(m: &DamageModel) {
    let t0 = ld(m.scratch.bits());
    let t1 = ld(m.flt_75b378.bits());
    let t2 = t0 / t1;
    m.drive_a_slip_share.set_bits(st(t2));
    let t3 = ld(crate::race::owner::race().shared.car.local_velocity[2].bits());
    let t4 = ld(crate::race::owner::race().shared.car.drive_speed_b.bits());
    let t5 = t3 - t4;
    let t6 = t5.abs();
    m.scratch.set_bits(st(t6));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + o2] = [m + 0x35B36C] / [m + 0x35B378]`
/// `[m + 0x35B36C] = |((([rc + o3] - [rc + 0x31A184]) * [m + 0x35B6F4]) + [rc + o6])|`
fn drive_extrapolation_step(
    m: &DamageModel,
    o2: &impl Cell32,
    o3: &impl crate::session::Cell32,
    o6: &impl crate::session::Cell32,
) {
    let t0 = ld(m.scratch.bits());
    let t1 = ld(m.flt_75b378.bits());
    let t2 = t0 / t1;
    o2.set_bits(st(t2));
    let t3 = ld(o3.bits());
    let t4 = ld(crate::race::owner::race().shared.car.local_velocity[2].bits());
    let t5 = t3 - t4;
    let t6 = ld(m.figures[31].bits());
    let t7 = t5 * t6;
    let t8 = ld(o6.bits());
    let t9 = t7 + t8;
    let t10 = t9.abs();
    m.scratch.set_bits(st(t10));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3AC] = [m + 0x35B36C] / [m + 0x35B378]`
/// `[m + 0x35B3F4] = 0`
fn finish_shares(m: &DamageModel) {
    let t0 = ld(m.scratch.bits());
    let t1 = ld(m.flt_75b378.bits());
    let t2 = t0 / t1;
    m.drive_b_extrapolated_share.set_bits(st(t2));
    let t3 = X(0.0);
    m.wheels_down.set_bits(st(t3));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3F4] = 1`
fn mark_wheels_down(m: &DamageModel) {
    let t0 = X(1.0);
    m.wheels_down.set_bits(st(t0));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + o0] = 0`
fn clear_cell(o0: &impl Cell32) {
    let t0 = X(0.0);
    o0.set_bits(st(t0));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3B0] = int [rc + 0x31A614] / [m + 0x35B35C]`
fn brake_share_step(m: &DamageModel) {
    let t0 = int(crate::race::owner::race().shared.car.brake.bits() as i32);
    let t1 = ld(m.fixed_point_scale.bits());
    let t2 = t0 / t1;
    m.brake_share.set_bits(st(t2));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `status [s + 0] = status`
fn engine_over_limit_status(m: &DamageModel) -> u16 {
    let t0 = ld(crate::race::owner::race()
        .shared
        .car
        .engine_speed_limit
        .bits());
    let t1 = ld(m.flt_75b37c.bits());
    let t2 = t0 * t1;
    let t3 = ld(crate::race::owner::race().shared.car.engine_speed.bits());
    compare(t2, t3)
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3C8] = (([rc + 0x31A208] * [rc + 0x31A208]) / [m + 0x35B394]) / [m + 0x35B378]`
/// `[m + 0x35B36C] = -[rc + 0x31A180] - [m + 0x35B624]`
fn hit_size_and_dive(m: &DamageModel) {
    let t0 = ld(crate::race::owner::race().shared.car.contact_depth.bits());
    let t1 = ld(crate::race::owner::race().shared.car.contact_depth.bits());
    let t2 = t0 * t1;
    let t3 = ld(m.flt_75b394.bits());
    let t4 = t2 / t3;
    let t5 = ld(m.flt_75b378.bits());
    let t6 = t4 / t5;
    m.flt_75b3c8.set_bits(st(t6));
    let t7 = ld(crate::race::owner::race().shared.car.local_velocity[1].bits());
    let t8 = -t7;
    let t9 = ld(m.gauge_rates[21].bits());
    let t10 = t8 - t9;
    m.scratch.set_bits(st(t10));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B36C] = ([m + 0x35B36C] * [m + 0x35B36C]) * [m + 0x35B620]`
fn square_dive(m: &DamageModel) {
    let t0 = ld(m.scratch.bits());
    let t1 = t0 * t0;
    let t2 = ld(m.gauge_rates[20].bits());
    let t3 = t1 * t2;
    m.scratch.set_bits(st(t3));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3DC] = ((([m + 0x35B3CC] + [m + 0x35B3D0]) + [m + 0x35B3D4]) + [m + 0x35B3D8]) * [m + 0x35B364]`
/// `[m + 0x35B36C] = [m + 0x35B3DC] * [m + 0x35B360]`
/// `[m + 0x35B3CC] = ([m + 0x35B3CC] * [m + 0x35B360]) + [m + 0x35B36C]`
/// `[m + 0x35B3D0] = ([m + 0x35B3D0] * [m + 0x35B360]) + [m + 0x35B36C]`
/// `[m + 0x35B3D4] = ([m + 0x35B3D4] * [m + 0x35B360]) + [m + 0x35B36C]`
/// `[m + 0x35B3D8] = ([m + 0x35B3D8] * [m + 0x35B360]) + [m + 0x35B36C]`
fn blend_wheel_loads(m: &DamageModel) {
    let t0 = ld(m.wheel_loads[0].bits());
    let t1 = ld(m.wheel_loads[1].bits());
    let t2 = t0 + t1;
    let t3 = ld(m.wheel_loads[2].bits());
    let t4 = t2 + t3;
    let t5 = ld(m.wheel_loads[3].bits());
    let t6 = t4 + t5;
    let t7 = ld(m.flt_75b364.bits());
    let t8 = t6 * t7;
    m.flt_75b3dc.set_bits(st(t8));
    let t9 = ld(m.flt_75b3dc.bits());
    let t10 = ld(m.flt_75b360.bits());
    let t11 = t9 * t10;
    m.scratch.set_bits(st(t11));
    let t12 = ld(m.wheel_loads[0].bits());
    let t13 = ld(m.flt_75b360.bits());
    let t14 = t12 * t13;
    let t15 = ld(m.scratch.bits());
    let t16 = t14 + t15;
    m.wheel_loads[0].set_bits(st(t16));
    let t17 = ld(m.wheel_loads[1].bits());
    let t18 = ld(m.flt_75b360.bits());
    let t19 = t17 * t18;
    let t20 = ld(m.scratch.bits());
    let t21 = t19 + t20;
    m.wheel_loads[1].set_bits(st(t21));
    let t22 = ld(m.wheel_loads[2].bits());
    let t23 = ld(m.flt_75b360.bits());
    let t24 = t22 * t23;
    let t25 = ld(m.scratch.bits());
    let t26 = t24 + t25;
    m.wheel_loads[2].set_bits(st(t26));
    let t27 = ld(m.wheel_loads[3].bits());
    let t28 = ld(m.flt_75b360.bits());
    let t29 = t27 * t28;
    let t30 = ld(m.scratch.bits());
    let t31 = t29 + t30;
    m.wheel_loads[3].set_bits(st(t31));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;

    /// The damage model as the test's buffer holds it (the buffer's base plus the RVA of
    /// 0x75B340).
    fn dm(base: usize) -> &'static DamageModel {
        unsafe { &*(crate::pointers::host_at(base + 0x35B340) as *const DamageModel) }
    }
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};
    use crate::scene::x87_mem::Base;

    // The original's instructions, one function per run.
    reference!(ref_0(rc, m) "fild dword ptr [{rc} + 0x31A008]", "fdiv dword ptr [{m} + 0x35B35C]", "fstp dword ptr [{m} + 0x35B3F8]", "fild dword ptr [{rc} + 0x31A00C]", "fdiv dword ptr [{m} + 0x35B35C]", "fstp dword ptr [{m} + 0x35B3FC]", "fild dword ptr [{rc} + 0x319FF8]", "fdiv dword ptr [{m} + 0x35B35C]", "fstp dword ptr [{m} + 0x35B400]");
    reference!(ref_1(m, rc) "fld dword ptr [{m} + 0x35B3FC]", "fmul dword ptr [{m} + 0x35B34C]", "fsub dword ptr [{m} + 0x35B474]", "fstp dword ptr [{rc} + 0x31A420]");
    reference!(ref_2(m, rc) "fld dword ptr [{m} + 0x35B3F8]", "fmul dword ptr [{m} + 0x35B34C]", "fstp dword ptr [{rc} + 0x31A410]");
    reference!(ref_3(m, rc) "fld dword ptr [{m} + 0x35B34C]", "fstp dword ptr [{rc} + 0x31A420]", "fld dword ptr [{m} + 0x35B350]", "fstp dword ptr [{rc} + 0x31A410]");
    reference!(ref_4(m, rc) "fld dword ptr [{m} + 0x35B3FC]", "fmul dword ptr [{m} + 0x35B34C]", "fsub dword ptr [{m} + 0x35B478]", "fstp dword ptr [{rc} + 0x31A424]");
    reference!(ref_5(m, rc) "fld dword ptr [{m} + 0x35B3F8]", "fmul dword ptr [{m} + 0x35B34C]", "fstp dword ptr [{rc} + 0x31A414]");
    reference!(ref_6(m, rc) "fld dword ptr [{m} + 0x35B34C]", "fstp dword ptr [{rc} + 0x31A424]", "fld dword ptr [{m} + 0x35B350]", "fstp dword ptr [{rc} + 0x31A414]");
    reference!(ref_7(m, rc) "fld dword ptr [{m} + 0x35B3FC]", "fmul dword ptr [{m} + 0x35B34C]", "fsub dword ptr [{m} + 0x35B47C]", "fstp dword ptr [{rc} + 0x31A428]");
    reference!(ref_8(m, rc) "fld dword ptr [{m} + 0x35B3F8]", "fmul dword ptr [{m} + 0x35B34C]", "fstp dword ptr [{rc} + 0x31A418]");
    reference!(ref_9(m, rc) "fld dword ptr [{m} + 0x35B34C]", "fstp dword ptr [{rc} + 0x31A428]", "fld dword ptr [{m} + 0x35B350]", "fstp dword ptr [{rc} + 0x31A418]");
    reference!(ref_10(m, rc) "fld dword ptr [{m} + 0x35B3FC]", "fmul dword ptr [{m} + 0x35B34C]", "fsub dword ptr [{m} + 0x35B480]", "fstp dword ptr [{rc} + 0x31A42C]");
    reference!(ref_11(m, rc) "fld dword ptr [{m} + 0x35B3F8]", "fmul dword ptr [{m} + 0x35B34C]", "fstp dword ptr [{rc} + 0x31A41C]");
    reference!(ref_12(m, rc) "fld dword ptr [{m} + 0x35B34C]", "fstp dword ptr [{rc} + 0x31A42C]", "fld dword ptr [{m} + 0x35B350]", "fstp dword ptr [{rc} + 0x31A41C]");
    reference!(ref_13(rc, m) "fld dword ptr [{rc} + 0x31A410]", "fadd dword ptr [{rc} + 0x31A414]", "fadd dword ptr [{rc} + 0x31A418]", "fadd dword ptr [{rc} + 0x31A41C]", "fmul dword ptr [{m} + 0x35B364]", "fstp dword ptr [{rc} + 0x31A434]", "fld dword ptr [{rc} + 0x31A420]", "fadd dword ptr [{rc} + 0x31A424]", "fadd dword ptr [{rc} + 0x31A428]", "fadd dword ptr [{rc} + 0x31A42C]", "fmul dword ptr [{m} + 0x35B364]", "fstp dword ptr [{rc} + 0x31A430]", "fld dword ptr [{rc} + 0x31A018]", "fstp dword ptr [{m} + 0x35B3E4]", "fld dword ptr [{rc} + 0x31A1B0]", ".byte 0xD9, 0xE1", "fstp dword ptr [{m} + 0x35B36C]");
    reference!(ref_14(m, rc) "fld dword ptr [{m} + 0x35B36C]", "fdiv dword ptr [{rc} + 0x31A1B4]", "fmul dword ptr [{m} + 0x35B34C]", "fstp dword ptr [{m} + 0x35B398]", "fld dword ptr [{rc} + 0x31A184]", ".byte 0xD9, 0xE1", "fstp dword ptr [{m} + 0x35B36C]");
    reference!(ref_15(m, rc) "fld dword ptr [{m} + 0x35B36C]", "fdiv dword ptr [{m} + 0x35B378]", "fmul dword ptr [{m} + 0x35B34C]", "fstp dword ptr [{m} + 0x35B39C]", "fld dword ptr [{rc} + 0x31A930]", "fstp dword ptr [{m} + 0x35B3E0]", "fld dword ptr [{rc} + 0x31A184]", "fsub dword ptr [{rc} + 0x31A190]", ".byte 0xD9, 0xE1", "fstp dword ptr [{m} + 0x35B36C]");
    reference!(ref_16(m, rc) "fld dword ptr [{m} + 0x35B36C]", "fdiv dword ptr [{m} + 0x35B378]", "fstp dword ptr [{m} + 0x35B3A0]", "fld dword ptr [{rc} + 0x31A184]", "fsub dword ptr [{rc} + 0x31A194]", ".byte 0xD9, 0xE1", "fstp dword ptr [{m} + 0x35B36C]");
    reference!(ref_17(m, rc) "fld dword ptr [{m} + 0x35B36C]", "fdiv dword ptr [{m} + 0x35B378]", "fstp dword ptr [{m} + 0x35B3A4]", "fld dword ptr [{rc} + 0x31A190]", "fsub dword ptr [{rc} + 0x31A184]", "fmul dword ptr [{m} + 0x35B6F4]", "fadd dword ptr [{rc} + 0x31A190]", ".byte 0xD9, 0xE1", "fstp dword ptr [{m} + 0x35B36C]");
    reference!(ref_18(m, rc) "fld dword ptr [{m} + 0x35B36C]", "fdiv dword ptr [{m} + 0x35B378]", "fstp dword ptr [{m} + 0x35B3A8]", "fld dword ptr [{rc} + 0x31A194]", "fsub dword ptr [{rc} + 0x31A184]", "fmul dword ptr [{m} + 0x35B6F4]", "fadd dword ptr [{rc} + 0x31A194]", ".byte 0xD9, 0xE1", "fstp dword ptr [{m} + 0x35B36C]");
    reference!(ref_19(m) "fld dword ptr [{m} + 0x35B36C]", "fdiv dword ptr [{m} + 0x35B378]", "fstp dword ptr [{m} + 0x35B3AC]", ".byte 0xD9, 0xEE", "fstp dword ptr [{m} + 0x35B3F4]");
    reference!(ref_20(m) ".byte 0xD9, 0xE8", "fstp dword ptr [{m} + 0x35B3F4]");
    reference!(ref_21(m) ".byte 0xD9, 0xEE", "fstp dword ptr [{m} + 0x35B3B0]");
    reference!(ref_22(rc, m) "fild dword ptr [{rc} + 0x31A614]", "fdiv dword ptr [{m} + 0x35B35C]", "fstp dword ptr [{m} + 0x35B3B0]");
    reference!(ref_23(rc, m, s) "fld dword ptr [{rc} + 0x31A1B4]", "fmul dword ptr [{m} + 0x35B37C]", "fcomp dword ptr [{rc} + 0x31A1B0]", "fnstsw word ptr [{s}]");
    reference!(ref_24(m) ".byte 0xD9, 0xEE", "fstp dword ptr [{m} + 0x35B3B4]");
    reference!(ref_25(rc, m) "fld dword ptr [{rc} + 0x31A208]", "fmul dword ptr [{rc} + 0x31A208]", "fdiv dword ptr [{m} + 0x35B394]", "fdiv dword ptr [{m} + 0x35B378]", "fstp dword ptr [{m} + 0x35B3C8]", "fld dword ptr [{rc} + 0x31A180]", ".byte 0xD9, 0xE0", "fsub dword ptr [{m} + 0x35B624]", "fstp dword ptr [{m} + 0x35B36C]");
    reference!(ref_26(m) "fld dword ptr [{m} + 0x35B36C]", ".byte 0xD8, 0xC8", "fmul dword ptr [{m} + 0x35B620]", "fstp dword ptr [{m} + 0x35B36C]");
    reference!(ref_27(m) "fld dword ptr [{m} + 0x35B3CC]", "fadd dword ptr [{m} + 0x35B3D0]", "fadd dword ptr [{m} + 0x35B3D4]", "fadd dword ptr [{m} + 0x35B3D8]", "fmul dword ptr [{m} + 0x35B364]", "fstp dword ptr [{m} + 0x35B3DC]", "fld dword ptr [{m} + 0x35B3DC]", "fmul dword ptr [{m} + 0x35B360]", "fstp dword ptr [{m} + 0x35B36C]", "fld dword ptr [{m} + 0x35B3CC]", "fmul dword ptr [{m} + 0x35B360]", "fadd dword ptr [{m} + 0x35B36C]", "fstp dword ptr [{m} + 0x35B3CC]", "fld dword ptr [{m} + 0x35B3D0]", "fmul dword ptr [{m} + 0x35B360]", "fadd dword ptr [{m} + 0x35B36C]", "fstp dword ptr [{m} + 0x35B3D0]", "fld dword ptr [{m} + 0x35B3D4]", "fmul dword ptr [{m} + 0x35B360]", "fadd dword ptr [{m} + 0x35B36C]", "fstp dword ptr [{m} + 0x35B3D4]", "fld dword ptr [{m} + 0x35B3D8]", "fmul dword ptr [{m} + 0x35B360]", "fadd dword ptr [{m} + 0x35B36C]", "fstp dword ptr [{m} + 0x35B3D8]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "damage_factors line 39",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[(0x319FF8, In::I), (0x31A008, In::I), (0x31A00C, In::I)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x35B35C, In::F)],
                        writes: &[(0x35B3F8, Out::F), (0x35B3FC, Out::F), (0x35B400, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1]) },
                plain: |a| scale_car_figures(dm(a[1])),
            },
            Block {
                name: "damage_factors line 53",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B3FC, In::F), (0x35B474, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A420, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1]) },
                plain: |a| {
                    wheel_stiffness_from_share(
                        dm(a[0]),
                        &dm(a[0]).wheel_stiffness_lerp[0],
                        &crate::race::owner::race().shared.car.wheel_stiffness[0],
                    );
                },
            },
            Block {
                name: "damage_factors line 63",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B3F8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A410, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0], a[1]) },
                plain: |a| {
                    wheel_ride_height_from_share(
                        dm(a[0]),
                        &crate::race::owner::race().shared.car.wheel_ride_height[0],
                    );
                },
            },
            Block {
                name: "damage_factors line 71",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B350, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A410, Out::F), (0x31A420, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_3(a[0], a[1]) },
                plain: |a| {
                    fail_wheel(
                        dm(a[0]),
                        &crate::race::owner::race().shared.car.wheel_stiffness[0],
                        &crate::race::owner::race().shared.car.wheel_ride_height[0],
                    );
                },
            },
            Block {
                name: "damage_factors line 81",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B3FC, In::F), (0x35B478, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A424, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_4(a[0], a[1]) },
                plain: |a| {
                    wheel_stiffness_from_share(
                        dm(a[0]),
                        &dm(a[0]).wheel_stiffness_lerp[1],
                        &crate::race::owner::race().shared.car.wheel_stiffness[1],
                    );
                },
            },
            Block {
                name: "damage_factors line 91",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B3F8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A414, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_5(a[0], a[1]) },
                plain: |a| {
                    wheel_ride_height_from_share(
                        dm(a[0]),
                        &crate::race::owner::race().shared.car.wheel_ride_height[1],
                    );
                },
            },
            Block {
                name: "damage_factors line 99",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B350, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A414, Out::F), (0x31A424, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_6(a[0], a[1]) },
                plain: |a| {
                    fail_wheel(
                        dm(a[0]),
                        &crate::race::owner::race().shared.car.wheel_stiffness[1],
                        &crate::race::owner::race().shared.car.wheel_ride_height[1],
                    );
                },
            },
            Block {
                name: "damage_factors line 109",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B3FC, In::F), (0x35B47C, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A428, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_7(a[0], a[1]) },
                plain: |a| {
                    wheel_stiffness_from_share(
                        dm(a[0]),
                        &dm(a[0]).wheel_stiffness_lerp[2],
                        &crate::race::owner::race().shared.car.wheel_stiffness[2],
                    );
                },
            },
            Block {
                name: "damage_factors line 119",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B3F8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A418, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_8(a[0], a[1]) },
                plain: |a| {
                    wheel_ride_height_from_share(
                        dm(a[0]),
                        &crate::race::owner::race().shared.car.wheel_ride_height[2],
                    );
                },
            },
            Block {
                name: "damage_factors line 127",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B350, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A418, Out::F), (0x31A428, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_9(a[0], a[1]) },
                plain: |a| {
                    fail_wheel(
                        dm(a[0]),
                        &crate::race::owner::race().shared.car.wheel_stiffness[2],
                        &crate::race::owner::race().shared.car.wheel_ride_height[2],
                    );
                },
            },
            Block {
                name: "damage_factors line 137",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B3FC, In::F), (0x35B480, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A42C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_10(a[0], a[1]) },
                plain: |a| {
                    wheel_stiffness_from_share(
                        dm(a[0]),
                        &dm(a[0]).wheel_stiffness_lerp[3],
                        &crate::race::owner::race().shared.car.wheel_stiffness[3],
                    );
                },
            },
            Block {
                name: "damage_factors line 147",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B3F8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A41C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_11(a[0], a[1]) },
                plain: |a| {
                    wheel_ride_height_from_share(
                        dm(a[0]),
                        &crate::race::owner::race().shared.car.wheel_ride_height[3],
                    );
                },
            },
            Block {
                name: "damage_factors line 155",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B350, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A41C, Out::F), (0x31A42C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_12(a[0], a[1]) },
                plain: |a| {
                    fail_wheel(
                        dm(a[0]),
                        &crate::race::owner::race().shared.car.wheel_stiffness[3],
                        &crate::race::owner::race().shared.car.wheel_ride_height[3],
                    );
                },
            },
            Block {
                name: "damage_factors line 164",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[
                            (0x31A018, In::F),
                            (0x31A1B0, In::F),
                            (0x31A410, In::F),
                            (0x31A414, In::F),
                            (0x31A418, In::F),
                            (0x31A41C, In::F),
                            (0x31A420, In::F),
                            (0x31A424, In::F),
                            (0x31A428, In::F),
                            (0x31A42C, In::F),
                        ],
                        writes: &[(0x31A430, Out::F), (0x31A434, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0x35B364, In::F)],
                        writes: &[(0x35B36C, Out::F), (0x35B3E4, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_13(a[0], a[1]) },
                plain: |a| means_and_engine_speed(dm(a[1])),
            },
            Block {
                name: "damage_factors line 186",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B36C, In::F)],
                        writes: &[(0x35B36C, Out::F), (0x35B398, Out::F)],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A184, In::F), (0x31A1B4, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_14(a[0], a[1]) },
                plain: |a| engine_speed_share_step(dm(a[0])),
            },
            Block {
                name: "damage_factors line 198",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B36C, In::F), (0x35B378, In::F)],
                        writes: &[(0x35B36C, Out::F), (0x35B39C, Out::F), (0x35B3E0, Out::F)],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A184, In::F), (0x31A190, In::F), (0x31A930, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_15(a[0], a[1]) },
                plain: |a| speed_share_step(dm(a[0])),
            },
            Block {
                name: "damage_factors line 213",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B36C, In::F), (0x35B378, In::F)],
                        writes: &[(0x35B36C, Out::F), (0x35B3A0, Out::F)],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A184, In::F), (0x31A194, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_16(a[0], a[1]) },
                plain: |a| drive_a_slip_step(dm(a[0])),
            },
            Block {
                name: "damage_factors line 225",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B36C, In::F), (0x35B378, In::F), (0x35B6F4, In::F)],
                        writes: &[(0x35B36C, Out::F), (0x35B3A4, Out::F)],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A184, In::F), (0x31A190, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_17(a[0], a[1]) },
                plain: |a| {
                    drive_extrapolation_step(
                        dm(a[0]),
                        &dm(a[0]).drive_b_slip_share,
                        &crate::race::owner::race().shared.car.drive_speed_a,
                        &crate::race::owner::race().shared.car.drive_speed_a,
                    );
                },
            },
            Block {
                name: "damage_factors line 239",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B36C, In::F), (0x35B378, In::F), (0x35B6F4, In::F)],
                        writes: &[(0x35B36C, Out::F), (0x35B3A8, Out::F)],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A184, In::F), (0x31A194, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_18(a[0], a[1]) },
                plain: |a| {
                    drive_extrapolation_step(
                        dm(a[0]),
                        &dm(a[0]).drive_a_extrapolated_share,
                        &crate::race::owner::race().shared.car.drive_speed_b,
                        &crate::race::owner::race().shared.car.drive_speed_b,
                    );
                },
            },
            Block {
                name: "damage_factors line 253",
                operands: vec![Operand::Buffer {
                    reads: &[(0x35B36C, In::F), (0x35B378, In::F)],
                    writes: &[(0x35B3AC, Out::F), (0x35B3F4, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_19(a[0]) },
                plain: |a| finish_shares(dm(a[0])),
            },
            Block {
                name: "damage_factors line 262",
                operands: vec![Operand::Buffer {
                    reads: &[],
                    writes: &[(0x35B3F4, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_20(a[0]) },
                plain: |a| mark_wheels_down(dm(a[0])),
            },
            Block {
                name: "damage_factors line 268",
                operands: vec![Operand::Buffer {
                    reads: &[],
                    writes: &[(0x35B3B0, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_21(a[0]) },
                plain: |a| clear_cell(&dm(a[0]).brake_share),
            },
            Block {
                name: "damage_factors line 276",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[(0x31A614, In::I)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x35B35C, In::F)],
                        writes: &[(0x35B3B0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_22(a[0], a[1]) },
                plain: |a| brake_share_step(dm(a[1])),
            },
            Block {
                name: "damage_factors line 287",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[(0x31A1B0, In::F), (0x31A1B4, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x35B37C, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::H)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_23(a[0], a[1], a[2]) },
                plain: |a| a[2].set_word(0, engine_over_limit_status(dm(a[1]))),
            },
            Block {
                name: "damage_factors line 302",
                operands: vec![Operand::Buffer {
                    reads: &[],
                    writes: &[(0x35B3B4, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_24(a[0]) },
                plain: |a| clear_cell(&dm(a[0]).flt_75b3b4),
            },
            Block {
                name: "damage_factors line 322",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[(0x31A180, In::F), (0x31A208, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x35B378, In::F), (0x35B394, In::F), (0x35B624, In::F)],
                        writes: &[(0x35B36C, Out::F), (0x35B3C8, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_25(a[0], a[1]) },
                plain: |a| hit_size_and_dive(dm(a[1])),
            },
            Block {
                name: "damage_factors line 337",
                operands: vec![Operand::Buffer {
                    reads: &[(0x35B36C, In::F), (0x35B620, In::F)],
                    writes: &[(0x35B36C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_26(a[0]) },
                plain: |a| square_dive(dm(a[0])),
            },
            Block {
                name: "damage_factors line 350",
                operands: vec![Operand::Buffer {
                    reads: &[
                        (0x35B360, In::F),
                        (0x35B364, In::F),
                        (0x35B36C, In::F),
                        (0x35B3CC, In::F),
                        (0x35B3D0, In::F),
                        (0x35B3D4, In::F),
                        (0x35B3D8, In::F),
                        (0x35B3DC, In::F),
                    ],
                    writes: &[
                        (0x35B36C, Out::F),
                        (0x35B3CC, Out::F),
                        (0x35B3D0, Out::F),
                        (0x35B3D4, Out::F),
                        (0x35B3D8, Out::F),
                        (0x35B3DC, Out::F),
                    ],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_27(a[0]) },
                plain: |a| blend_wheel_loads(dm(a[0])),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
