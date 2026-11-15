//! The damage model's figures for this step (`physics/damage_model.rs`).
//!
//! From the car block it works out, in the damage model: the ride-height, stiffness and brake
//! shares; per wheel the stiffness and ride height that follow the suspension damage (a wheel
//! at the top of the gauge is worn out); the engine's and the car's speed shares against their
//! limits;
//! the switches `wheels_down`, `brake_share` and `DamageModel.engine_over_limit` (the engine
//! past its limit); the copies of the gear, tyre kind and surface; the side hit
//! (`DamageModel.hit_side`, 1 .. 4) and its size; the four wheels' loads; and the hit time.
//!
//! Caller: `physics::damage_step`.

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
mod reference_vectors {
    use super::*;
    use crate::effects::cell_x87::CellX87;
    use crate::math::x87_float::compare;
    use crate::physics::store::laid;
    use crate::test_support::own::race;

    /// The status word of the engine's speed against its limit times the factor.
    fn over_limit_status(car: &Car, m: &DamageModel) -> u16 {
        compare(
            car.engine_speed_limit.ld() * m.engine_limit_factor.ld(),
            car.engine_speed.ld(),
        )
    }

    /// The damage model as the test's buffer holds it.
    fn dm(base: usize) -> &'static DamageModel {
        crate::physics::store::model_in_test_buffer(base)
    }
    use crate::scene::x87_mem::Base;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse_at};

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
                        reads: &[(laid!(0x35B35C), In::F)],
                        writes: &[
                            (laid!(0x35B3F8), Out::F),
                            (laid!(0x35B3FC), Out::F),
                            (laid!(0x35B400), Out::F),
                        ],
                    },
                ],
                ulps: 0,
                plain: |a| scale_car_figures(race().shared.car(), dm(a[1])),
            },
            Block {
                name: "damage_factors line 53",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (laid!(0x35B34C), In::F),
                            (laid!(0x35B3FC), In::F),
                            (laid!(0x35B474), In::F),
                        ],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A420, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    wheel_stiffness_from_share(
                        dm(a[0]),
                        &dm(a[0]).wheel_stiffness_lerp[0],
                        &crate::test_support::own::race()
                            .shared
                            .car()
                            .wheel_stiffness[0],
                    );
                },
            },
            Block {
                name: "damage_factors line 63",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(laid!(0x35B34C), In::F), (laid!(0x35B3F8), In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A410, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    wheel_ride_height_from_share(
                        dm(a[0]),
                        &crate::test_support::own::race()
                            .shared
                            .car()
                            .wheel_ride_height[0],
                    );
                },
            },
            Block {
                name: "damage_factors line 71",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(laid!(0x35B34C), In::F), (laid!(0x35B350), In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A410, Out::F), (0x31A420, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    fail_wheel(
                        dm(a[0]),
                        &crate::test_support::own::race()
                            .shared
                            .car()
                            .wheel_stiffness[0],
                        &crate::test_support::own::race()
                            .shared
                            .car()
                            .wheel_ride_height[0],
                    );
                },
            },
            Block {
                name: "damage_factors line 81",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (laid!(0x35B34C), In::F),
                            (laid!(0x35B3FC), In::F),
                            (laid!(0x35B478), In::F),
                        ],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A424, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    wheel_stiffness_from_share(
                        dm(a[0]),
                        &dm(a[0]).wheel_stiffness_lerp[1],
                        &crate::test_support::own::race()
                            .shared
                            .car()
                            .wheel_stiffness[1],
                    );
                },
            },
            Block {
                name: "damage_factors line 91",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(laid!(0x35B34C), In::F), (laid!(0x35B3F8), In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A414, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    wheel_ride_height_from_share(
                        dm(a[0]),
                        &crate::test_support::own::race()
                            .shared
                            .car()
                            .wheel_ride_height[1],
                    );
                },
            },
            Block {
                name: "damage_factors line 99",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(laid!(0x35B34C), In::F), (laid!(0x35B350), In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A414, Out::F), (0x31A424, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    fail_wheel(
                        dm(a[0]),
                        &crate::test_support::own::race()
                            .shared
                            .car()
                            .wheel_stiffness[1],
                        &crate::test_support::own::race()
                            .shared
                            .car()
                            .wheel_ride_height[1],
                    );
                },
            },
            Block {
                name: "damage_factors line 109",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (laid!(0x35B34C), In::F),
                            (laid!(0x35B3FC), In::F),
                            (laid!(0x35B47C), In::F),
                        ],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A428, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    wheel_stiffness_from_share(
                        dm(a[0]),
                        &dm(a[0]).wheel_stiffness_lerp[2],
                        &crate::test_support::own::race()
                            .shared
                            .car()
                            .wheel_stiffness[2],
                    );
                },
            },
            Block {
                name: "damage_factors line 119",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(laid!(0x35B34C), In::F), (laid!(0x35B3F8), In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A418, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    wheel_ride_height_from_share(
                        dm(a[0]),
                        &crate::test_support::own::race()
                            .shared
                            .car()
                            .wheel_ride_height[2],
                    );
                },
            },
            Block {
                name: "damage_factors line 127",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(laid!(0x35B34C), In::F), (laid!(0x35B350), In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A418, Out::F), (0x31A428, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    fail_wheel(
                        dm(a[0]),
                        &crate::test_support::own::race()
                            .shared
                            .car()
                            .wheel_stiffness[2],
                        &crate::test_support::own::race()
                            .shared
                            .car()
                            .wheel_ride_height[2],
                    );
                },
            },
            Block {
                name: "damage_factors line 137",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (laid!(0x35B34C), In::F),
                            (laid!(0x35B3FC), In::F),
                            (laid!(0x35B480), In::F),
                        ],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A42C, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    wheel_stiffness_from_share(
                        dm(a[0]),
                        &dm(a[0]).wheel_stiffness_lerp[3],
                        &crate::test_support::own::race()
                            .shared
                            .car()
                            .wheel_stiffness[3],
                    );
                },
            },
            Block {
                name: "damage_factors line 147",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(laid!(0x35B34C), In::F), (laid!(0x35B3F8), In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A41C, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    wheel_ride_height_from_share(
                        dm(a[0]),
                        &crate::test_support::own::race()
                            .shared
                            .car()
                            .wheel_ride_height[3],
                    );
                },
            },
            Block {
                name: "damage_factors line 155",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(laid!(0x35B34C), In::F), (laid!(0x35B350), In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A41C, Out::F), (0x31A42C, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    fail_wheel(
                        dm(a[0]),
                        &crate::test_support::own::race()
                            .shared
                            .car()
                            .wheel_stiffness[3],
                        &crate::test_support::own::race()
                            .shared
                            .car()
                            .wheel_ride_height[3],
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
                        reads: &[(laid!(0x35B364), In::F)],
                        writes: &[(laid!(0x35B36C), Out::F), (laid!(0x35B3E4), Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| means_and_engine_speed(race().shared.car(), dm(a[1])),
            },
            Block {
                name: "damage_factors line 186",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(laid!(0x35B34C), In::F), (laid!(0x35B36C), In::F)],
                        writes: &[(laid!(0x35B36C), Out::F), (laid!(0x35B398), Out::F)],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A184, In::F), (0x31A1B4, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                plain: |a| engine_speed_share_step(race().shared.car(), dm(a[0])),
            },
            Block {
                name: "damage_factors line 198",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (laid!(0x35B34C), In::F),
                            (laid!(0x35B36C), In::F),
                            (laid!(0x35B378), In::F),
                        ],
                        writes: &[
                            (laid!(0x35B36C), Out::F),
                            (laid!(0x35B39C), Out::F),
                            (laid!(0x35B3E0), Out::F),
                        ],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A184, In::F), (0x31A190, In::F), (0x31A930, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                plain: |a| speed_share_step(race().shared.car(), dm(a[0])),
            },
            Block {
                name: "damage_factors line 213",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(laid!(0x35B36C), In::F), (laid!(0x35B378), In::F)],
                        writes: &[(laid!(0x35B36C), Out::F), (laid!(0x35B3A0), Out::F)],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A184, In::F), (0x31A194, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                plain: |a| drive_a_slip_step(race().shared.car(), dm(a[0])),
            },
            Block {
                name: "damage_factors line 225",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (laid!(0x35B36C), In::F),
                            (laid!(0x35B378), In::F),
                            (laid!(0x35B6F4), In::F),
                        ],
                        writes: &[(laid!(0x35B36C), Out::F), (laid!(0x35B3A4), Out::F)],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A184, In::F), (0x31A190, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    drive_extrapolation_step(
                        race().shared.car(),
                        dm(a[0]),
                        &dm(a[0]).drive_b_slip_share,
                        &crate::test_support::own::race().shared.car().drive_speed_a,
                        &crate::test_support::own::race().shared.car().drive_speed_a,
                    );
                },
            },
            Block {
                name: "damage_factors line 239",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (laid!(0x35B36C), In::F),
                            (laid!(0x35B378), In::F),
                            (laid!(0x35B6F4), In::F),
                        ],
                        writes: &[(laid!(0x35B36C), Out::F), (laid!(0x35B3A8), Out::F)],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A184, In::F), (0x31A194, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    drive_extrapolation_step(
                        race().shared.car(),
                        dm(a[0]),
                        &dm(a[0]).drive_a_extrapolated_share,
                        &crate::test_support::own::race().shared.car().drive_speed_b,
                        &crate::test_support::own::race().shared.car().drive_speed_b,
                    );
                },
            },
            Block {
                name: "damage_factors line 253",
                operands: vec![Operand::Buffer {
                    reads: &[(laid!(0x35B36C), In::F), (laid!(0x35B378), In::F)],
                    writes: &[(laid!(0x35B3AC), Out::F), (laid!(0x35B3F4), Out::F)],
                }],
                ulps: 0,
                plain: |a| finish_shares(dm(a[0])),
            },
            Block {
                name: "damage_factors line 262",
                operands: vec![Operand::Buffer {
                    reads: &[],
                    writes: &[(laid!(0x35B3F4), Out::F)],
                }],
                ulps: 0,
                plain: |a| dm(a[0]).wheels_down.set(1.0),
            },
            Block {
                name: "damage_factors line 268",
                operands: vec![Operand::Buffer {
                    reads: &[],
                    writes: &[(laid!(0x35B3B0), Out::F)],
                }],
                ulps: 0,
                plain: |a| dm(a[0]).brake_share.set(0.0),
            },
            Block {
                name: "damage_factors line 276",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[(0x31A614, In::I)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(laid!(0x35B35C), In::F)],
                        writes: &[(laid!(0x35B3B0), Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| brake_share_step(race().shared.car(), dm(a[1])),
            },
            Block {
                name: "damage_factors line 287",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[(0x31A1B0, In::F), (0x31A1B4, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(laid!(0x35B37C), In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::H)],
                    },
                ],
                ulps: 0,
                plain: |a| a[2].set_word(0, over_limit_status(race().shared.car(), dm(a[1]))),
            },
            Block {
                name: "damage_factors line 302",
                operands: vec![Operand::Buffer {
                    reads: &[],
                    writes: &[(laid!(0x35B3B4), Out::F)],
                }],
                ulps: 0,
                plain: |a| dm(a[0]).on_surface_5.set(0.0),
            },
            Block {
                name: "damage_factors line 322",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[(0x31A180, In::F), (0x31A208, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[
                            (laid!(0x35B378), In::F),
                            (laid!(0x35B394), In::F),
                            (laid!(0x35B624), In::F),
                        ],
                        writes: &[(laid!(0x35B36C), Out::F), (laid!(0x35B3C8), Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| hit_size_and_dive(race().shared.car(), dm(a[1])),
            },
            Block {
                name: "damage_factors line 337",
                operands: vec![Operand::Buffer {
                    reads: &[(laid!(0x35B36C), In::F), (laid!(0x35B620), In::F)],
                    writes: &[(laid!(0x35B36C), Out::F)],
                }],
                ulps: 0,
                plain: |a| square_dive(dm(a[0])),
            },
            Block {
                name: "damage_factors line 350",
                operands: vec![Operand::Buffer {
                    reads: &[
                        (laid!(0x35B360), In::F),
                        (laid!(0x35B364), In::F),
                        (laid!(0x35B36C), In::F),
                        (laid!(0x35B3CC), In::F),
                        (laid!(0x35B3D0), In::F),
                        (laid!(0x35B3D4), In::F),
                        (laid!(0x35B3D8), In::F),
                        (laid!(0x35B3DC), In::F),
                    ],
                    writes: &[
                        (laid!(0x35B36C), Out::F),
                        (laid!(0x35B3CC), Out::F),
                        (laid!(0x35B3D0), Out::F),
                        (laid!(0x35B3D4), Out::F),
                        (laid!(0x35B3D8), Out::F),
                        (laid!(0x35B3DC), Out::F),
                    ],
                }],
                ulps: 0,
                plain: |a| blend_wheel_loads(dm(a[0])),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        // The plain code rounds to nearest, the mode the damage model always runs in.
        sparse_at("physics.damage_factors", &blocks(), 1200, 100, &[0x027F]);
    }
}

#[cfg(test)]
mod plain_matches_x87 {
    use super::*;
    use crate::physics::damage_model::proof::prove;

    #[test]
    fn shares_and_wheels() {
        prove("scale_car_figures", |s| {
            scale_car_figures(&s.car, &s.damage);
        });
        prove("wheel_stiffness_from_share", |s| {
            let (d, c) = (&s.damage, &s.car);
            wheel_stiffness_from_share(d, &d.wheel_stiffness_lerp[1], &c.wheel_stiffness[1]);
        });
        prove("wheel_ride_height_from_share", |s| {
            wheel_ride_height_from_share(&s.damage, &s.car.wheel_ride_height[2]);
        });
        prove("fail_wheel", |s| {
            fail_wheel(
                &s.damage,
                &s.car.wheel_stiffness[3],
                &s.car.wheel_ride_height[3],
            );
        });
        prove("means_and_engine_speed", |s| {
            s.car.drive_kind.set(s.car.drive_kind.get() & 3);
            means_and_engine_speed(&s.car, &s.damage);
        });
    }

    #[test]
    fn speed_shares() {
        prove("engine_speed_share_step", |s| {
            engine_speed_share_step(&s.car, &s.damage);
        });
        prove("speed_share_step", |s| speed_share_step(&s.car, &s.damage));
        prove("drive_a_slip_step", |s| {
            drive_a_slip_step(&s.car, &s.damage);
        });
        prove("drive_extrapolation_step", |s| {
            let (c, d) = (&s.car, &s.damage);
            drive_extrapolation_step(
                c,
                d,
                &d.drive_b_slip_share,
                &c.drive_speed_a,
                &c.drive_speed_a,
            );
        });
        prove(
            "drive_extrapolation_step with a separate carried figure",
            |s| {
                let (c, d) = (&s.car, &s.damage);
                drive_extrapolation_step(
                    c,
                    d,
                    &d.drive_a_extrapolated_share,
                    &c.drive_speed_b,
                    &c.drive_speed_a,
                );
            },
        );
        prove("finish_shares", |s| finish_shares(&s.damage));
        prove("brake_share_step", |s| brake_share_step(&s.car, &s.damage));
    }

    #[test]
    fn engine_limit_hit_and_loads() {
        prove("engine_over_limit", |s| {
            s.damage
                .scratch
                .set_bits(u32::from(engine_over_limit(&s.car, &s.damage)));
        });
        prove("hit_size_and_dive", |s| {
            hit_size_and_dive(&s.car, &s.damage);
        });
        prove("square_dive", |s| square_dive(&s.damage));
        prove("blend_wheel_loads", |s| blend_wheel_loads(&s.damage));
        // No wheel just landed: the loads are zero, the constants are the model's own.
        let still = |s: &crate::physics::damage_model::proof::State| {
            for load in &s.damage.wheel_loads {
                load.set(0.0);
            }
            s.damage.quarter.set(0.25);
            s.damage.half.set(0.5);
        };
        prove("blend_wheel_loads at rest", |s| {
            still(s);
            blend_wheel_loads(&s.damage);
        });
        prove("the touch-down load", |s| {
            let (c, d) = (&s.car, &s.damage);
            store(
                &d.wheel_loads[0],
                value(&c.wheel_squeeze[0]) * value(&d.scratch),
            );
        });
    }
}
