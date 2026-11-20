//! The damage model's figures for this step (`physics/damage_model.rs`).
//!
//! From the car block it works out, in the damage model: the ride-height, stiffness and brake
//! shares; per wheel the stiffness and ride height that follow the suspension damage (a wheel
//! at the top of the gauge is worn out); the engine's and the car's speed shares against their
//! limits;
//! the switches `wheels_down`, `brake_share` and `DamageModel.engine_over_limit` (the engine
//! past its limit); the copies of the gear, tyre kind and surface; the side hit
//! (`DamageModel.hit_side`, 1 .. 4) and its size; the four wheels' loads; and the hit time.

use crate::memory::session::PlainFloat;
use crate::physics::car_store::Car;
use crate::physics::damage_model::{is_set, store, value, whole};
use crate::physics::flow::{
    abs_below_or_unordered, below_or_unordered, equal_or_unordered, hold_high, hold_low, table_lerp,
};
use crate::physics::store::DamageModel;
use core::cmp::Ordering;

/// The damage model's figures for this step.
pub fn steps(car: &Car, damage: &DamageModel) {
    scale_car_figures(car, damage);
    // Per wheel: the stiffness and ride height follow the suspension damage; at the top
    // of the gauge the wheel is worn out.
    for wheel in 0..4 {
        let suspension = &car.suspension_damage[wheel];
        let stiffness = &car.wheel_stiffness[wheel];
        let ride_height = &car.wheel_ride_height[wheel];
        let lerp = &damage.wheel_stiffness_lerp[wheel];
        table_lerp(suspension, &damage.lerp_tables[2], &damage.gauge_low, lerp);
        wheel_stiffness_from_share(damage, lerp, stiffness);
        hold_low(stiffness, &damage.gauge_low);
        hold_high(stiffness, &damage.gauge_top);
        wheel_ride_height_from_share(damage, ride_height);
        if equal_or_unordered(suspension, &damage.gauge_top) {
            fail_wheel(damage, stiffness, ride_height);
        }
    }
    means_and_engine_speed(car, damage);
    hold_high(&damage.scratch, &car.engine_speed_limit);
    engine_speed_share_step(car, damage);
    // The speed gaps are each held to `speed_range` before their share is taken.
    let speed_range = &damage.speed_range;
    hold_high(&damage.scratch, speed_range);
    speed_share_step(car, damage);
    hold_high(&damage.scratch, speed_range);
    drive_a_slip_step(car, damage);
    hold_high(&damage.scratch, speed_range);
    drive_extrapolation_step(
        car,
        damage,
        &damage.drive_b_slip_share,
        &car.drive_speed_a,
        &car.drive_speed_a,
    );
    hold_high(&damage.scratch, speed_range);
    drive_extrapolation_step(
        car,
        damage,
        &damage.drive_a_extrapolated_share,
        &car.drive_speed_b,
        &car.drive_speed_b,
    );
    hold_high(&damage.scratch, speed_range);
    finish_shares(damage);
    if car.all_wheels_off.get() == 0 {
        damage.wheels_down.set(1.0);
    }
    damage.brake_share.set(0.0);
    if !abs_below_or_unordered(&car.local_velocity[2], &damage.grade_high)
        && (car.brake.get().wrapping_sub(0x10) as i32) >= 0
    {
        brake_share_step(car, damage);
    }
    // 1 when the engine's speed is above the limit times `DamageModel.engine_limit_factor`.
    damage
        .engine_over_limit
        .set(if engine_over_limit(car, damage) {
            1.0
        } else {
            0.0
        });
    damage.gear_changed.set(car.gear_changed.get());
    damage.gear.set(car.gear.get());
    damage.tyre_kind.set(car.tyre_kind.get());
    let surface = car.wheel_surface_a[4].get();
    damage.surface.set(surface);
    damage.on_surface_5.set(0.0);
    if surface == 5 {
        damage.on_surface_5.set(1.0);
    }
    damage.surface_blend.set(car.surface_blend.get());
    damage.hit_side.set(side_hit(car, damage));
    hit_size_and_dive(car, damage);
    hold_low(&damage.scratch, &damage.gauge_low);
    hold_high(&damage.scratch, &damage.dive_top);
    square_dive(damage);
    // A wheel that has just touched down carries the dive's load.
    for wheel in 0..4 {
        damage.wheel_loads[wheel].set(0.0);
        // Word tests: a squeeze of -0.0 counts as touching.
        if !is_set(&car.wheel_squeeze_last[wheel]) && is_set(&car.wheel_squeeze[wheel]) {
            store(
                &damage.wheel_loads[wheel],
                value(&car.wheel_squeeze[wheel]) * value(&damage.scratch),
            );
        }
    }
    blend_wheel_loads(damage);
    if is_set(&damage.hit_size) && car.hit_cooldown.get() != 0 {
        damage.hit_size.set(0.0);
        car.hit_cooldown.set(0x64);
    }
}

/// Where the car was hit: 0 for no contact, else 1 .. 4 by the side of the contact point.
fn side_hit(car: &Car, damage: &DamageModel) -> u32 {
    if !is_set(&car.contact_depth) {
        return 0;
    }
    let mut hit = 1;
    if !below_or_unordered(&car.contact_point[0], &damage.gauge_low) {
        hit += 1;
    }
    if below_or_unordered(&car.contact_point[2], &damage.gauge_low) {
        hit += 2;
    }
    hit
}

// ---- the arithmetic ----

/// The ride height, stiffness and brake balance of the car, which it holds as whole numbers
/// over 256, as shares.
fn scale_car_figures(car: &Car, m: &DamageModel) {
    let scale = value(&m.fixed_point_scale);
    store(&m.ride_height_share, whole(&car.ride_height) / scale);
    store(&m.stiffness_share, whole(&car.suspension_stiffness) / scale);
    store(&m.brake_balance_share, whole(&car.brake_balance) / scale);
}

/// A wheel's stiffness: the stiffness share times the top of the gauge, less the lerp of its
/// suspension damage.
fn wheel_stiffness_from_share(m: &DamageModel, lerp: &PlainFloat, stiffness: &PlainFloat) {
    store(
        stiffness,
        value(&m.stiffness_share) * value(&m.gauge_top) - value(lerp),
    );
}

/// A wheel's ride height: the ride height share times the top of the gauge.
fn wheel_ride_height_from_share(m: &DamageModel, ride_height: &PlainFloat) {
    store(
        ride_height,
        value(&m.ride_height_share) * value(&m.gauge_top),
    );
}

/// A worn-out wheel: full stiffness, no ride height.
fn fail_wheel(m: &DamageModel, stiffness: &PlainFloat, ride_height: &PlainFloat) {
    store(stiffness, value(&m.gauge_top));
    store(ride_height, value(&m.gauge_low));
}

/// The means of the wheels' ride heights and stiffnesses, the drive kind, and the engine's
/// speed made positive (into `scratch`).
fn means_and_engine_speed(car: &Car, m: &DamageModel) {
    let quarter = value(&m.quarter);
    let [h0, h1, h2, h3] = &car.wheel_ride_height;
    store(
        &car.ride_height_mean,
        (value(h0) + value(h1) + value(h2) + value(h3)) * quarter,
    );
    let [s0, s1, s2, s3] = &car.wheel_stiffness;
    store(
        &car.stiffness_mean,
        (value(s0) + value(s1) + value(s2) + value(s3)) * quarter,
    );
    // The car's whole number passes through the x87 as a float, which gives back the same bits.
    m.drive_kind.set(car.drive_kind.get());
    store(&m.scratch, value(&car.engine_speed).abs());
}

/// The engine's speed (in `scratch`) over its limit as a share of the gauge, then the
/// forward speed made positive.
fn engine_speed_share_step(car: &Car, m: &DamageModel) {
    store(
        &m.engine_speed_share,
        value(&m.scratch) / value(&car.engine_speed_limit) * value(&m.gauge_top),
    );
    store(&m.scratch, value(&car.local_velocity[2]).abs());
}

/// The forward speed (in `scratch`) over the speed range as a share of the gauge, the surface
/// wear factor copied, then the gap between the forward speed and the first drive speed.
fn speed_share_step(car: &Car, m: &DamageModel) {
    store(
        &m.speed_share,
        value(&m.scratch) / value(&m.speed_range) * value(&m.gauge_top),
    );
    store(&m.point_surface_copy, value(&car.point_wear_factor[4]));
    store(
        &m.scratch,
        (value(&car.local_velocity[2]) - value(&car.drive_speed_a)).abs(),
    );
}

/// The first drive's slip share, then the gap between the forward speed and the second
/// drive speed.
fn drive_a_slip_step(car: &Car, m: &DamageModel) {
    store(
        &m.drive_a_slip_share,
        value(&m.scratch) / value(&m.speed_range),
    );
    store(
        &m.scratch,
        (value(&car.local_velocity[2]) - value(&car.drive_speed_b)).abs(),
    );
}

/// A drive's share into `share`, then the next gap: the drive speed's distance from the
/// forward speed, carried on by the extrapolation figure.
fn drive_extrapolation_step(
    car: &Car,
    m: &DamageModel,
    share: &PlainFloat,
    drive_speed: &PlainFloat,
    carried: &PlainFloat,
) {
    store(share, value(&m.scratch) / value(&m.speed_range));
    store(
        &m.scratch,
        ((value(drive_speed) - value(&car.local_velocity[2])) * value(&m.figures[31])
            + value(carried))
        .abs(),
    );
}

/// The last share, and no wheel down yet.
fn finish_shares(m: &DamageModel) {
    store(
        &m.drive_b_extrapolated_share,
        value(&m.scratch) / value(&m.speed_range),
    );
    m.wheels_down.set(0.0);
}

/// The brake pedal, a whole number over 256, as a share.
fn brake_share_step(car: &Car, m: &DamageModel) {
    store(
        &m.brake_share,
        whole(&car.brake) / value(&m.fixed_point_scale),
    );
}

/// Whether the engine's speed is above its limit times `engine_limit_factor` (also when
/// either is no number).
fn engine_over_limit(car: &Car, m: &DamageModel) -> bool {
    let limit = value(&car.engine_speed_limit) * value(&m.engine_limit_factor);
    !matches!(
        limit.partial_cmp(&value(&car.engine_speed)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// The size of the hit: the contact depth squared over the divisor over the speed range;
/// and, into `scratch`, the vertical speed's fall less the dive rate.
fn hit_size_and_dive(car: &Car, m: &DamageModel) {
    let depth = value(&car.contact_depth);
    store(
        &m.hit_size,
        depth * depth / value(&m.hit_divisor) / value(&m.speed_range),
    );
    store(
        &m.scratch,
        -value(&car.local_velocity[1]) - value(&m.gauge_rates[21]),
    );
}

/// The dive squared times its rate.
fn square_dive(m: &DamageModel) {
    let dive = value(&m.scratch);
    store(&m.scratch, dive * dive * value(&m.gauge_rates[20]));
}

/// The wheels' loads: their mean, and each load halved with half the mean added.
fn blend_wheel_loads(m: &DamageModel) {
    // No wheel just landed (the usual step): the four loads are +0.0, so the sum, the mean
    // (`quarter` 0.25) and the half-mean (`half` 0.5, both constants of the model) are +0.0 in
    // every rounding mode, and each load is +0.0 x 0.5 + +0.0 = +0.0: set the results at once.
    if m.wheel_loads.iter().all(|load| !is_set(load)) {
        m.wheel_load_mean.set(0.0);
        m.scratch.set(0.0);
        return;
    }
    let [l0, l1, l2, l3] = &m.wheel_loads;
    store(
        &m.wheel_load_mean,
        (value(l0) + value(l1) + value(l2) + value(l3)) * value(&m.quarter),
    );
    store(&m.scratch, value(&m.wheel_load_mean) * value(&m.half));
    for load in &m.wheel_loads {
        store(load, value(load) * value(&m.half) + value(&m.scratch));
    }
}

#[cfg(test)]
mod reference_vectors_tests;

#[cfg(test)]
mod plain_matches_x87_tests;
