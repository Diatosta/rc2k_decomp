//! The damage model's figures for this step (`physics/damage_model.rs`).
//!
//! From the car block it works out, in the damage model: the ride-height, stiffness and brake
//! shares; per wheel the stiffness and ride height that follow the suspension damage (a wheel
//! at the top of the gauge is worn out); the engine's and the car's speed shares against their
//! limits;
//! the switches `wheels_down`, `brake_share` and `DamageModel.engine_over_limit` (the engine
//! past its limit); the copies of the gear, tyre kind and surface; the side hit
//! (`DamageModel.hit_side`, 1 .. 4) and its size; the four wheels' loads; and the hit time.

use crate::memory::session::{FloatLike, PlainFloat};
use crate::physics::car_store::Car;
use crate::physics::damage_model::{
    figure, figure_is_set, held_high, held_low, is_set, lerp_table, store, value, whole,
};
use crate::physics::flow::{
    abs_below_or_unordered, below_or_unordered, equal_or_unordered, hold_high_to, hold_low_to,
};
use crate::physics::store::DamageModel;
use core::cmp::Ordering;

/// The damage model's figures for this step.
pub fn steps(car: &Car, damage: &mut DamageModel) {
    scale_car_figures(car, damage);
    // Per wheel: the stiffness and ride height follow the suspension damage; at the top
    // of the gauge the wheel is worn out.
    for wheel in 0..4 {
        let suspension = &car.suspension_damage.float_at(wheel);
        let stiffness = &car.wheel_stiffness.float_at(wheel);
        let ride_height = &car.wheel_ride_height.float_at(wheel);
        let lerp = lerp_table(suspension.get(), &damage.lerp_tables[2], damage.gauge_low);
        damage.wheel_stiffness_lerp[wheel] = lerp;
        wheel_stiffness_from_share(damage, lerp, stiffness);
        hold_low_to(stiffness, damage.gauge_low);
        hold_high_to(stiffness, damage.gauge_top);
        wheel_ride_height_from_share(damage, ride_height);
        if equal_or_unordered(suspension, damage.gauge_top) {
            fail_wheel(damage, stiffness, ride_height);
        }
    }
    means_and_engine_speed(car, damage);
    held_high(&mut damage.scratch, car.engine_speed_limit.get());
    engine_speed_share_step(car, damage);
    // The speed gaps are each held to `speed_range` before their share is taken.
    held_high(&mut damage.scratch, damage.speed_range);
    speed_share_step(car, damage);
    held_high(&mut damage.scratch, damage.speed_range);
    drive_a_slip_step(car, damage);
    held_high(&mut damage.scratch, damage.speed_range);
    damage.drive_b_slip_share =
        drive_extrapolation_step(car, damage, &car.drive_speed_a, &car.drive_speed_a);
    held_high(&mut damage.scratch, damage.speed_range);
    damage.drive_a_extrapolated_share =
        drive_extrapolation_step(car, damage, &car.drive_speed_b, &car.drive_speed_b);
    held_high(&mut damage.scratch, damage.speed_range);
    finish_shares(damage);
    if car.all_wheels_off.get() == 0 {
        damage.wheels_down = 1.0;
    }
    damage.brake_share = 0.0;
    if !abs_below_or_unordered(car.local_velocity.bits_at(2), damage.grade_high)
        && (car.brake.get().wrapping_sub(0x10) as i32) >= 0
    {
        brake_share_step(car, damage);
    }
    // 1 when the engine's speed is above the limit times `DamageModel.engine_limit_factor`.
    damage.engine_over_limit = if engine_over_limit(car, damage) {
        1.0
    } else {
        0.0
    };
    damage.gear_changed = car.gear_changed.get();
    damage.gear = car.gear.get();
    damage.tyre_kind = car.tyre_kind.get();
    let surface = car.wheel_surface_a[4].get();
    damage.surface = surface;
    damage.on_surface_5 = 0.0;
    if surface == 5 {
        damage.on_surface_5 = 1.0;
    }
    damage.surface_blend = car.surface_blend.get();
    damage.hit_side = side_hit(car, damage);
    hit_size_and_dive(car, damage);
    held_low(&mut damage.scratch, damage.gauge_low);
    held_high(&mut damage.scratch, damage.dive_top);
    square_dive(damage);
    // A wheel that has just touched down carries the dive's load.
    for wheel in 0..4 {
        damage.wheel_loads[wheel] = 0.0;
        // Word tests: a squeeze of -0.0 counts as touching.
        if !is_set(&car.wheel_squeeze_last[wheel]) && is_set(&car.wheel_squeeze[wheel]) {
            damage.wheel_loads[wheel] =
                (value(&car.wheel_squeeze[wheel]) * figure(damage.scratch)) as f32;
        }
    }
    blend_wheel_loads(damage);
    if figure_is_set(damage.hit_size) && car.hit_cooldown.get() != 0 {
        damage.hit_size = 0.0;
        car.hit_cooldown.set(0x64);
    }
}

/// Where the car was hit: 0 for no contact, else 1 .. 4 by the side of the contact point.
fn side_hit(car: &Car, damage: &DamageModel) -> u32 {
    if !is_set(&car.contact_depth) {
        return 0;
    }
    let mut hit = 1;
    if !below_or_unordered(car.contact_point.bits_at(0), damage.gauge_low) {
        hit += 1;
    }
    if below_or_unordered(car.contact_point.bits_at(2), damage.gauge_low) {
        hit += 2;
    }
    hit
}

// ---- the arithmetic ----

/// The ride height, stiffness and brake balance of the car, which it holds as whole numbers
/// over 256, as shares.
fn scale_car_figures(car: &Car, m: &mut DamageModel) {
    let scale = figure(m.fixed_point_scale);
    m.ride_height_share = (whole(&car.ride_height) / scale) as f32;
    m.stiffness_share = (whole(&car.suspension_stiffness) / scale) as f32;
    m.brake_balance_share = (whole(&car.brake_balance) / scale) as f32;
}

/// A wheel's stiffness: the stiffness share times the top of the gauge, less the lerp of its
/// suspension damage.
fn wheel_stiffness_from_share(m: &DamageModel, lerp: f32, stiffness: &impl FloatLike) {
    store(
        stiffness,
        figure(m.stiffness_share) * figure(m.gauge_top) - figure(lerp),
    );
}

/// A wheel's ride height: the ride height share times the top of the gauge.
fn wheel_ride_height_from_share(m: &DamageModel, ride_height: &impl FloatLike) {
    store(
        ride_height,
        figure(m.ride_height_share) * figure(m.gauge_top),
    );
}

/// A worn-out wheel: full stiffness, no ride height.
fn fail_wheel(m: &DamageModel, stiffness: &impl FloatLike, ride_height: &impl FloatLike) {
    store(stiffness, figure(m.gauge_top));
    store(ride_height, figure(m.gauge_low));
}

/// The means of the wheels' ride heights and stiffnesses, the drive kind, and the engine's
/// speed made positive (into `scratch`).
fn means_and_engine_speed(car: &Car, m: &mut DamageModel) {
    let quarter = figure(m.quarter);
    let [h0, h1, h2, h3] = &car.wheel_ride_height.floats();
    store(
        &car.ride_height_mean,
        (value(h0) + value(h1) + value(h2) + value(h3)) * quarter,
    );
    let [s0, s1, s2, s3] = &car.wheel_stiffness.floats();
    store(
        &car.stiffness_mean,
        (value(s0) + value(s1) + value(s2) + value(s3)) * quarter,
    );
    // The car's whole number passes through the x87 as a float, which gives back the same bits.
    m.drive_kind = car.drive_kind.get();
    m.scratch = value(&car.engine_speed).abs() as f32;
}

/// The engine's speed (in `scratch`) over its limit as a share of the gauge, then the
/// forward speed made positive.
fn engine_speed_share_step(car: &Car, m: &mut DamageModel) {
    m.engine_speed_share =
        (figure(m.scratch) / value(&car.engine_speed_limit) * figure(m.gauge_top)) as f32;
    m.scratch = f64::from(car.local_velocity.at(2)).abs() as f32;
}

/// The forward speed (in `scratch`) over the speed range as a share of the gauge, the surface
/// wear factor copied, then the gap between the forward speed and the first drive speed.
fn speed_share_step(car: &Car, m: &mut DamageModel) {
    m.speed_share = (figure(m.scratch) / figure(m.speed_range) * figure(m.gauge_top)) as f32;
    m.point_surface_copy = value(&car.point_wear_factor[4]) as f32;
    m.scratch = (f64::from(car.local_velocity.at(2)) - value(&car.drive_speed_a)).abs() as f32;
}

/// The first drive's slip share, then the gap between the forward speed and the second
/// drive speed.
fn drive_a_slip_step(car: &Car, m: &mut DamageModel) {
    m.drive_a_slip_share = (figure(m.scratch) / figure(m.speed_range)) as f32;
    m.scratch = (f64::from(car.local_velocity.at(2)) - value(&car.drive_speed_b)).abs() as f32;
}

/// A drive's share (returned), then the next gap (into `scratch`): the drive speed's distance
/// from the forward speed, carried on by the extrapolation figure.
fn drive_extrapolation_step(
    car: &Car,
    m: &mut DamageModel,
    drive_speed: &PlainFloat,
    carried: &PlainFloat,
) -> f32 {
    let share = (figure(m.scratch) / figure(m.speed_range)) as f32;
    m.scratch = ((value(drive_speed) - f64::from(car.local_velocity.at(2))) * figure(m.figures[31])
        + value(carried))
    .abs() as f32;
    share
}

/// The last share, and no wheel down yet.
fn finish_shares(m: &mut DamageModel) {
    m.drive_b_extrapolated_share = (figure(m.scratch) / figure(m.speed_range)) as f32;
    m.wheels_down = 0.0;
}

/// The brake pedal, a whole number over 256, as a share.
fn brake_share_step(car: &Car, m: &mut DamageModel) {
    m.brake_share = (whole(&car.brake) / figure(m.fixed_point_scale)) as f32;
}

/// Whether the engine's speed is above its limit times `engine_limit_factor` (also when
/// either is no number).
fn engine_over_limit(car: &Car, m: &DamageModel) -> bool {
    let limit = value(&car.engine_speed_limit) * figure(m.engine_limit_factor);
    !matches!(
        limit.partial_cmp(&value(&car.engine_speed)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// The size of the hit: the contact depth squared over the divisor over the speed range;
/// and, into `scratch`, the vertical speed's fall less the dive rate.
fn hit_size_and_dive(car: &Car, m: &mut DamageModel) {
    let depth = value(&car.contact_depth);
    m.hit_size = (depth * depth / figure(m.hit_divisor) / figure(m.speed_range)) as f32;
    m.scratch = (-f64::from(car.local_velocity.at(1)) - figure(m.gauge_rates[21])) as f32;
}

/// The dive squared times its rate.
fn square_dive(m: &mut DamageModel) {
    let dive = figure(m.scratch);
    m.scratch = (dive * dive * figure(m.gauge_rates[20])) as f32;
}

/// The wheels' loads: their mean, and each load halved with half the mean added.
fn blend_wheel_loads(m: &mut DamageModel) {
    // No wheel just landed (the usual step): the four loads are +0.0, so the sum, the mean
    // (`quarter` 0.25) and the half-mean (`half` 0.5, both constants of the model) are +0.0 in
    // every rounding mode, and each load is +0.0 x 0.5 + +0.0 = +0.0: set the results at once.
    if m.wheel_loads.iter().all(|&load| !figure_is_set(load)) {
        m.wheel_load_mean = 0.0;
        m.scratch = 0.0;
        return;
    }
    let [l0, l1, l2, l3] = m.wheel_loads;
    m.wheel_load_mean =
        ((figure(l0) + figure(l1) + figure(l2) + figure(l3)) * figure(m.quarter)) as f32;
    m.scratch = (figure(m.wheel_load_mean) * figure(m.half)) as f32;
    for load in &mut m.wheel_loads {
        *load = (figure(*load) * figure(m.half) + figure(m.scratch)) as f32;
    }
}

#[cfg(test)]
mod reference_vectors_tests;

#[cfg(test)]
mod plain_matches_x87_tests;
