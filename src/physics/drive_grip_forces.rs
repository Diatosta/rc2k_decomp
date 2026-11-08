//! `physics::drive_grip_forces` (0x43CD79): the driven wheels' grip forces for the step.
//! In order: `flt_614768` = -(+0x1F4 / 50 - 1) held to 0 ..= 1; +0x210 the steer's sine
//! (+0x64C >> 20 into `g_sines`) x speed / `dword_614BA0` (twice over, as the original);
//! when moving: the slip limit `flt_614260` (a grip figure x `flt_613FC8` x a drive-kind
//! factor x `dword_614B74` / speed, `fabs`), +0x210 held to ±it, 0 below `flt_613E84`
//! speed with the drive speed not; with bit 12 of +0x468 +0x644 scaled by +0x210's share
//! of what it was (held 0 ..= 1, `fimul`, `fistp`), or 0; +0x210 x `flt_613E5C` /
//! `flt_613E60` with +0x284 1; the drive-kind 0 lean force +0x21C; the engine braking
//! +0x214 (by drive kind, the counts at +0x63C / +0x638 / +0x640, scaled, x `flt_613EB4`
//! against +0x210's sign, x `flt_613E54` / `58` with +0x284 -1); the drive-kind 1 yaw force
//! +0x218 (speed's share of `flt_613E7C`, +0x180 held to ±`flt_613E80`, negated in place
//! and put back); +0x204 the sum of the four; force records 19 and 20 (`g_forceRecords`)
//! 0, or with +0x254 clear the yaw twist eased toward +0x204 by `flt_613EA4`
//! (`flt_5C5E48`) and split over the rear wheels' places (+0xCF0 ..).
//!
//! The flow, the compares and the integer steps are Rust; each straight x87 run is plain
//! Rust over `X` (`math/x87_float.rs`) in the named helpers below, the original's
//! instructions being `asm_reference` in the tests.
//!
//! Caller: `physics::car_physics_step` (0x43B7A3), every physics step.

use crate::math::x87;
use crate::math::x87_float::X;
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::math::x87_float::to_int;
use crate::math::x87_run::mul_status;
use crate::physics::flow::{abs_below_or_unordered, c3, ease_zero, hold_high, hold_low, negate};
use crate::race::owner::Race;
use crate::race::owner_shared::Shared;
use crate::scene::engine_tables_store::ENGINE_TABLES;
use crate::session::Float;

/// The steer angle's table index (the angle >> 20).
fn steer_index(shared: &Shared) -> u32 {
    shared.car.steer_angle.get() >> 20
}

/// `a b` is below `c`, or one of them is not a number (the compare's C0).
fn product_below(a: u32, b: u32, c: u32) -> bool {
    mul_status(a, b, c) & 0x100 != 0
}

/// The Rust version.
pub fn drive_grip_forces(race: &Race, scene: &crate::scene::owner::Scene) {
    let drawing = &scene.drawing;
    let kind = race.shared.car.drive_kind.get();
    // 0x43CD79: the share of +0x1F4 out of 50, from 1, held 0 ..= 1.
    drawing.scene.scratch.set(0x32);
    lever_share_from_timer(race, scene);
    hold_low(
        &race.shared.physics_tables.lever_share,
        &race.course.numbers.zero,
    );
    hold_high(
        &race.shared.physics_tables.lever_share,
        &race.course.numbers.one,
    );
    race.shared.car.grip_forces[0].set_bits(0);
    if race.shared.car.front_wheels_off.get() == 0 {
        steer_sine_force(&race.shared, steer_index(&race.shared) as usize);
    }
    // The original takes the force a second time over.
    steer_sine_force(&race.shared, steer_index(&race.shared) as usize);
    if !c3(
        race.shared.car.local_velocity[2].bits(),
        race.course.numbers.zero.bits(),
    ) {
        // 0x43CE53: the slip limit, by the drive kind's factor.
        let factor = if kind == 1 {
            &race.shared.physics_tables.grip_factor[1]
        } else {
            &race.shared.physics_tables.grip_factor[0]
        };
        grip_limit_for(&race.shared, factor);
        drawing
            .scene
            .scratch
            .set(race.shared.car.grip_forces[0].bits());
        // +0x210 held to ±the limit (the value negated in place between).
        hold_high(
            &race.shared.car.grip_forces[0],
            &race.shared.physics_tables.grip_limit,
        );
        negate(&race.shared.car.grip_forces[0]);
        hold_high(
            &race.shared.car.grip_forces[0],
            &race.shared.physics_tables.grip_limit,
        );
        negate(&race.shared.car.grip_forces[0]);
        let slow = |v: &Float| {
            abs_below_or_unordered(v.bits(), race.shared.physics_tables.grip_slow_speed.bits())
        };
        if !slow(&race.shared.car.local_velocity[2]) && slow(&race.shared.car.drive_speed_a) {
            race.shared.car.grip_forces[0].set_bits(0);
        }
        if race.shared.car.input_keys.get() & 0x1000 != 0 {
            if drawing.scene.scratch.get() == 0 {
                race.shared.car.steering.set(0);
            } else {
                divide_into_scratch(
                    &race.shared.car.grip_forces[0],
                    &drawing.scene.scratch,
                    scene,
                );
                hold_high(&drawing.scene.scratch, &race.course.numbers.one);
                hold_low(&drawing.scene.scratch, &race.course.numbers.zero);
                scale_steering(&race.shared, scene);
            }
        }
    }
    // 0x43CFB7
    if race.shared.car.parts_state.get() == 1 {
        let k = if kind == 1 {
            &race.shared.physics_tables.grip_rear_drive[0]
        } else {
            &race.shared.physics_tables.grip_rear_drive[1]
        };
        race.shared.car.grip_forces[0]
            .set_bits(x87::mul(race.shared.car.grip_forces[0].bits(), k.bits()));
    }
    race.shared.car.grip_forces[3].set_bits(0);
    if kind == 0 {
        ground_speed_into_scratch(&race.shared, scene);
        hold_high(
            &drawing.scene.scratch,
            &race.shared.physics_tables.grip_ramp,
        );
        lean_force(race, steer_index(&race.shared) as usize, scene);
    }
    // 0x43D08E: the engine braking.
    let braking_word = if kind != 0 {
        braking_count_geared(race, scene);
        ((race
            .shared
            .car
            .brake
            .get()
            .wrapping_sub(race.shared.car.throttle.get()) as i32
            >> 1) as u32)
            .wrapping_add(0x80)
            .wrapping_add(drawing.scene.scratch.get())
            .wrapping_add(race.shared.car.handbrake.get())
    } else {
        braking_count_neutral(&race.shared, scene);
        0x100u32
            .wrapping_sub(race.shared.car.throttle.get())
            .wrapping_add(drawing.scene.scratch.get())
            .wrapping_add(race.shared.car.handbrake.get())
    };
    race.shared.physics_tables.grip_drive_word.set(braking_word);
    engine_braking_force(race);
    if product_below(
        race.shared.car.grip_forces[1].bits(),
        race.shared.car.grip_forces[0].bits(),
        race.course.numbers.zero.bits(),
    ) {
        race.shared.car.grip_forces[1].set_bits(x87::mul(
            race.shared.car.grip_forces[1].bits(),
            race.shared.physics_tables.side_grip_share.bits(),
        ));
    }
    if race.shared.car.parts_state.get() == u32::MAX {
        let k = if kind == 1 {
            &race.shared.physics_tables.grip_front_drive[0]
        } else {
            &race.shared.physics_tables.grip_front_drive[1]
        };
        race.shared.car.grip_forces[1]
            .set_bits(x87::mul(race.shared.car.grip_forces[1].bits(), k.bits()));
    }
    race.shared.car.grip_forces[2].set_bits(0);
    if kind == 1 {
        abs_speed_into_work(&race.shared, scene);
        hold_high(
            &drawing.scene.work_float_a,
            &race.shared.physics_tables.grip_slip_limit,
        );
        yaw_share_from_limit(&race.shared, scene);
        drawing
            .scene
            .scratch
            .set(x87::loaded(race.shared.car.drive_slip_speed.bits()));
        // Held to ±`flt_613E80`, the limit negated in place between and put back.
        hold_high(
            &drawing.scene.scratch,
            &race.shared.physics_tables.grip_force_limit,
        );
        negate(&race.shared.physics_tables.grip_force_limit);
        hold_low(
            &drawing.scene.scratch,
            &race.shared.physics_tables.grip_force_limit,
        );
        negate(&race.shared.physics_tables.grip_force_limit);
        divide_into_scratch(
            &drawing.scene.scratch,
            &race.shared.physics_tables.grip_force_limit,
            scene,
        );
        yaw_force(race, steer_index(&race.shared) as usize, scene);
    }
    // 0x43D2C9: the sum, and the yaw twist on the rear wheels.
    sum_grip_forces(&race.shared);
    race.shared.physics_tables.forces[19].force[0].set_bits(0);
    race.shared.physics_tables.forces[20].force[0].set_bits(0);
    if race.shared.car.all_wheels_off.get() != 0 {
        return;
    }
    race.shared
        .car
        .eased_yaw_spin
        .set_bits(x87::loaded(race.shared.car.spin[1].bits()));
    let old = race.shared.car.eased_yaw_spin.bits();
    race.course
        .numbers
        .grip_change
        .set_bits(x87::sub(old, race.shared.car.grip_force_sum.bits()));
    ease_zero(
        &race.course.numbers.grip_change,
        &race.shared.physics_tables.grip_ease_step,
    );
    ease_yaw_twist(race, old, scene);
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A1E8] = ([bf + e * 4 + 0x623AE4] * [rc + 0x31A184]) / [rc + 0x214BA0]`
fn steer_sine_force(shared: &Shared, e: usize) {
    let t0 = ld(ENGINE_TABLES.sines[e].bits());
    let t1 = ld(shared.car.local_velocity[2].bits());
    let t2 = t0 * t1;
    let t3 = ld(shared.physics_tables.spec_grip.bits());
    let t4 = t2 / t3;
    shared.car.grip_forces[0].set_bits(st(t4));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214768] = -((int [rc + 0x31A1CC] / int [rc + 0x22400C]) - [rc + 0x1C5D38])`
fn lever_share_from_timer(race: &Race, scene: &crate::scene::owner::Scene) {
    let t0 = int(race.shared.car.wall_hit_timer.bits() as i32);
    let t1 = int(scene.drawing.scene.scratch.bits() as i32);
    let t2 = t0 / t1;
    let t3 = ld(race.course.numbers.one.bits());
    let t4 = t2 - t3;
    let t5 = -t4;
    race.shared.physics_tables.lever_share.set_bits(st(t5));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214260] = |(((([rc + 0x2142A8] * [rc + 0x213FC8]) * [f + 0]) * [rc + 0x214B74]) / [rc + 0x31A184])|`
fn grip_limit_for(shared: &Shared, f: &Float) {
    let t0 = ld(shared.physics_tables.wheel_grips[4].bits());
    let t1 = ld(shared.physics_tables.gravity.bits());
    let t2 = t0 * t1;
    let t3 = ld(f.bits());
    let t4 = t2 * t3;
    let t5 = ld(shared.physics_tables.grip_scale.bits());
    let t6 = t4 * t5;
    let t7 = ld(shared.car.local_velocity[2].bits());
    let t8 = t6 / t7;
    let t9 = t8.abs();
    shared.physics_tables.grip_limit.set_bits(st(t9));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x22400C] = [rc + o0] / [rc + o1]`
fn divide_into_scratch(
    o0: &impl crate::session::Cell32,
    o1: &impl crate::session::Cell32,
    scene: &crate::scene::owner::Scene,
) {
    let t0 = ld(o0.bits());
    let t1 = ld(o1.bits());
    let t2 = t0 / t1;
    scene.drawing.scene.scratch.set_bits(st(t2));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [rc + 0x31A61C] = [rc + 0x22400C] * int [rc + 0x31A61C]`
fn scale_steering(shared: &Shared, scene: &crate::scene::owner::Scene) {
    let t0 = ld(scene.drawing.scene.scratch.bits());
    let t1 = int(shared.car.steering.bits() as i32);
    let t2 = t0 * t1;
    shared.car.steering.set_bits(to_int(t2) as u32);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x22400C] = sqrt((([rc + 0x31A17C] * [rc + 0x31A17C]) + ([rc + 0x31A184] * [rc + 0x31A184])))`
fn ground_speed_into_scratch(shared: &Shared, scene: &crate::scene::owner::Scene) {
    let t0 = ld(shared.car.local_velocity[0].bits());
    let t1 = t0 * t0;
    let t2 = ld(shared.car.local_velocity[2].bits());
    let t3 = t2 * t2;
    let t4 = t1 + t3;
    let t5 = t4.sqrt();
    scene.drawing.scene.scratch.set_bits(st(t5));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (a long formula: see the instructions in the tests' `asm_reference`)
fn lean_force(race: &Race, e: usize, scene: &crate::scene::owner::Scene) {
    let t0 = ld(race.shared.physics_tables.grip_ramp.bits());
    let t1 = ld(scene.drawing.scene.scratch.bits());
    let t2 = t0 - t1;
    let t3 = ld(race.shared.physics_tables.grip_ramp.bits());
    let t4 = t2 / t3;
    let t5 = ld(ENGINE_TABLES.sines[e].bits());
    let t6 = t4 * t5;
    let t7 = int(race.shared.car.handbrake.bits() as i32);
    let t8 = t6 * t7;
    let t9 = ld(race.course.numbers.k256.bits());
    let t10 = t8 / t9;
    let t11 = ld(race.shared.car.drive_speed_a.bits());
    let t12 = t10 * t11;
    let t13 = ld(race.shared.physics_tables.grip_ramp_share.bits());
    let t14 = t12 * t13;
    let t15 = int(race.shared.car.throttle.bits() as i32);
    let t16 = t14 * t15;
    let t17 = ld(race.course.numbers.k256.bits());
    let t18 = t16 / t17;
    race.shared.car.grip_forces[3].set_bits(st(t18));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [rc + 0x22400C] = ([rc + 0x214BF0] * int [rc + 0x31A614]) * [rc + 0x1C5DB4]`
fn braking_count_geared(race: &Race, scene: &crate::scene::owner::Scene) {
    let t0 = ld(race.shared.physics_tables.handling[1].bits());
    let t1 = int(race.shared.car.brake.bits() as i32);
    let t2 = t0 * t1;
    let t3 = ld(race.course.numbers.one_and_a_half.bits());
    let t4 = t2 * t3;
    scene.drawing.scene.scratch.set_bits(to_int(t4) as u32);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [rc + 0x22400C] = ([rc + 0x214BF0] * int [rc + 0x31A614]) + ([rc + 0x214BF0] * int [rc + 0x31A614])`
fn braking_count_neutral(shared: &Shared, scene: &crate::scene::owner::Scene) {
    let t0 = ld(shared.physics_tables.handling[1].bits());
    let t1 = int(shared.car.brake.bits() as i32);
    let t2 = t0 * t1;
    let t3 = t2 + t2;
    scene.drawing.scene.scratch.set_bits(to_int(t3) as u32);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A1EC] = (-((int [rc + 0x214164] / [rc + 0x1C5E2C]) * [rc + 0x31A1A0]) * [rc + 0x213ECC]) * [rc + 0x214B78]`
fn engine_braking_force(race: &Race) {
    let t0 = int(race.shared.physics_tables.grip_drive_word.bits() as i32);
    let t1 = ld(race.course.numbers.k256.bits());
    let t2 = t0 / t1;
    let t3 = ld(race.shared.car.side_speed_held.bits());
    let t4 = t2 * t3;
    let t5 = -t4;
    let t6 = ld(race.shared.physics_tables.side_force_scale.bits());
    let t7 = t5 * t6;
    let t8 = ld(race.shared.physics_tables.side_grip_scale.bits());
    let t9 = t7 * t8;
    race.shared.car.grip_forces[1].set_bits(st(t9));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x22401C] = |[rc + 0x31A184]|`
fn abs_speed_into_work(shared: &Shared, scene: &crate::scene::owner::Scene) {
    let t0 = ld(shared.car.local_velocity[2].bits());
    let t1 = t0.abs();
    scene.drawing.scene.work_float_a.set_bits(st(t1));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x22401C] = ([rc + 0x213E7C] - [rc + 0x22401C]) / [rc + 0x213E7C]`
fn yaw_share_from_limit(shared: &Shared, scene: &crate::scene::owner::Scene) {
    let drawing = &scene.drawing;
    let t0 = ld(shared.physics_tables.grip_slip_limit.bits());
    let t1 = ld(drawing.scene.work_float_a.bits());
    let t2 = t0 - t1;
    let t3 = ld(shared.physics_tables.grip_slip_limit.bits());
    let t4 = t2 / t3;
    drawing.scene.work_float_a.set_bits(st(t4));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (a long formula: see the instructions in the tests' `asm_reference`)
fn yaw_force(race: &Race, e: usize, scene: &crate::scene::owner::Scene) {
    let drawing = &scene.drawing;
    let t0 = ld(ENGINE_TABLES.sines[e].bits());
    let t1 = ld(race.shared.physics_tables.spec_grip.bits());
    let t2 = t0 / t1;
    let t3 = int(race.shared.car.throttle.bits() as i32);
    let t4 = t2 * t3;
    let t5 = ld(race.course.numbers.k256.bits());
    let t6 = t4 / t5;
    let t7 = ld(drawing.scene.scratch.bits());
    let t8 = t6 * t7;
    let t9 = ld(drawing.scene.work_float_a.bits());
    let t10 = t8 * t9;
    let t11 = ld(race.shared.physics_tables.grip_slide_scale.bits());
    let t12 = t10 * t11;
    race.shared.car.grip_forces[2].set_bits(st(t12));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A1DC] = (([rc + 0x31A1E8] + [rc + 0x31A1EC]) + [rc + 0x31A1F0]) + [rc + 0x31A1F4]`
fn sum_grip_forces(shared: &Shared) {
    let t0 = ld(shared.car.grip_forces[0].bits());
    let t1 = ld(shared.car.grip_forces[1].bits());
    let t2 = t0 + t1;
    let t3 = ld(shared.car.grip_forces[2].bits());
    let t4 = t2 + t3;
    let t5 = ld(shared.car.grip_forces[3].bits());
    let t6 = t4 + t5;
    shared.car.grip_force_sum.set_bits(st(t6));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A1D8] = [rc + 0x1C5E48] + [rc + 0x31A1DC]`
/// `[rc + 0x22400C] = -([o + 0] - [rc + 0x31A1D8])`
/// `[rc + 0x2149D0] = (([rc + 0x22400C] / [rc + 0x21388C]) * [rc + 0x1C5E40]) / [rc + 0x213FAC]`
/// `[rc + 0x2149F0] = -((([rc + 0x22400C] / [rc + 0x21388C]) * [rc + 0x1C5E40]) / [rc + 0x213FAC])`
/// `[rc + 0x2149E4] = 1`
/// `[rc + 0x214A04] = -1`
/// `[rc + 0x2149D0] = [rc + 0x2149D0] / [rc + 0x31ACC8]`
/// `[rc + 0x2149E4] = [rc + 0x2149E4] * [rc + 0x31ACC8]`
/// `[rc + 0x2149F0] = -([rc + 0x2149F0] / [rc + 0x31ACD4])`
/// `[rc + 0x214A04] = -([rc + 0x214A04] * [rc + 0x31ACD4])`
fn ease_yaw_twist(race: &Race, o: u32, scene: &crate::scene::owner::Scene) {
    let drawing = &scene.drawing;
    let t0 = ld(race.course.numbers.grip_change.bits());
    let t1 = ld(race.shared.car.grip_force_sum.bits());
    let t2 = t0 + t1;
    race.shared.car.eased_yaw_spin.set_bits(st(t2));
    let t3 = ld(o);
    let t4 = ld(race.shared.car.eased_yaw_spin.bits());
    let t5 = t3 - t4;
    let t6 = -t5;
    drawing.scene.scratch.set_bits(st(t6));
    let t7 = ld(drawing.scene.scratch.bits());
    let t8 = ld(race.shared.physics_tables.step_time.bits());
    let t9 = t7 / t8;
    let t10 = ld(race.course.numbers.half.bits());
    let t11 = t9 * t10;
    let t12 = ld(race.shared.physics_tables.inertia[1].bits());
    let t13 = t11 / t12;
    race.shared.physics_tables.forces[19].force[0].set_bits(st(t13));
    let t14 = -t13;
    race.shared.physics_tables.forces[20].force[0].set_bits(st(t14));
    let t15 = X(1.0);
    race.shared.physics_tables.forces[19].point[2].set_bits(st(t15));
    let t16 = -t15;
    race.shared.physics_tables.forces[20].point[2].set_bits(st(t16));
    let t17 = ld(race.shared.physics_tables.forces[19].force[0].bits());
    let t18 = ld(race.shared.car.probe_point_z[0].bits());
    let t19 = t17 / t18;
    race.shared.physics_tables.forces[19].force[0].set_bits(st(t19));
    let t20 = ld(race.shared.physics_tables.forces[19].point[2].bits());
    let t21 = ld(race.shared.car.probe_point_z[0].bits());
    let t22 = t20 * t21;
    race.shared.physics_tables.forces[19].point[2].set_bits(st(t22));
    let t23 = ld(race.shared.physics_tables.forces[20].force[0].bits());
    let t24 = ld(race.shared.car.probe_point_z[3].bits());
    let t25 = t23 / t24;
    let t26 = -t25;
    race.shared.physics_tables.forces[20].force[0].set_bits(st(t26));
    let t27 = ld(race.shared.physics_tables.forces[20].point[2].bits());
    let t28 = ld(race.shared.car.probe_point_z[3].bits());
    let t29 = t27 * t28;
    let t30 = -t29;
    race.shared.physics_tables.forces[20].point[2].set_bits(st(t30));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::race::owner::race;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};
    use crate::scene::owner::scene;
    use crate::scene::x87_mem::{HostCell, view};
    use crate::session::Cell32;

    // The original's instructions, one function per run.
    reference!(ref_0(bf, e, rc) "fld dword ptr [{bf} + {e}*4 + 0x623AE4]", "fmul dword ptr [{rc} + 0x31A184]", "fdiv dword ptr [{rc} + 0x214BA0]", "fstp dword ptr [{rc} + 0x31A1E8]");
    reference!(ref_1(rc) "fild dword ptr [{rc} + 0x31A1CC]", "fild dword ptr [{rc} + 0x22400C]", ".byte 0xDE, 0xF9", "fsub dword ptr [{rc} + 0x1C5D38]", ".byte 0xD9, 0xE0", "fstp dword ptr [{rc} + 0x214768]");
    reference!(ref_2(rc, f) "fld dword ptr [{rc} + 0x2142A8]", "fmul dword ptr [{rc} + 0x213FC8]", "fmul dword ptr [{f}]", "fmul dword ptr [{rc} + 0x214B74]", "fdiv dword ptr [{rc} + 0x31A184]", ".byte 0xD9, 0xE1", "fstp dword ptr [{rc} + 0x214260]");
    reference!(ref_3(rc) "fld dword ptr [{rc} + 0x31A1E8]", "fdiv dword ptr [{rc} + 0x22400C]", "fstp dword ptr [{rc} + 0x22400C]");
    reference!(ref_4(rc) "fld dword ptr [{rc} + 0x22400C]", "fimul dword ptr [{rc} + 0x31A61C]", "fistp dword ptr [{rc} + 0x31A61C]");
    reference!(ref_5(rc) "fld dword ptr [{rc} + 0x31A17C]", ".byte 0xD8, 0xC8", "fld dword ptr [{rc} + 0x31A184]", ".byte 0xD8, 0xC8", ".byte 0xDE, 0xC1", ".byte 0xD9, 0xFA", "fstp dword ptr [{rc} + 0x22400C]");
    reference!(ref_6(rc, bf, e) "fld dword ptr [{rc} + 0x213F34]", "fsub dword ptr [{rc} + 0x22400C]", "fdiv dword ptr [{rc} + 0x213F34]", "fmul dword ptr [{bf} + {e}*4 + 0x623AE4]", "fimul dword ptr [{rc} + 0x31A618]", "fdiv dword ptr [{rc} + 0x1C5E2C]", "fmul dword ptr [{rc} + 0x31A190]", "fmul dword ptr [{rc} + 0x213F38]", "fimul dword ptr [{rc} + 0x31A610]", "fdiv dword ptr [{rc} + 0x1C5E2C]", "fstp dword ptr [{rc} + 0x31A1F4]");
    reference!(ref_7(rc) "fld dword ptr [{rc} + 0x214BF0]", "fimul dword ptr [{rc} + 0x31A614]", "fmul dword ptr [{rc} + 0x1C5DB4]", "fistp dword ptr [{rc} + 0x22400C]");
    reference!(ref_8(rc) "fld dword ptr [{rc} + 0x214BF0]", "fimul dword ptr [{rc} + 0x31A614]", ".byte 0xD8, 0xC0", "fistp dword ptr [{rc} + 0x22400C]");
    reference!(ref_9(rc) "fild dword ptr [{rc} + 0x214164]", "fdiv dword ptr [{rc} + 0x1C5E2C]", "fmul dword ptr [{rc} + 0x31A1A0]", ".byte 0xD9, 0xE0", "fmul dword ptr [{rc} + 0x213ECC]", "fmul dword ptr [{rc} + 0x214B78]", "fstp dword ptr [{rc} + 0x31A1EC]");
    reference!(ref_10(rc) "fld dword ptr [{rc} + 0x31A184]", ".byte 0xD9, 0xE1", "fstp dword ptr [{rc} + 0x22401C]");
    reference!(ref_11(rc) "fld dword ptr [{rc} + 0x213E7C]", "fsub dword ptr [{rc} + 0x22401C]", "fdiv dword ptr [{rc} + 0x213E7C]", "fstp dword ptr [{rc} + 0x22401C]");
    reference!(ref_12(rc) "fld dword ptr [{rc} + 0x22400C]", "fdiv dword ptr [{rc} + 0x213E80]", "fstp dword ptr [{rc} + 0x22400C]");
    reference!(ref_13(bf, e, rc) "fld dword ptr [{bf} + {e}*4 + 0x623AE4]", "fdiv dword ptr [{rc} + 0x214BA0]", "fimul dword ptr [{rc} + 0x31A610]", "fdiv dword ptr [{rc} + 0x1C5E2C]", "fmul dword ptr [{rc} + 0x22400C]", "fmul dword ptr [{rc} + 0x22401C]", "fmul dword ptr [{rc} + 0x213E78]", "fstp dword ptr [{rc} + 0x31A1F0]");
    reference!(ref_14(rc) "fld dword ptr [{rc} + 0x31A1E8]", "fadd dword ptr [{rc} + 0x31A1EC]", "fadd dword ptr [{rc} + 0x31A1F0]", "fadd dword ptr [{rc} + 0x31A1F4]", "fstp dword ptr [{rc} + 0x31A1DC]");
    reference!(ref_15(rc, o) "fld dword ptr [{rc} + 0x1C5E48]", "fadd dword ptr [{rc} + 0x31A1DC]", "fstp dword ptr [{rc} + 0x31A1D8]", "fld dword ptr [{o}]", "fsub dword ptr [{rc} + 0x31A1D8]", ".byte 0xD9, 0xE0", "fstp dword ptr [{rc} + 0x22400C]", "fld dword ptr [{rc} + 0x22400C]", "fdiv dword ptr [{rc} + 0x21388C]", "fmul dword ptr [{rc} + 0x1C5E40]", "fdiv dword ptr [{rc} + 0x213FAC]", "fst dword ptr [{rc} + 0x2149D0]", ".byte 0xD9, 0xE0", "fstp dword ptr [{rc} + 0x2149F0]", ".byte 0xD9, 0xE8", "fst dword ptr [{rc} + 0x2149E4]", ".byte 0xD9, 0xE0", "fstp dword ptr [{rc} + 0x214A04]", "fld dword ptr [{rc} + 0x2149D0]", "fdiv dword ptr [{rc} + 0x31ACC8]", "fstp dword ptr [{rc} + 0x2149D0]", "fld dword ptr [{rc} + 0x2149E4]", "fmul dword ptr [{rc} + 0x31ACC8]", "fstp dword ptr [{rc} + 0x2149E4]", "fld dword ptr [{rc} + 0x2149F0]", "fdiv dword ptr [{rc} + 0x31ACD4]", ".byte 0xD9, 0xE0", "fstp dword ptr [{rc} + 0x2149F0]", "fld dword ptr [{rc} + 0x214A04]", "fmul dword ptr [{rc} + 0x31ACD4]", ".byte 0xD9, 0xE0", "fstp dword ptr [{rc} + 0x214A04]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "drive_grip_forces line 39 idx e=0",
                operands: vec![
                    Operand::Buffers {
                        reads: &[(0x623AE4, In::F)],
                        writes: &[],
                    },
                    Operand::Value(0),
                    Operand::RaceBlock {
                        reads: &[(0x214BA0, In::F), (0x31A184, In::F)],
                        writes: &[(0x31A1E8, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1], a[2]) },
                plain: |a| steer_sine_force(&race().shared, a[1]),
            },
            Block {
                name: "drive_grip_forces line 39 idx e=1",
                operands: vec![
                    Operand::Buffers {
                        reads: &[(0x623AE8, In::F)],
                        writes: &[],
                    },
                    Operand::Value(1),
                    Operand::RaceBlock {
                        reads: &[(0x214BA0, In::F), (0x31A184, In::F)],
                        writes: &[(0x31A1E8, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1], a[2]) },
                plain: |a| steer_sine_force(&race().shared, a[1]),
            },
            Block {
                name: "drive_grip_forces line 39 idx e=4",
                operands: vec![
                    Operand::Buffers {
                        reads: &[(0x623AF4, In::F)],
                        writes: &[],
                    },
                    Operand::Value(4),
                    Operand::RaceBlock {
                        reads: &[(0x214BA0, In::F), (0x31A184, In::F)],
                        writes: &[(0x31A1E8, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1], a[2]) },
                plain: |a| steer_sine_force(&race().shared, a[1]),
            },
            Block {
                name: "drive_grip_forces line 54",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x1C5D38, In::F), (0x22400C, In::I), (0x31A1CC, In::I)],
                    writes: &[(0x214768, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0]) },
                plain: |_| lever_share_from_timer(race(), crate::scene::owner::scene()),
            },
            Block {
                name: "drive_grip_forces line 74",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[
                            (0x213FC8, In::F),
                            (0x2142A8, In::F),
                            (0x214B74, In::F),
                            (0x31A184, In::F),
                        ],
                        writes: &[(0x214260, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0], a[1]) },
                plain: |a| grip_limit_for(&race().shared, view(a[1], 0)),
            },
            Block {
                name: "drive_grip_forces line 98",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x22400C, In::F), (0x31A1E8, In::F)],
                    writes: &[(0x22400C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_3(a[0]) },
                plain: |_| {
                    divide_into_scratch(
                        &crate::race::owner::race().shared.car.grip_forces[0],
                        &scene().drawing.scene.scratch,
                        crate::scene::owner::scene(),
                    );
                },
            },
            Block {
                name: "drive_grip_forces line 106",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x22400C, In::F), (0x31A61C, In::I)],
                    writes: &[(0x31A61C, Out::I)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_4(a[0]) },
                plain: |_| scale_steering(&race().shared, crate::scene::owner::scene()),
            },
            Block {
                name: "drive_grip_forces line 122",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x31A17C, In::F), (0x31A184, In::F)],
                    writes: &[(0x22400C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_5(a[0]) },
                plain: |_| ground_speed_into_scratch(&race().shared, crate::scene::owner::scene()),
            },
            Block {
                name: "drive_grip_forces line 134 idx e=0",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[
                            (0x1C5E2C, In::F),
                            (0x213F34, In::F),
                            (0x213F38, In::F),
                            (0x22400C, In::F),
                            (0x31A190, In::F),
                            (0x31A610, In::I),
                            (0x31A618, In::I),
                        ],
                        writes: &[(0x31A1F4, Out::F)],
                    },
                    Operand::Buffers {
                        reads: &[(0x623AE4, In::F)],
                        writes: &[],
                    },
                    Operand::Value(0),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_6(a[0], a[1], a[2]) },
                plain: |a| lean_force(race(), a[2], crate::scene::owner::scene()),
            },
            Block {
                name: "drive_grip_forces line 134 idx e=1",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[
                            (0x1C5E2C, In::F),
                            (0x213F34, In::F),
                            (0x213F38, In::F),
                            (0x22400C, In::F),
                            (0x31A190, In::F),
                            (0x31A610, In::I),
                            (0x31A618, In::I),
                        ],
                        writes: &[(0x31A1F4, Out::F)],
                    },
                    Operand::Buffers {
                        reads: &[(0x623AE8, In::F)],
                        writes: &[],
                    },
                    Operand::Value(1),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_6(a[0], a[1], a[2]) },
                plain: |a| lean_force(race(), a[2], crate::scene::owner::scene()),
            },
            Block {
                name: "drive_grip_forces line 134 idx e=4",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[
                            (0x1C5E2C, In::F),
                            (0x213F34, In::F),
                            (0x213F38, In::F),
                            (0x22400C, In::F),
                            (0x31A190, In::F),
                            (0x31A610, In::I),
                            (0x31A618, In::I),
                        ],
                        writes: &[(0x31A1F4, Out::F)],
                    },
                    Operand::Buffers {
                        reads: &[(0x623AF4, In::F)],
                        writes: &[],
                    },
                    Operand::Value(4),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_6(a[0], a[1], a[2]) },
                plain: |a| lean_force(race(), a[2], crate::scene::owner::scene()),
            },
            Block {
                name: "drive_grip_forces line 151",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x1C5DB4, In::F), (0x214BF0, In::F), (0x31A614, In::I)],
                    writes: &[(0x22400C, Out::I)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_7(a[0]) },
                plain: |_| braking_count_geared(race(), crate::scene::owner::scene()),
            },
            Block {
                name: "drive_grip_forces line 163",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x214BF0, In::F), (0x31A614, In::I)],
                    writes: &[(0x22400C, Out::I)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_8(a[0]) },
                plain: |_| braking_count_neutral(&race().shared, crate::scene::owner::scene()),
            },
            Block {
                name: "drive_grip_forces line 176",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x1C5E2C, In::F),
                        (0x213ECC, In::F),
                        (0x214164, In::I),
                        (0x214B78, In::F),
                        (0x31A1A0, In::F),
                    ],
                    writes: &[(0x31A1EC, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_9(a[0]) },
                plain: |_| engine_braking_force(race()),
            },
            Block {
                name: "drive_grip_forces line 195",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x31A184, In::F)],
                    writes: &[(0x22401C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_10(a[0]) },
                plain: |_| abs_speed_into_work(&race().shared, crate::scene::owner::scene()),
            },
            Block {
                name: "drive_grip_forces line 202",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x213E7C, In::F), (0x22401C, In::F)],
                    writes: &[(0x22401C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_11(a[0]) },
                plain: |_| yaw_share_from_limit(&race().shared, crate::scene::owner::scene()),
            },
            Block {
                name: "drive_grip_forces line 215",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x213E80, In::F), (0x22400C, In::F)],
                    writes: &[(0x22400C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_12(a[0]) },
                plain: |_| {
                    divide_into_scratch(
                        &scene().drawing.scene.scratch,
                        &crate::race::owner::race()
                            .shared
                            .physics_tables
                            .grip_force_limit,
                        crate::scene::owner::scene(),
                    );
                },
            },
            Block {
                name: "drive_grip_forces line 222 idx e=0",
                operands: vec![
                    Operand::Buffers {
                        reads: &[(0x623AE4, In::F)],
                        writes: &[],
                    },
                    Operand::Value(0),
                    Operand::RaceBlock {
                        reads: &[
                            (0x1C5E2C, In::F),
                            (0x213E78, In::F),
                            (0x214BA0, In::F),
                            (0x22400C, In::F),
                            (0x22401C, In::F),
                            (0x31A610, In::I),
                        ],
                        writes: &[(0x31A1F0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_13(a[0], a[1], a[2]) },
                plain: |a| yaw_force(race(), a[1], crate::scene::owner::scene()),
            },
            Block {
                name: "drive_grip_forces line 222 idx e=1",
                operands: vec![
                    Operand::Buffers {
                        reads: &[(0x623AE8, In::F)],
                        writes: &[],
                    },
                    Operand::Value(1),
                    Operand::RaceBlock {
                        reads: &[
                            (0x1C5E2C, In::F),
                            (0x213E78, In::F),
                            (0x214BA0, In::F),
                            (0x22400C, In::F),
                            (0x22401C, In::F),
                            (0x31A610, In::I),
                        ],
                        writes: &[(0x31A1F0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_13(a[0], a[1], a[2]) },
                plain: |a| yaw_force(race(), a[1], crate::scene::owner::scene()),
            },
            Block {
                name: "drive_grip_forces line 222 idx e=4",
                operands: vec![
                    Operand::Buffers {
                        reads: &[(0x623AF4, In::F)],
                        writes: &[],
                    },
                    Operand::Value(4),
                    Operand::RaceBlock {
                        reads: &[
                            (0x1C5E2C, In::F),
                            (0x213E78, In::F),
                            (0x214BA0, In::F),
                            (0x22400C, In::F),
                            (0x22401C, In::F),
                            (0x31A610, In::I),
                        ],
                        writes: &[(0x31A1F0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_13(a[0], a[1], a[2]) },
                plain: |a| yaw_force(race(), a[1], crate::scene::owner::scene()),
            },
            Block {
                name: "drive_grip_forces line 235",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x31A1E8, In::F),
                        (0x31A1EC, In::F),
                        (0x31A1F0, In::F),
                        (0x31A1F4, In::F),
                    ],
                    writes: &[(0x31A1DC, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_14(a[0]) },
                plain: |_| sum_grip_forces(&race().shared),
            },
            Block {
                name: "drive_grip_forces line 253",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[
                            (0x1C5E40, In::F),
                            (0x1C5E48, In::F),
                            (0x21388C, In::F),
                            (0x213FAC, In::F),
                            (0x2149D0, In::F),
                            (0x2149E4, In::F),
                            (0x2149F0, In::F),
                            (0x214A04, In::F),
                            (0x22400C, In::F),
                            (0x31A1D8, In::F),
                            (0x31A1DC, In::F),
                            (0x31ACC8, In::F),
                            (0x31ACD4, In::F),
                        ],
                        writes: &[
                            (0x2149D0, Out::F),
                            (0x2149E4, Out::F),
                            (0x2149F0, Out::F),
                            (0x214A04, Out::F),
                            (0x22400C, Out::F),
                            (0x31A1D8, Out::F),
                        ],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_15(a[0], a[1]) },
                plain: |a| {
                    ease_yaw_twist(race(), HostCell(a[1]).bits(), crate::scene::owner::scene());
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
