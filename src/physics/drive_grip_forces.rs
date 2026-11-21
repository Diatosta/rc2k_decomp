//! The driven wheels' grip forces for the step. In order: `lever_share` = 1 - `wall_hit_timer`
//! / 50, held to 0 ..= 1; `grip_forces[0]` = the steer's sine (`steer_angle` >> 20 into
//! the sine table) x speed / `spec_grip` (applied twice); when moving: the slip limit `grip_limit`
//! (a grip figure x `gravity` x a drive-kind factor x `grip_scale` / speed, absolute value),
//! `grip_forces[0]` held to its limit, 0 below `grip_slow_speed` with the drive speed not; with
//! bit 12 of `input_keys` the `steering` scaled by `grip_forces[0]`'s share of what it was (held
//! 0 ..= 1), or 0; `grip_forces[0]` x `grip_rear_drive` / the next table figure with
//! `parts_state` 1; the drive-kind 0 lean force `grip_forces[3]`; the engine braking
//! `grip_forces[1]` (by drive kind, the counts at `brake`, `throttle` and `handbrake`, scaled,
//! x `side_grip_share` against `grip_forces[0]`'s sign, x `grip_front_drive` / the next with
//! `parts_state` -1); the drive-kind 1 yaw force `grip_forces[2]` (speed's share of
//! `grip_slip_limit`, `drive_slip_speed` held to +-`grip_force_limit`); `grip_force_sum` the sum
//! of the four; force records 19 and 20 are 0, or with the wheels on the ground the yaw twist is
//! eased toward `grip_force_sum` by `grip_ease_step` (`grip_change`) and split over the rear
//! wheels' places (`probe_point_z`).
//!
//! The flow, the compares and the integer steps are Rust; each straight x87 run is plain Rust
//! over `X` in the named helpers below.

use crate::math::cell_x87::CellX87;
use crate::math::engine_tables::EngineTables;
use crate::math::x87;
use crate::math::x87::mul_status;
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::to_int;
use crate::math::x87_float::{X, stf};
use crate::memory::session::PlainFloat;
use crate::physics::car_store::Car;
use crate::physics::figures::{
    GRIP_EASE_STEP, GRIP_FACTOR, GRIP_FRONT_DRIVE, GRIP_REAR_DRIVE, GRIP_SLOW_SPEED,
    SIDE_GRIP_SHARE,
};
use crate::physics::flow::FloatView;
use crate::physics::flow::{
    ZERO_BITS as ZERO, abs_below_or_unordered, c3, ease_zero, hold_high, hold_high_to,
    hold_low_bits, hold_low_to, negate, negated,
};
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;

/// The steer angle's table index (the angle >> 20).
fn steer_index(car: &Car) -> u32 {
    car.steer_angle.get() >> 20
}

/// `a b` is below `c`, or one of them is not a number (the compare's C0).
fn product_below(a: u32, b: u32, c: u32) -> bool {
    mul_status(a, b, c) & 0x100 != 0
}

pub fn drive_grip_forces(
    car: &Car,
    tables: &PhysicsTables,
    numbers: &Numbers,
    grip_change: &crate::memory::session::PlainFloat,
    engine_tables: &EngineTables,
    scene: &crate::scene::owner::Scene,
) {
    // The work value the steps below pass through; each is stored before it is read.
    let scratch = crate::memory::session::PlainDword::new(0);
    let kind = car.drive_kind.get();
    // The share of `wall_hit_timer` out of 50, from 1, held 0 ..= 1.
    scratch.set(0x32);
    lever_share_from_timer(car, tables, numbers, &scratch);
    hold_low_bits(&tables.lever_share, ZERO);
    hold_high(&tables.lever_share, &numbers.one);
    car.grip_forces[0].set(0.0);
    if car.front_wheels_off.get() == 0 {
        steer_sine_force(car, tables, engine_tables, steer_index(car) as usize);
    }
    // The force is taken a second time over.
    steer_sine_force(car, tables, engine_tables, steer_index(car) as usize);
    if !c3(car.local_velocity.bits_at(2), ZERO) {
        // The slip limit, by the drive kind's factor.
        let factor = if kind == 1 {
            GRIP_FACTOR[1]
        } else {
            GRIP_FACTOR[0]
        };
        grip_limit_for(car, tables, factor);
        scratch.set_float(car.grip_forces[0].get());
        // The first grip force held to +-the limit (the value negated in place between).
        hold_high(&car.grip_forces[0], &tables.grip_limit);
        negate(&car.grip_forces[0]);
        hold_high(&car.grip_forces[0], &tables.grip_limit);
        negate(&car.grip_forces[0]);
        let slow = |v: &PlainFloat| abs_below_or_unordered(v, GRIP_SLOW_SPEED);
        if !abs_below_or_unordered(car.local_velocity.bits_at(2), GRIP_SLOW_SPEED)
            && slow(&car.drive_speed_a)
        {
            car.grip_forces[0].set(0.0);
        }
        if car.input_keys.get() & 0x1000 != 0 {
            if scratch.get() == 0 {
                car.steering.set(0);
            } else {
                divide_into_scratch(&car.grip_forces[0], scratch.float(), &scratch);
                hold_high(&scratch, &numbers.one);
                hold_low_bits(&scratch, ZERO);
                scale_steering(car, &scratch);
            }
        }
    }
    if car.parts_state.get() == 1 {
        let k = if kind == 1 {
            GRIP_REAR_DRIVE[0]
        } else {
            GRIP_REAR_DRIVE[1]
        };
        car.grip_forces[0].set(x87::mul(&car.grip_forces[0], k));
    }
    car.grip_forces[3].set(0.0);
    if kind == 0 {
        ground_speed_into_scratch(car, &scratch);
        hold_high_to(&scratch, numbers.grip_ramp.get());
        lean_force(
            car,
            numbers,
            engine_tables,
            steer_index(car) as usize,
            &scratch,
        );
    }
    // The engine braking.
    let braking_word = if kind != 0 {
        braking_count_geared(car, tables, numbers, &scratch);
        ((car.brake.get().wrapping_sub(car.throttle.get()) as i32 >> 1) as u32)
            .wrapping_add(0x80)
            .wrapping_add(scratch.get())
            .wrapping_add(car.handbrake.get())
    } else {
        braking_count_neutral(car, tables, &scratch);
        0x100u32
            .wrapping_sub(car.throttle.get())
            .wrapping_add(scratch.get())
            .wrapping_add(car.handbrake.get())
    };
    tables.grip_drive_word.set(braking_word as i32);
    engine_braking_force(car, tables, numbers);
    if product_below(car.grip_forces[1].bits(), car.grip_forces[0].bits(), ZERO) {
        car.grip_forces[1].set(x87::mul(&car.grip_forces[1], SIDE_GRIP_SHARE));
    }
    if car.parts_state.get() == u32::MAX {
        let k = if kind == 1 {
            GRIP_FRONT_DRIVE[0]
        } else {
            GRIP_FRONT_DRIVE[1]
        };
        car.grip_forces[1].set(x87::mul(&car.grip_forces[1], k));
    }
    car.grip_forces[2].set(0.0);
    if kind == 1 {
        abs_speed_into_work(car, scene);
        let work_float = crate::memory::session::PlainFloat::new(scene.draw_work().work_float_a);
        hold_high_to(&work_float, numbers.grip_slip_limit.get());
        scene.draw_work_mut().work_float_a = work_float.get();
        yaw_share_from_limit(numbers, scene);
        scratch.set(x87::loaded(&car.drive_slip_speed).to_bits());
        // Held to ±`GRIP_FORCE_LIMIT`.
        hold_high_to(&scratch, numbers.grip_force_limit.get());
        hold_low_to(&scratch, negated(numbers.grip_force_limit.get()));
        divide_into_scratch(&scratch, numbers.grip_force_limit.get(), &scratch);
        yaw_force(
            car,
            tables,
            numbers,
            engine_tables,
            steer_index(car) as usize,
            scene,
            &scratch,
        );
    }
    // The sum, and the yaw twist on the rear wheels.
    sum_grip_forces(car);
    tables.forces[19].force[0].set(0.0);
    tables.forces[20].force[0].set(0.0);
    if car.all_wheels_off.get() != 0 {
        return;
    }
    car.eased_yaw_spin.set(x87::loaded(car.spin.bits_at(1)));
    let old = car.eased_yaw_spin.bits();
    grip_change.set(x87::sub(old, &car.grip_force_sum));
    ease_zero(grip_change, GRIP_EASE_STEP);
    ease_yaw_twist(car, tables, numbers, grip_change, old, &scratch);
}

// ---- the x87 arithmetic ----

/// One straight x87 run as plain Rust over `X`.
fn steer_sine_force(car: &Car, tables: &PhysicsTables, engine_tables: &EngineTables, e: usize) {
    let t0 = engine_tables.sines[e].ld();
    let t1 = car.local_velocity.ld(2);
    let t2 = t0 * t1;
    let t3 = tables.spec_grip.ld();
    let t4 = t2 / t3;
    car.grip_forces[0].st(t4);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[lever_share] = -((int [wall_hit_timer] / int [scratch]) - [one])`
fn lever_share_from_timer(
    car: &Car,
    tables: &PhysicsTables,
    numbers: &Numbers,
    scratch: &crate::memory::session::PlainDword,
) {
    let t0 = int(car.wall_hit_timer.get() as i32);
    let t1 = int(scratch.get() as i32);
    let t2 = t0 / t1;
    let t3 = numbers.one.ld();
    let t4 = t2 - t3;
    let t5 = -t4;
    tables.lever_share.st(t5);
}

/// One straight x87 run as plain Rust over `X`.
fn grip_limit_for(car: &Car, tables: &PhysicsTables, f: f32) {
    let t0 = tables.wheel_grips[4].ld();
    let t1 = tables.gravity.ld();
    let t2 = t0 * t1;
    let t3 = ld(f);
    let t4 = t2 * t3;
    let t5 = tables.grip_scale.ld();
    let t6 = t4 * t5;
    let t7 = car.local_velocity.ld(2);
    let t8 = t6 / t7;
    let t9 = t8.abs();
    tables.grip_limit.st(t9);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[scratch] = [rc + o0] / [rc + o1]`
fn divide_into_scratch(o0: &impl FloatView, o1: f32, scratch: &crate::memory::session::PlainDword) {
    let t0 = crate::math::x87_float::ld(o0.float());
    let t1 = crate::math::x87_float::ld(o1);
    let t2 = t0 / t1;
    scratch.st(t2);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [steering] = [scratch] * int [steering]`
fn scale_steering(car: &Car, scratch: &crate::memory::session::PlainDword) {
    let t0 = scratch.ld();
    let t1 = int(car.steering.get() as i32);
    let t2 = t0 * t1;
    car.steering.set(to_int(t2) as u32);
}

/// One straight x87 run as plain Rust over `X`.
fn ground_speed_into_scratch(car: &Car, scratch: &crate::memory::session::PlainDword) {
    let t0 = car.local_velocity.ld(0);
    let t1 = t0 * t0;
    let t2 = car.local_velocity.ld(2);
    let t3 = t2 * t2;
    let t4 = t1 + t3;
    let t5 = t4.sqrt();
    scratch.st(t5);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// A long formula, covered by the tests.
fn lean_force(
    car: &Car,
    numbers: &Numbers,
    engine_tables: &EngineTables,
    e: usize,
    scratch: &crate::memory::session::PlainDword,
) {
    let t0 = numbers.grip_ramp.ld();
    let t1 = scratch.ld();
    let t2 = t0 - t1;
    let t3 = numbers.grip_ramp.ld();
    let t4 = t2 / t3;
    let t5 = engine_tables.sines[e].ld();
    let t6 = t4 * t5;
    let t7 = int(car.handbrake.get() as i32);
    let t8 = t6 * t7;
    let t9 = numbers.k256.ld();
    let t10 = t8 / t9;
    let t11 = car.drive_speed_a.ld();
    let t12 = t10 * t11;
    let t13 = numbers.grip_ramp_share.ld();
    let t14 = t12 * t13;
    let t15 = int(car.throttle.get() as i32);
    let t16 = t14 * t15;
    let t17 = numbers.k256.ld();
    let t18 = t16 / t17;
    car.grip_forces[3].st(t18);
}

/// One straight x87 run as plain Rust over `X`.
fn braking_count_geared(
    car: &Car,
    tables: &PhysicsTables,
    numbers: &Numbers,
    scratch: &crate::memory::session::PlainDword,
) {
    let t0 = tables.handling[1].ld();
    let t1 = int(car.brake.get() as i32);
    let t2 = t0 * t1;
    let t3 = numbers.one_and_a_half.ld();
    let t4 = t2 * t3;
    scratch.set(to_int(t4) as u32);
}

/// One straight x87 run as plain Rust over `X`.
fn braking_count_neutral(
    car: &Car,
    tables: &PhysicsTables,
    scratch: &crate::memory::session::PlainDword,
) {
    let t0 = tables.handling[1].ld();
    let t1 = int(car.brake.get() as i32);
    let t2 = t0 * t1;
    let t3 = t2 + t2;
    scratch.set(to_int(t3) as u32);
}

/// One straight x87 run as plain Rust over `X`.
fn engine_braking_force(car: &Car, tables: &PhysicsTables, numbers: &Numbers) {
    let t0 = int(tables.grip_drive_word.get());
    let t1 = numbers.k256.ld();
    let t2 = t0 / t1;
    let t3 = car.side_speed_held.ld();
    let t4 = t2 * t3;
    let t5 = -t4;
    let t6 = numbers.side_force_scale.ld();
    let t7 = t5 * t6;
    let t8 = tables.side_grip_scale.ld();
    let t9 = t7 * t8;
    car.grip_forces[1].st(t9);
}

/// One straight x87 run as plain Rust over `X`.
fn abs_speed_into_work(car: &Car, scene: &crate::scene::owner::Scene) {
    let t0 = car.local_velocity.ld(2);
    let t1 = t0.abs();
    scene.draw_work_mut().work_float_a = stf(t1);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[work_float_a] = ([grip_slip_limit] - [work_float_a]) / [grip_slip_limit]`
fn yaw_share_from_limit(numbers: &Numbers, scene: &crate::scene::owner::Scene) {
    let t0 = numbers.grip_slip_limit.ld();
    let t1 = ld(scene.draw_work().work_float_a);
    let t2 = t0 - t1;
    let t3 = numbers.grip_slip_limit.ld();
    let t4 = t2 / t3;
    scene.draw_work_mut().work_float_a = stf(t4);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// A long formula, covered by the tests.
fn yaw_force(
    car: &Car,
    tables: &PhysicsTables,
    numbers: &Numbers,
    engine_tables: &EngineTables,
    e: usize,
    scene: &crate::scene::owner::Scene,
    scratch: &crate::memory::session::PlainDword,
) {
    let t0 = engine_tables.sines[e].ld();
    let t1 = tables.spec_grip.ld();
    let t2 = t0 / t1;
    let t3 = int(car.throttle.get() as i32);
    let t4 = t2 * t3;
    let t5 = numbers.k256.ld();
    let t6 = t4 / t5;
    let t7 = scratch.ld();
    let t8 = t6 * t7;
    let t9 = ld(scene.draw_work().work_float_a);
    let t10 = t8 * t9;
    let t11 = numbers.grip_slide_scale.ld();
    let t12 = t10 * t11;
    car.grip_forces[2].st(t12);
}

/// One straight x87 run as plain Rust over `X`.
fn sum_grip_forces(car: &Car) {
    let t0 = car.grip_forces[0].ld();
    let t1 = car.grip_forces[1].ld();
    let t2 = t0 + t1;
    let t3 = car.grip_forces[2].ld();
    let t4 = t2 + t3;
    let t5 = car.grip_forces[3].ld();
    let t6 = t4 + t5;
    car.grip_force_sum.st(t6);
}

/// One straight x87 run as plain Rust over `X`.
fn ease_yaw_twist(
    car: &Car,
    tables: &PhysicsTables,
    numbers: &Numbers,
    grip_change: &crate::memory::session::PlainFloat,
    o: u32,
    scratch: &crate::memory::session::PlainDword,
) {
    let t0 = grip_change.ld();
    let t1 = car.grip_force_sum.ld();
    let t2 = t0 + t1;
    car.eased_yaw_spin.st(t2);
    let t3 = ld(o);
    let t4 = car.eased_yaw_spin.ld();
    let t5 = t3 - t4;
    let t6 = -t5;
    scratch.st(t6);
    let t7 = scratch.ld();
    let t8 = numbers.step_time.ld();
    let t9 = t7 / t8;
    let t10 = numbers.half.ld();
    let t11 = t9 * t10;
    let t12 = numbers.inertia[1].ld();
    let t13 = t11 / t12;
    tables.forces[19].force[0].st(t13);
    let t14 = -t13;
    tables.forces[20].force[0].st(t14);
    let t15 = X(1.0);
    tables.forces[19].point[2].st(t15);
    let t16 = -t15;
    tables.forces[20].point[2].st(t16);
    let t17 = tables.forces[19].force[0].ld();
    let t18 = car.probe_point_z[0].ld();
    let t19 = t17 / t18;
    tables.forces[19].force[0].st(t19);
    let t20 = tables.forces[19].point[2].ld();
    let t21 = car.probe_point_z[0].ld();
    let t22 = t20 * t21;
    tables.forces[19].point[2].st(t22);
    let t23 = tables.forces[20].force[0].ld();
    let t24 = car.probe_point_z[3].ld();
    let t25 = t23 / t24;
    let t26 = -t25;
    tables.forces[20].force[0].st(t26);
    let t27 = tables.forces[20].point[2].ld();
    let t28 = car.probe_point_z[3].ld();
    let t29 = t27 * t28;
    let t30 = -t29;
    tables.forces[20].point[2].st(t30);
}

#[cfg(test)]
mod tests;
