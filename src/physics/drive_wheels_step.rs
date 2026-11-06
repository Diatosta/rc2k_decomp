//! `physics::drive_wheels_step` (0x43DFB1): the driven wheels' step: the slip shares from
//! the car-frame wheel speeds (held 0 ..= 1 and turned into a ±1 share), the clutch and
//! the gear deciding the drive force (`flt_614750` 0 in neutral), `physics::cap_drive_speed`;
//! the drive speeds stepped by the drive force less the wheels' drag and held toward the
//! car's speed, `physics::revs_from_wheels` (twice, on the two paths); the slips
//! `flt_614234` / `614238`; `physics::cap_reverse_inputs`, `physics::add_drive_forces`,
//! `physics::ease_low_speed_forces`; and the rest of the wheels' forces into the force
//! records.
//!
//! The flow, the compares, the integer steps and the calls are Rust; each straight x87
//! run is plain Rust over `X` (`math/x87_float.rs`) in the named helpers below, the
//! original's instructions being `asm_reference` in the tests.
//!
//! Caller: `sub_43B635`, every physics step. Refuses (before anything) a gear outside -1
//! ..= 7 (`revs_from_wheels` would; the gear is not written here).

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_float::X;
use crate::math::x87_float::compare;
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::math::x87_run::{abs_compare, abs_of, neg_rounded};
use crate::physics::flow::{c0, ease_zero, hold_high, hold_low};
use crate::physics::gearing::{GearPlaces, revs_from_wheels_at};
use crate::race::owner_course::Course;
use crate::scene::matrix3::extended;
use crate::scene::owner::scene;
use crate::session::{Cell32, Float};

/// |a| below |b|, or one of them not a number (`fld b; fabs; fld a; fabs; fcompp`, C0).
fn abs_below(x: u32, y: u32) -> bool {
    status_below(abs_status([x, y]))
}

/// `fld from; fstp to`.
fn store_loaded(to: &Float, from: &Float) {
    to.set_bits(x87::loaded(from.bits()));
}

/// 0x43E5CF .. 0x43E643 (and the second wheel's at 0x43E68D): the slip `s` (at `slip`)
/// signed by the slip ratio's sign, the drive speed (`drive`) less it x (`dword_614BA8`
/// - 1), and the two force records' parts at `f1`, `f2`.
fn settle_slip(
    course: &Course,
    drive: &Float,
    slip: &Float,
    ratio: &Float,
    f1: &Float,
    f2: &Float,
) {
    slip_ratio(drive, ratio);
    let s = abs_of(slip.bits());
    let signed = if c0(ratio.bits(), course.numbers.zero.bits()) {
        neg_rounded(extended(s))
    } else {
        s
    };
    slip.set_bits(signed);
    apply_slip_to_drive(course, drive, slip, f1, f2);
}

/// The slip's first limit (0x43E58B / 0x43E649): |drive - speed| / `dword_614BA8` into
/// `ratio`, and `slip` the lesser of it and `force` (by `fcompp`).
fn slip_limit(drive: &Float, ratio: &Float, force: &Float, slip: &Float) {
    slip_ratio_size(drive, ratio);
    let pick = if c0(ratio.bits(), force.bits()) {
        ratio
    } else {
        force
    };
    store_loaded(slip, pick);
}

/// The x87 status word after a compare: the first operand is below the second, or one is
/// not a number (C0).
fn status_below(status: u16) -> bool {
    status & 0x100 != 0
}

/// The x87 status word after a compare: the first operand is above the second (neither
/// C0 nor C3 set).
fn status_above(status: u16) -> bool {
    status & 0x4100 == 0
}

/// The steps.
fn steps(course: &Course) -> Result<(), Refused> {
    let drawing = &scene().drawing;
    let gear = crate::race::owner::race().shared.car.gear.get();
    one_less_abs(
        course,
        &crate::race::owner::race().shared.car.crash_matrix[1],
        &crate::race::owner::race()
            .shared
            .physics_tables
            .drive_shares[0],
    );
    hold_low(
        &crate::race::owner::race()
            .shared
            .physics_tables
            .drive_shares[0],
        &course.numbers.tenth,
    );
    hold_high(
        &crate::race::owner::race()
            .shared
            .physics_tables
            .drive_shares[0],
        &course.numbers.one,
    );
    one_less_abs(
        course,
        &crate::race::owner::race().shared.car.crash_matrix[9],
        &crate::race::owner::race()
            .shared
            .physics_tables
            .drive_shares[1],
    );
    hold_low(
        &crate::race::owner::race()
            .shared
            .physics_tables
            .drive_shares[1],
        &course.numbers.tenth,
    );
    hold_high(
        &crate::race::owner::race()
            .shared
            .physics_tables
            .drive_shares[1],
        &course.numbers.one,
    );
    signed_share(course);
    hold_low(
        &crate::race::owner::race()
            .shared
            .physics_tables
            .drive_shares[2],
        &course.numbers.zero,
    );
    hold_high(
        &crate::race::owner::race()
            .shared
            .physics_tables
            .drive_shares[2],
        &course.numbers.one,
    );
    if crate::race::owner::race()
        .shared
        .car
        .driven_wheels_off
        .get()
        == 0
    {
        if gear == 0 {
            crate::race::owner::race()
                .shared
                .physics_tables
                .drive_torque
                .set_bits(0);
        }
        crate::physics::cap_drive_speed::cap_drive_speed(course);
        crate::race::owner::race()
            .shared
            .physics_tables
            .wheels_driven
            .set(0);
        // the grips at +0 or, braking, +0x14: `wheel_grips_b`
        let mut grips = &crate::race::owner::race().shared.physics_tables.wheel_grips;
        if crate::race::owner::race().shared.car.brake.get() != 0 {
            grips = &crate::race::owner::race()
                .shared
                .physics_tables
                .wheel_grips_b;
            crate::race::owner::race()
                .shared
                .physics_tables
                .wheels_driven
                .set(1);
        }
        drive_forces_from_grips(grips);
        drawing.scene.scratch.set(
            crate::race::owner::race()
                .shared
                .car
                .throttle
                .get()
                .wrapping_add(0x100),
        );
        throttle_drive_limit(course);
        hold_high(
            &crate::race::owner::race()
                .shared
                .physics_tables
                .drive_limits[1],
            &drawing.scene.scratch,
        );
        let pick = if abs_below(
            crate::race::owner::race()
                .shared
                .physics_tables
                .drive_limits[1]
                .bits(),
            crate::race::owner::race()
                .shared
                .physics_tables
                .drive_limits[0]
                .bits(),
        ) {
            &crate::race::owner::race()
                .shared
                .physics_tables
                .drive_limits[0]
        } else {
            &crate::race::owner::race()
                .shared
                .physics_tables
                .drive_limits[1]
        };
        store_loaded(
            &crate::race::owner::race()
                .shared
                .physics_tables
                .drive_limits[2],
            pick,
        );
        let kind = crate::race::owner::race().shared.car.drive_kind.get();
        if gear != 0 && kind == 1 {
            push_both_drive_speeds(course);
        }
        if kind == 0 {
            push_drive_speed_a();
        }
        revs(course)?;
        // 0x43E2C2: the wheels' drag on each drive speed.
        brake_step_for_a(course);
        ease_zero(
            &crate::race::owner::race().shared.car.drive_speed_a,
            &crate::race::owner::race()
                .shared
                .physics_tables
                .wheel_speed_step,
        );
        brake_step_for_b(course);
        ease_zero(
            &crate::race::owner::race().shared.car.drive_speed_b,
            &crate::race::owner::race()
                .shared
                .physics_tables
                .wheel_speed_step,
        );
        handbrake_step_for_b(course);
        ease_zero(
            &crate::race::owner::race().shared.car.drive_speed_b,
            &crate::race::owner::race()
                .shared
                .physics_tables
                .wheel_speed_step,
        );
        if crate::race::owner::race().shared.car.handbrake.get() != 0 {
            let status = handbrake_stop_status([
                crate::race::owner::race().shared.car.drive_speed_a.bits(),
                crate::race::owner::race().shared.car.drive_speed_b.bits(),
                course.numbers.one.bits(),
            ]);
            if status_below(status) {
                crate::race::owner::race()
                    .shared
                    .car
                    .drive_speed_a
                    .set_bits(0);
                crate::race::owner::race()
                    .shared
                    .car
                    .drive_speed_b
                    .set_bits(0);
            }
        }
        revs(course)?;
        slip_gaps();
        crate::physics::cap_reverse_inputs::cap_reverse_inputs(course);
        crate::physics::add_drive_forces::add_drive_forces();
        crate::physics::ease_low_speed_forces::ease_low_speed_forces();

        scale_axle_forces();
        if crate::race::owner::race().shared.car.parts_state.get() != 0 {
            crate::race::owner::race()
                .shared
                .physics_tables
                .drive_forces[0]
                .set_bits(x87::mul(
                    crate::race::owner::race()
                        .shared
                        .physics_tables
                        .drive_forces[0]
                        .bits(),
                    course.numbers.half.bits(),
                ));
            crate::race::owner::race()
                .shared
                .physics_tables
                .drive_forces[1]
                .set_bits(x87::mul(
                    crate::race::owner::race()
                        .shared
                        .physics_tables
                        .drive_forces[1]
                        .bits(),
                    course.numbers.half.bits(),
                ));
        }
        slip_limit(
            &crate::race::owner::race().shared.car.drive_speed_a,
            &crate::race::owner::race()
                .shared
                .physics_tables
                .axle_slip_limits[0],
            &crate::race::owner::race()
                .shared
                .physics_tables
                .drive_forces[0],
            &crate::race::owner::race().shared.physics_tables.axle_slips[0],
        );
        settle_slip(
            course,
            &crate::race::owner::race().shared.car.drive_speed_a,
            &crate::race::owner::race().shared.physics_tables.axle_slips[0],
            &crate::race::owner::race()
                .shared
                .physics_tables
                .axle_slip_limits[0],
            &crate::race::owner::race().shared.physics_tables.forces[15].force[2],
            &crate::race::owner::race().shared.physics_tables.forces[16].force[2],
        );
        slip_limit(
            &crate::race::owner::race().shared.car.drive_speed_b,
            &crate::race::owner::race()
                .shared
                .physics_tables
                .axle_slip_limits[1],
            &crate::race::owner::race()
                .shared
                .physics_tables
                .drive_forces[1],
            &crate::race::owner::race().shared.physics_tables.axle_slips[1],
        );
        settle_slip(
            course,
            &crate::race::owner::race().shared.car.drive_speed_b,
            &crate::race::owner::race().shared.physics_tables.axle_slips[1],
            &crate::race::owner::race()
                .shared
                .physics_tables
                .axle_slip_limits[1],
            &crate::race::owner::race().shared.physics_tables.forces[17].force[2],
            &crate::race::owner::race().shared.physics_tables.forces[18].force[2],
        );
        // 0x43E707: the car-frame side speed eased by the side grip.
        drawing
            .scene
            .scratch
            .set(crate::race::owner::race().shared.car.local_velocity[0].bits());
        ease_zero(
            &drawing.scene.scratch,
            &crate::race::owner::race()
                .shared
                .physics_tables
                .drive_limits[2],
        );
        side_slip_forces(course);
        store_loaded(
            &crate::race::owner::race().shared.physics_tables.slip_limit,
            &crate::race::owner::race()
                .shared
                .physics_tables
                .drive_limits[0],
        );
        store_loaded(
            &crate::race::owner::race()
                .shared
                .physics_tables
                .wheel_spin_push[0],
            &crate::race::owner::race().shared.physics_tables.handling[4],
        );
        let status = abs_compare(
            crate::race::owner::race().shared.physics_tables.drive_slip[0].bits(),
            crate::race::owner::race()
                .shared
                .physics_tables
                .slip_limit
                .bits(),
        );
        if status_above(status) {
            slip_side_share();
        }
        spin_push_b();
        crate::race::owner::race().shared.physics_tables.forces[10].force[0].set_bits(0);
        crate::race::owner::race().shared.physics_tables.forces[10].force[2].set_bits(0);
        // 0x43E855: the rolling drag against the motion.
        let status = speed_over_ground();
        if status_above(status) {
            rolling_drag_forces();
        }
        if crate::race::owner::race().shared.vars.run_kind.get() != 1
            && (crate::race::owner::race()
                .shared
                .car
                .run_ticks
                .get()
                .wrapping_sub(0x320) as i32)
                < 0
        {
            crate::race::owner::race()
                .shared
                .car
                .drive_speed_a
                .set_bits(0);
            crate::race::owner::race()
                .shared
                .car
                .drive_speed_b
                .set_bits(0);
        }
        share_wheel_forces(course);
        crate::race::owner::race().shared.physics_tables.forces[19].force[0].set_bits(0);
        crate::race::owner::race().shared.physics_tables.forces[20].force[0].set_bits(0);
    }
    // 0x43E957: the wheels' slip figures.
    slip_figure_a();
    hold_low(&drawing.scene.work_float_a, &course.numbers.zero);
    slip_figure_b();
    hold_low(&drawing.scene.work_float_b, &course.numbers.zero);
    slip_lengths();
    Ok(())
}

/// `revs_from_wheels` (the gear checked before; the ebx it leaves is read by no one).
fn revs(course: &Course) -> Result<(), Refused> {
    { revs_from_wheels_at(course, &GearPlaces::game()) }?;
    Ok(())
}

/// The Rust version. The eax and ebx it leaves are read by none of its callers.
pub fn drive_wheels_step(course: &Course) -> Result<(), Refused> {
    let gear = crate::race::owner::race().shared.car.gear.get();
    if gear != 0 && !(-1..=7).contains(&(gear as i32)) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    steps(course)
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[t + 0] = -(|[v + 0]| - [rc + 0x1C5D38])`
fn one_less_abs(course: &Course, v: &dyn Cell32, t: &dyn Cell32) {
    let t0 = ld(v.bits());
    let t1 = t0.abs();
    let t2 = ld(course.numbers.one.bits());
    let t3 = t1 - t2;
    let t4 = -t3;
    t.set_bits(st(t4));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `status [s + 0] = status`
fn abs_status(v: [u32; 2]) -> u16 {
    let t0 = ld(v[1]);
    let t1 = t0.abs();
    let t2 = ld(v[0]);
    let t3 = t2.abs();
    compare(t3, t1)
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[r + 0] = ([d + 0] - [rc + 0x31A184]) / [rc + 0x214BA8]`
fn slip_ratio(d: &dyn Cell32, r: &dyn Cell32) {
    let t0 = ld(d.bits());
    let t1 = ld(crate::race::owner::race().shared.car.local_velocity[2].bits());
    let t2 = t0 - t1;
    let t3 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .spec_wheel
        .bits());
    let t4 = t2 / t3;
    r.set_bits(st(t4));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[d + 0] = [d + 0] - (([rc + 0x214BA8] - [rc + 0x1C5D38]) * [s + 0])`
/// `[f1 + 0] = ([s + 0] / [rc + 0x21388C]) * [rc + 0x1C5E40]`
/// `[f2 + 0] = ([s + 0] / [rc + 0x21388C]) * [rc + 0x1C5E40]`
fn apply_slip_to_drive(
    course: &Course,
    d: &dyn Cell32,
    s: &dyn Cell32,
    f1: &dyn Cell32,
    f2: &dyn Cell32,
) {
    let t0 = ld(d.bits());
    let t1 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .spec_wheel
        .bits());
    let t2 = ld(course.numbers.one.bits());
    let t3 = t1 - t2;
    let t4 = ld(s.bits());
    let t5 = t3 * t4;
    let t6 = t0 - t5;
    d.set_bits(st(t6));
    let t7 = ld(s.bits());
    let t8 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .step_time
        .bits());
    let t9 = t7 / t8;
    let t10 = ld(course.numbers.half.bits());
    let t11 = t9 * t10;
    f1.set_bits(st(t11));
    f2.set_bits(st(t11));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[r + 0] = |(([d + 0] - [rc + 0x31A184]) / [rc + 0x214BA8])|`
fn slip_ratio_size(d: &dyn Cell32, r: &dyn Cell32) {
    let t0 = ld(d.bits());
    let t1 = ld(crate::race::owner::race().shared.car.local_velocity[2].bits());
    let t2 = t0 - t1;
    let t3 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .spec_wheel
        .bits());
    let t4 = t2 / t3;
    let t5 = t4.abs();
    r.set_bits(st(t5));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2140B8] = ([rc + 0x2140B4] + [rc + 0x2140B4]) - [rc + 0x1C5D38]`
fn signed_share(course: &Course) {
    let t0 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .drive_shares[1]
        .bits());
    let t1 = t0 + t0;
    let t2 = ld(course.numbers.one.bits());
    let t3 = t1 - t2;
    crate::race::owner::race()
        .shared
        .physics_tables
        .drive_shares[2]
        .set_bits(st(t3));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (a long formula: see the instructions in the tests' `asm_reference`)
/// (a long formula: see the instructions in the tests' `asm_reference`)
/// (a long formula: see the instructions in the tests' `asm_reference`)
fn drive_forces_from_grips(grips: &[Float; 5]) {
    let t0 = ld(grips[0].bits());
    let t1 = ld(crate::race::owner::race().shared.physics_tables.forces[0].force[1].bits());
    let t2 = t0 * t1;
    let t3 = ld(grips[1].bits());
    let t4 = ld(crate::race::owner::race().shared.physics_tables.forces[1].force[1].bits());
    let t5 = t3 * t4;
    let t6 = t2 + t5;
    let t7 = -t6;
    let t8 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .step_time
        .bits());
    let t9 = t7 * t8;
    let t10 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .weight
        .bits());
    let t11 = t9 * t10;
    let t12 = ld(crate::race::owner::race().shared.car.drive_efficiency[0].bits());
    let t13 = t11 * t12;
    crate::race::owner::race()
        .shared
        .physics_tables
        .drive_forces[0]
        .set_bits(st(t13));
    let t14 = ld(grips[2].bits());
    let t15 = ld(crate::race::owner::race().shared.physics_tables.forces[2].force[1].bits());
    let t16 = t14 * t15;
    let t17 = ld(grips[3].bits());
    let t18 = ld(crate::race::owner::race().shared.physics_tables.forces[3].force[1].bits());
    let t19 = t17 * t18;
    let t20 = t16 + t19;
    let t21 = -t20;
    let t22 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .step_time
        .bits());
    let t23 = t21 * t22;
    let t24 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .weight
        .bits());
    let t25 = t23 * t24;
    let t26 = ld(crate::race::owner::race().shared.car.drive_efficiency[1].bits());
    let t27 = t25 * t26;
    crate::race::owner::race()
        .shared
        .physics_tables
        .drive_forces[1]
        .set_bits(st(t27));
    let t28 = ld(crate::race::owner::race().shared.physics_tables.wheel_loads[0].bits());
    let t29 = ld(crate::race::owner::race().shared.physics_tables.wheel_loads[1].bits());
    let t30 = t28 + t29;
    let t31 = ld(crate::race::owner::race().shared.physics_tables.wheel_loads[2].bits());
    let t32 = t30 + t31;
    let t33 = ld(crate::race::owner::race().shared.physics_tables.wheel_loads[3].bits());
    let t34 = t32 + t33;
    let t35 = ld(crate::race::owner::race().shared.physics_tables.forces[0].force[1].bits());
    let t36 = t34 * t35;
    let t37 = -t36;
    let t38 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .step_time
        .bits());
    let t39 = t37 * t38;
    let t40 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .weight
        .bits());
    let t41 = t39 * t40;
    crate::race::owner::race()
        .shared
        .physics_tables
        .drive_limits[0]
        .set_bits(st(t41));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214240] = ((int [rc + 0x22400C] / [rc + 0x1C5E2C]) * |([rc + 0x31A17C] * [rc + 0x214268])|) * [rc + 0x214170]`
/// `[rc + 0x22400C] = [rc + 0x21423C] * [rc + 0x213EC4]`
fn throttle_drive_limit(course: &Course) {
    let drawing = &scene().drawing;
    let t0 = int(drawing.scene.scratch.bits() as i32);
    let t1 = ld(course.numbers.k256.bits());
    let t2 = t0 / t1;
    let t3 = ld(crate::race::owner::race().shared.car.local_velocity[0].bits());
    let t4 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .wheel_load
        .bits());
    let t5 = t3 * t4;
    let t6 = t5.abs();
    let t7 = t2 * t6;
    let t8 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .weight
        .bits());
    let t9 = t7 * t8;
    crate::race::owner::race()
        .shared
        .physics_tables
        .drive_limits[1]
        .set_bits(st(t9));
    let t10 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .drive_limits[0]
        .bits());
    let t11 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .drive_speed_scale
        .bits());
    let t12 = t10 * t11;
    drawing.scene.scratch.set_bits(st(t12));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A190] = ((([rc + 0x214750] * [rc + 0x1C5E40]) * [rc + 0x21388C]) * [rc + 0x214BA8]) + [rc + 0x31A190]`
/// `[rc + 0x31A194] = ((([rc + 0x214750] * [rc + 0x1C5E40]) * [rc + 0x21388C]) * [rc + 0x214BA8]) + [rc + 0x31A194]`
fn push_both_drive_speeds(course: &Course) {
    let t0 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .drive_torque
        .bits());
    let t1 = ld(course.numbers.half.bits());
    let t2 = t0 * t1;
    let t3 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .step_time
        .bits());
    let t4 = t2 * t3;
    let t5 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .spec_wheel
        .bits());
    let t6 = t4 * t5;
    let t7 = ld(crate::race::owner::race().shared.car.drive_speed_a.bits());
    let t8 = t6 + t7;
    crate::race::owner::race()
        .shared
        .car
        .drive_speed_a
        .set_bits(st(t8));
    let t9 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .drive_torque
        .bits());
    let t10 = ld(course.numbers.half.bits());
    let t11 = t9 * t10;
    let t12 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .step_time
        .bits());
    let t13 = t11 * t12;
    let t14 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .spec_wheel
        .bits());
    let t15 = t13 * t14;
    let t16 = ld(crate::race::owner::race().shared.car.drive_speed_b.bits());
    let t17 = t15 + t16;
    crate::race::owner::race()
        .shared
        .car
        .drive_speed_b
        .set_bits(st(t17));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A190] = (([rc + 0x214750] * [rc + 0x21388C]) * [rc + 0x214BA8]) + [rc + 0x31A190]`
fn push_drive_speed_a() {
    let t0 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .drive_torque
        .bits());
    let t1 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .step_time
        .bits());
    let t2 = t0 * t1;
    let t3 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .spec_wheel
        .bits());
    let t4 = t2 * t3;
    let t5 = ld(crate::race::owner::race().shared.car.drive_speed_a.bits());
    let t6 = t4 + t5;
    crate::race::owner::race()
        .shared
        .car
        .drive_speed_a
        .set_bits(st(t6));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (a long formula: see the instructions in the tests' `asm_reference`)
fn brake_step_for_a(course: &Course) {
    let t0 = X(1.0);
    let t1 = ld(crate::race::owner::race().shared.physics_tables.handling[1].bits());
    let t2 = t0 - t1;
    let t3 = ld(crate::race::owner::race().shared.physics_tables.spec_drive[1].bits());
    let t4 = t2 * t3;
    let t5 = int(crate::race::owner::race().shared.car.brake.bits() as i32);
    let t6 = t4 * t5;
    let t7 = ld(course.numbers.k256.bits());
    let t8 = t6 / t7;
    let t9 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .step_time
        .bits());
    let t10 = t8 * t9;
    let t11 = ld(crate::race::owner::race().shared.car.brake_efficiency[0].bits());
    let t12 = t10 * t11;
    let t13 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .spec_wheel
        .bits());
    let t14 = t12 * t13;
    crate::race::owner::race()
        .shared
        .physics_tables
        .wheel_speed_step
        .set_bits(st(t14));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (a long formula: see the instructions in the tests' `asm_reference`)
fn brake_step_for_b(course: &Course) {
    let t0 = ld(crate::race::owner::race().shared.physics_tables.handling[1].bits());
    let t1 = ld(crate::race::owner::race().shared.physics_tables.spec_drive[1].bits());
    let t2 = t0 * t1;
    let t3 = int(crate::race::owner::race().shared.car.brake.bits() as i32);
    let t4 = t2 * t3;
    let t5 = ld(course.numbers.k256.bits());
    let t6 = t4 / t5;
    let t7 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .step_time
        .bits());
    let t8 = t6 * t7;
    let t9 = ld(crate::race::owner::race().shared.car.brake_efficiency[1].bits());
    let t10 = t8 * t9;
    let t11 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .spec_wheel
        .bits());
    let t12 = t10 * t11;
    crate::race::owner::race()
        .shared
        .physics_tables
        .wheel_speed_step
        .set_bits(st(t12));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x21422C] = (((([rc + 0x214BAC] * int [rc + 0x31A618]) / [rc + 0x1C5E2C]) * [rc + 0x21388C]) * [rc + 0x214BA8]) * [rc + 0x214170]`
fn handbrake_step_for_b(course: &Course) {
    let t0 = ld(crate::race::owner::race().shared.physics_tables.spec_drive[0].bits());
    let t1 = int(crate::race::owner::race().shared.car.handbrake.bits() as i32);
    let t2 = t0 * t1;
    let t3 = ld(course.numbers.k256.bits());
    let t4 = t2 / t3;
    let t5 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .step_time
        .bits());
    let t6 = t4 * t5;
    let t7 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .spec_wheel
        .bits());
    let t8 = t6 * t7;
    let t9 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .weight
        .bits());
    let t10 = t8 * t9;
    crate::race::owner::race()
        .shared
        .physics_tables
        .wheel_speed_step
        .set_bits(st(t10));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `status [s + 0] = status`
fn handbrake_stop_status(v: [u32; 3]) -> u16 {
    let t0 = ld(v[0]);
    let t1 = ld(v[1]);
    let t2 = t0 + t1;
    let t3 = t2.abs();
    let t4 = ld(v[2]);
    compare(t3, t4)
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214238] = |([rc + 0x31A194] - [rc + 0x31A184])|`
/// `[rc + 0x214234] = |([rc + 0x31A190] - [rc + 0x31A184])|`
fn slip_gaps() {
    let t0 = ld(crate::race::owner::race().shared.car.drive_speed_b.bits());
    let t1 = ld(crate::race::owner::race().shared.car.local_velocity[2].bits());
    let t2 = t0 - t1;
    let t3 = t2.abs();
    crate::race::owner::race().shared.physics_tables.wheel_slips[1].set_bits(st(t3));
    let t4 = ld(crate::race::owner::race().shared.car.drive_speed_a.bits());
    let t5 = ld(crate::race::owner::race().shared.car.local_velocity[2].bits());
    let t6 = t4 - t5;
    let t7 = t6.abs();
    crate::race::owner::race().shared.physics_tables.wheel_slips[0].set_bits(st(t7));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214244] = [rc + 0x214244] * [rc + 0x2140B0]`
/// `[rc + 0x214248] = [rc + 0x214248] * [rc + 0x2140B4]`
/// `[rc + 0x21424C] = [rc + 0x21424C] * [rc + 0x2140B4]`
fn scale_axle_forces() {
    let t0 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .drive_limits[2]
        .bits());
    let t1 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .drive_shares[0]
        .bits());
    let t2 = t0 * t1;
    crate::race::owner::race()
        .shared
        .physics_tables
        .drive_limits[2]
        .set_bits(st(t2));
    let t3 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .drive_forces[0]
        .bits());
    let t4 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .drive_shares[1]
        .bits());
    let t5 = t3 * t4;
    crate::race::owner::race()
        .shared
        .physics_tables
        .drive_forces[0]
        .set_bits(st(t5));
    let t6 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .drive_forces[1]
        .bits());
    let t7 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .drive_shares[1]
        .bits());
    let t8 = t6 * t7;
    crate::race::owner::race()
        .shared
        .physics_tables
        .drive_forces[1]
        .set_bits(st(t8));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214250] = [rc + 0x22400C] - [rc + 0x31A17C]`
/// `[rc + 0x2148D0] = (([rc + 0x22400C] - [rc + 0x31A17C]) / [rc + 0x21388C]) * [rc + 0x1C5DA4]`
/// `[rc + 0x2148F0] = (([rc + 0x22400C] - [rc + 0x31A17C]) / [rc + 0x21388C]) * [rc + 0x1C5DA4]`
/// `[rc + 0x214910] = (([rc + 0x22400C] - [rc + 0x31A17C]) / [rc + 0x21388C]) * [rc + 0x1C5DA4]`
/// `[rc + 0x214930] = (([rc + 0x22400C] - [rc + 0x31A17C]) / [rc + 0x21388C]) * [rc + 0x1C5DA4]`
fn side_slip_forces(course: &Course) {
    let t0 = ld(scene().drawing.scene.scratch.bits());
    let t1 = ld(crate::race::owner::race().shared.car.local_velocity[0].bits());
    let t2 = t0 - t1;
    crate::race::owner::race().shared.physics_tables.drive_slip[0].set_bits(st(t2));
    let t3 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .step_time
        .bits());
    let t4 = t2 / t3;
    let t5 = ld(course.numbers.quarter.bits());
    let t6 = t4 * t5;
    crate::race::owner::race().shared.physics_tables.forces[11].force[0].set_bits(st(t6));
    crate::race::owner::race().shared.physics_tables.forces[12].force[0].set_bits(st(t6));
    crate::race::owner::race().shared.physics_tables.forces[13].force[0].set_bits(st(t6));
    crate::race::owner::race().shared.physics_tables.forces[14].force[0].set_bits(st(t6));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214254] = ((|[rc + 0x214250]| - [rc + 0x214264]) * [rc + 0x213ED0]) + [rc + 0x214264]`
/// `[rc + 0x2141D4] = |([rc + 0x214254] / [rc + 0x214250])| * [rc + 0x214BFC]`
fn slip_side_share() {
    let t0 = ld(crate::race::owner::race().shared.physics_tables.drive_slip[0].bits());
    let t1 = t0.abs();
    let t2 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .slip_limit
        .bits());
    let t3 = t1 - t2;
    let t4 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .slip_blend
        .bits());
    let t5 = t3 * t4;
    let t6 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .slip_limit
        .bits());
    let t7 = t5 + t6;
    crate::race::owner::race().shared.physics_tables.drive_slip[1].set_bits(st(t7));
    let t8 = ld(crate::race::owner::race().shared.physics_tables.drive_slip[1].bits());
    let t9 = ld(crate::race::owner::race().shared.physics_tables.drive_slip[0].bits());
    let t10 = t8 / t9;
    let t11 = t10.abs();
    let t12 = ld(crate::race::owner::race().shared.physics_tables.handling[4].bits());
    let t13 = t11 * t12;
    crate::race::owner::race()
        .shared
        .physics_tables
        .wheel_spin_push[0]
        .set_bits(st(t13));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2141D8] = [rc + 0x214BFC] * [rc + 0x213ED4]`
fn spin_push_b() {
    let t0 = ld(crate::race::owner::race().shared.physics_tables.handling[4].bits());
    let t1 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .slip_share
        .bits());
    let t2 = t0 * t1;
    crate::race::owner::race()
        .shared
        .physics_tables
        .wheel_spin_push[1]
        .set_bits(st(t2));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x22400C] = sqrt((([rc + 0x31A17C] * [rc + 0x31A17C]) + ([rc + 0x31A184] * [rc + 0x31A184])))`
/// `status [s + 0] = status`
fn speed_over_ground() -> u16 {
    let t0 = ld(crate::race::owner::race().shared.car.local_velocity[0].bits());
    let t1 = t0 * t0;
    let t2 = ld(crate::race::owner::race().shared.car.local_velocity[2].bits());
    let t3 = t2 * t2;
    let t4 = t1 + t3;
    let t5 = t4.sqrt();
    scene().drawing.scene.scratch.set_bits(st(t5));
    let t6 = ld(crate::race::owner::race().shared.physics_tables.wheel_drag[1].bits());
    compare(t5, t6)
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2148B0] = -(([rc + 0x214760] / [rc + 0x22400C]) * [rc + 0x31A17C])`
/// `[rc + 0x2148B8] = -(([rc + 0x214760] / [rc + 0x22400C]) * [rc + 0x31A184])`
fn rolling_drag_forces() {
    let drawing = &scene().drawing;
    let t0 = ld(crate::race::owner::race().shared.physics_tables.wheel_drag[0].bits());
    let t1 = ld(drawing.scene.scratch.bits());
    let t2 = t0 / t1;
    let t3 = ld(crate::race::owner::race().shared.car.local_velocity[0].bits());
    let t4 = t2 * t3;
    let t5 = -t4;
    crate::race::owner::race().shared.physics_tables.forces[10].force[0].set_bits(st(t5));
    let t6 = ld(crate::race::owner::race().shared.physics_tables.wheel_drag[0].bits());
    let t7 = ld(drawing.scene.scratch.bits());
    let t8 = t6 / t7;
    let t9 = ld(crate::race::owner::race().shared.car.local_velocity[2].bits());
    let t10 = t8 * t9;
    let t11 = -t10;
    crate::race::owner::race().shared.physics_tables.forces[10].force[2].set_bits(st(t11));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2148D0] = ([rc + 0x2149D0] * [rc + 0x1C5E40]) + [rc + 0x2148D0]`
/// `[rc + 0x2148F0] = ([rc + 0x2149D0] * [rc + 0x1C5E40]) + [rc + 0x2148F0]`
/// `[rc + 0x214910] = ([rc + 0x2149F0] * [rc + 0x1C5E40]) + [rc + 0x214910]`
/// `[rc + 0x214930] = ([rc + 0x2149F0] * [rc + 0x1C5E40]) + [rc + 0x214930]`
fn share_wheel_forces(course: &Course) {
    let t0 = ld(crate::race::owner::race().shared.physics_tables.forces[19].force[0].bits());
    let t1 = ld(course.numbers.half.bits());
    let t2 = t0 * t1;
    let t3 = ld(crate::race::owner::race().shared.physics_tables.forces[11].force[0].bits());
    let t4 = t2 + t3;
    crate::race::owner::race().shared.physics_tables.forces[11].force[0].set_bits(st(t4));
    let t5 = ld(crate::race::owner::race().shared.physics_tables.forces[19].force[0].bits());
    let t6 = ld(course.numbers.half.bits());
    let t7 = t5 * t6;
    let t8 = ld(crate::race::owner::race().shared.physics_tables.forces[12].force[0].bits());
    let t9 = t7 + t8;
    crate::race::owner::race().shared.physics_tables.forces[12].force[0].set_bits(st(t9));
    let t10 = ld(crate::race::owner::race().shared.physics_tables.forces[20].force[0].bits());
    let t11 = ld(course.numbers.half.bits());
    let t12 = t10 * t11;
    let t13 = ld(crate::race::owner::race().shared.physics_tables.forces[13].force[0].bits());
    let t14 = t12 + t13;
    crate::race::owner::race().shared.physics_tables.forces[13].force[0].set_bits(st(t14));
    let t15 = ld(crate::race::owner::race().shared.physics_tables.forces[20].force[0].bits());
    let t16 = ld(course.numbers.half.bits());
    let t17 = t15 * t16;
    let t18 = ld(crate::race::owner::race().shared.physics_tables.forces[14].force[0].bits());
    let t19 = t17 + t18;
    crate::race::owner::race().shared.physics_tables.forces[14].force[0].set_bits(st(t19));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A158] = [rc + 0x31A190] - [rc + 0x31A184]`
/// `[rc + 0x22401C] = |([rc + 0x31A190] - [rc + 0x31A184])| - [rc + 0x213EDC]`
fn slip_figure_a() {
    let t0 = ld(crate::race::owner::race().shared.car.drive_speed_a.bits());
    let t1 = ld(crate::race::owner::race().shared.car.local_velocity[2].bits());
    let t2 = t0 - t1;
    crate::race::owner::race()
        .shared
        .car
        .drive_slip_speed
        .set_bits(st(t2));
    let t3 = t2.abs();
    let t4 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .wheel_speed_gap
        .bits());
    let t5 = t3 - t4;
    scene().drawing.scene.work_float_a.set_bits(st(t5));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x224020] = |[rc + 0x31A17C]| - [rc + 0x213ED8]`
fn slip_figure_b() {
    let t0 = ld(crate::race::owner::race().shared.car.local_velocity[0].bits());
    let t1 = t0.abs();
    let t2 = ld(crate::race::owner::race()
        .shared
        .physics_tables
        .wheel_speed_margin
        .bits());
    let t3 = t1 - t2;
    scene().drawing.scene.work_float_b.set_bits(st(t3));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A260] = sqrt((([rc + 0x22401C] * [rc + 0x22401C]) + ([rc + 0x224020] * [rc + 0x224020])))`
/// `[rc + 0x224020] = |[rc + 0x31A17C]|`
/// `[rc + 0x22401C] = |([rc + 0x31A190] - [rc + 0x31A184])|`
/// `[rc + 0x31AF18] = sqrt((([rc + 0x22401C] * [rc + 0x22401C]) + ([rc + 0x224020] * [rc + 0x224020])))`
/// `[rc + 0x22401C] = |([rc + 0x31A194] - [rc + 0x31A184])|`
/// `[rc + 0x31AF1C] = sqrt((([rc + 0x22401C] * [rc + 0x22401C]) + ([rc + 0x224020] * [rc + 0x224020])))`
fn slip_lengths() {
    let drawing = &scene().drawing;
    let t0 = ld(drawing.scene.work_float_a.bits());
    let t1 = t0 * t0;
    let t2 = ld(drawing.scene.work_float_b.bits());
    let t3 = t2 * t2;
    let t4 = t1 + t3;
    let t5 = t4.sqrt();
    crate::race::owner::race()
        .shared
        .car
        .slip_speed
        .set_bits(st(t5));
    let t6 = ld(crate::race::owner::race().shared.car.local_velocity[0].bits());
    let t7 = t6.abs();
    drawing.scene.work_float_b.set_bits(st(t7));
    let t8 = ld(crate::race::owner::race().shared.car.drive_speed_a.bits());
    let t9 = ld(crate::race::owner::race().shared.car.local_velocity[2].bits());
    let t10 = t8 - t9;
    let t11 = t10.abs();
    drawing.scene.work_float_a.set_bits(st(t11));
    let t12 = ld(drawing.scene.work_float_a.bits());
    let t13 = t12 * t12;
    let t14 = ld(drawing.scene.work_float_b.bits());
    let t15 = t14 * t14;
    let t16 = t13 + t15;
    let t17 = t16.sqrt();
    crate::race::owner::race().shared.car.skid_grip[0].set_bits(st(t17));
    let t18 = ld(crate::race::owner::race().shared.car.drive_speed_b.bits());
    let t19 = ld(crate::race::owner::race().shared.car.local_velocity[2].bits());
    let t20 = t18 - t19;
    let t21 = t20.abs();
    drawing.scene.work_float_a.set_bits(st(t21));
    let t22 = ld(drawing.scene.work_float_a.bits());
    let t23 = t22 * t22;
    let t24 = ld(drawing.scene.work_float_b.bits());
    let t25 = t24 * t24;
    let t26 = t23 + t25;
    let t27 = t26.sqrt();
    crate::race::owner::race().shared.car.skid_grip[1].set_bits(st(t27));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};
    use crate::scene::x87_mem::{Base, view};

    // The original's instructions, one function per run.
    reference!(ref_0(v, rc, t) "fld dword ptr [{v}]", ".byte 0xD9, 0xE1", "fsub dword ptr [{rc} + 0x1C5D38]", ".byte 0xD9, 0xE0", "fstp dword ptr [{t}]");
    reference!(ref_1(v, s) "fld dword ptr [{v} + 4]", ".byte 0xD9, 0xE1", "fld dword ptr [{v}]", ".byte 0xD9, 0xE1", ".byte 0xDE, 0xD9", "fnstsw word ptr [{s}]");
    reference!(ref_2(d, rc, r) "fld dword ptr [{d}]", "fsub dword ptr [{rc} + 0x31A184]", "fdiv dword ptr [{rc} + 0x214BA8]", "fstp dword ptr [{r}]");
    reference!(ref_3(d, rc, s, f1, f2) "fld dword ptr [{d}]", "fld dword ptr [{rc} + 0x214BA8]", "fsub dword ptr [{rc} + 0x1C5D38]", "fmul dword ptr [{s}]", ".byte 0xDE, 0xE9", "fstp dword ptr [{d}]", "fld dword ptr [{s}]", "fdiv dword ptr [{rc} + 0x21388C]", "fmul dword ptr [{rc} + 0x1C5E40]", "fst dword ptr [{f1}]", "fstp dword ptr [{f2}]");
    reference!(ref_4(d, rc, r) "fld dword ptr [{d}]", "fsub dword ptr [{rc} + 0x31A184]", "fdiv dword ptr [{rc} + 0x214BA8]", ".byte 0xD9, 0xE1", "fstp dword ptr [{r}]");
    reference!(ref_5(rc) "fld dword ptr [{rc} + 0x2140B4]", ".byte 0xD8, 0xC0", "fsub dword ptr [{rc} + 0x1C5D38]", "fstp dword ptr [{rc} + 0x2140B8]");
    reference!(ref_6(rc, e) "fld dword ptr [{rc} + {e} + 0x214298]", "fmul dword ptr [{rc} + 0x214774]", "fld dword ptr [{rc} + {e} + 0x21429C]", "fmul dword ptr [{rc} + 0x214794]", ".byte 0xDE, 0xC1", ".byte 0xD9, 0xE0", "fmul dword ptr [{rc} + 0x21388C]", "fmul dword ptr [{rc} + 0x214170]", "fmul dword ptr [{rc} + 0x31A3D8]", "fstp dword ptr [{rc} + 0x214248]", "fld dword ptr [{rc} + {e} + 0x2142A0]", "fmul dword ptr [{rc} + 0x2147B4]", "fld dword ptr [{rc} + {e} + 0x2142A4]", "fmul dword ptr [{rc} + 0x2147D4]", ".byte 0xDE, 0xC1", ".byte 0xD9, 0xE0", "fmul dword ptr [{rc} + 0x21388C]", "fmul dword ptr [{rc} + 0x214170]", "fmul dword ptr [{rc} + 0x31A3DC]", "fstp dword ptr [{rc} + 0x21424C]", "fld dword ptr [{rc} + 0x214284]", "fadd dword ptr [{rc} + 0x214288]", "fadd dword ptr [{rc} + 0x21428C]", "fadd dword ptr [{rc} + 0x214290]", "fmul dword ptr [{rc} + 0x214774]", ".byte 0xD9, 0xE0", "fmul dword ptr [{rc} + 0x21388C]", "fmul dword ptr [{rc} + 0x214170]", "fstp dword ptr [{rc} + 0x21423C]");
    reference!(ref_7(rc) "fild dword ptr [{rc} + 0x22400C]", "fdiv dword ptr [{rc} + 0x1C5E2C]", "fld dword ptr [{rc} + 0x31A17C]", "fmul dword ptr [{rc} + 0x214268]", ".byte 0xD9, 0xE1", ".byte 0xDE, 0xC9", "fmul dword ptr [{rc} + 0x214170]", "fstp dword ptr [{rc} + 0x214240]", "fld dword ptr [{rc} + 0x21423C]", "fmul dword ptr [{rc} + 0x213EC4]", "fstp dword ptr [{rc} + 0x22400C]");
    reference!(ref_8(rc) "fld dword ptr [{rc} + 0x214750]", "fmul dword ptr [{rc} + 0x1C5E40]", "fmul dword ptr [{rc} + 0x21388C]", "fmul dword ptr [{rc} + 0x214BA8]", "fadd dword ptr [{rc} + 0x31A190]", "fstp dword ptr [{rc} + 0x31A190]", "fld dword ptr [{rc} + 0x214750]", "fmul dword ptr [{rc} + 0x1C5E40]", "fmul dword ptr [{rc} + 0x21388C]", "fmul dword ptr [{rc} + 0x214BA8]", "fadd dword ptr [{rc} + 0x31A194]", "fstp dword ptr [{rc} + 0x31A194]");
    reference!(ref_9(rc) "fld dword ptr [{rc} + 0x214750]", "fmul dword ptr [{rc} + 0x21388C]", "fmul dword ptr [{rc} + 0x214BA8]", "fadd dword ptr [{rc} + 0x31A190]", "fstp dword ptr [{rc} + 0x31A190]");
    reference!(ref_10(rc) ".byte 0xD9, 0xE8", "fsub dword ptr [{rc} + 0x214BF0]", "fmul dword ptr [{rc} + 0x214BB0]", "fild dword ptr [{rc} + 0x31A614]", ".byte 0xDE, 0xC9", "fdiv dword ptr [{rc} + 0x1C5E2C]", "fmul dword ptr [{rc} + 0x21388C]", "fmul dword ptr [{rc} + 0x31A3D0]", "fmul dword ptr [{rc} + 0x214BA8]", "fstp dword ptr [{rc} + 0x21422C]");
    reference!(ref_11(rc) "fld dword ptr [{rc} + 0x214BF0]", "fmul dword ptr [{rc} + 0x214BB0]", "fild dword ptr [{rc} + 0x31A614]", ".byte 0xDE, 0xC9", "fdiv dword ptr [{rc} + 0x1C5E2C]", "fmul dword ptr [{rc} + 0x21388C]", "fmul dword ptr [{rc} + 0x31A3D4]", "fmul dword ptr [{rc} + 0x214BA8]", "fstp dword ptr [{rc} + 0x21422C]");
    reference!(ref_12(rc) "fld dword ptr [{rc} + 0x214BAC]", "fild dword ptr [{rc} + 0x31A618]", ".byte 0xDE, 0xC9", "fdiv dword ptr [{rc} + 0x1C5E2C]", "fmul dword ptr [{rc} + 0x21388C]", "fmul dword ptr [{rc} + 0x214BA8]", "fmul dword ptr [{rc} + 0x214170]", "fstp dword ptr [{rc} + 0x21422C]");
    reference!(ref_13(v, s) "fld dword ptr [{v}]", "fadd dword ptr [{v} + 4]", ".byte 0xD9, 0xE1", "fcomp dword ptr [{v} + 8]", "fnstsw word ptr [{s}]");
    reference!(ref_14(rc) "fld dword ptr [{rc} + 0x31A194]", "fsub dword ptr [{rc} + 0x31A184]", ".byte 0xD9, 0xE1", "fstp dword ptr [{rc} + 0x214238]", "fld dword ptr [{rc} + 0x31A190]", "fsub dword ptr [{rc} + 0x31A184]", ".byte 0xD9, 0xE1", "fstp dword ptr [{rc} + 0x214234]");
    reference!(ref_15(rc) "fld dword ptr [{rc} + 0x214244]", "fmul dword ptr [{rc} + 0x2140B0]", "fstp dword ptr [{rc} + 0x214244]", "fld dword ptr [{rc} + 0x214248]", "fmul dword ptr [{rc} + 0x2140B4]", "fstp dword ptr [{rc} + 0x214248]", "fld dword ptr [{rc} + 0x21424C]", "fmul dword ptr [{rc} + 0x2140B4]", "fstp dword ptr [{rc} + 0x21424C]");
    reference!(ref_16(rc) "fld dword ptr [{rc} + 0x22400C]", "fsub dword ptr [{rc} + 0x31A17C]", "fst dword ptr [{rc} + 0x214250]", "fdiv dword ptr [{rc} + 0x21388C]", "fmul dword ptr [{rc} + 0x1C5DA4]", "fst dword ptr [{rc} + 0x2148D0]", "fst dword ptr [{rc} + 0x2148F0]", "fst dword ptr [{rc} + 0x214910]", "fstp dword ptr [{rc} + 0x214930]");
    reference!(ref_17(rc) "fld dword ptr [{rc} + 0x214250]", ".byte 0xD9, 0xE1", "fsub dword ptr [{rc} + 0x214264]", "fmul dword ptr [{rc} + 0x213ED0]", "fadd dword ptr [{rc} + 0x214264]", "fstp dword ptr [{rc} + 0x214254]", "fld dword ptr [{rc} + 0x214254]", "fdiv dword ptr [{rc} + 0x214250]", ".byte 0xD9, 0xE1", "fmul dword ptr [{rc} + 0x214BFC]", "fstp dword ptr [{rc} + 0x2141D4]");
    reference!(ref_18(rc) "fld dword ptr [{rc} + 0x214BFC]", "fmul dword ptr [{rc} + 0x213ED4]", "fstp dword ptr [{rc} + 0x2141D8]");
    reference!(ref_19(rc, s) "fld dword ptr [{rc} + 0x31A17C]", ".byte 0xD8, 0xC8", "fld dword ptr [{rc} + 0x31A184]", ".byte 0xD8, 0xC8", ".byte 0xDE, 0xC1", ".byte 0xD9, 0xFA", "fst dword ptr [{rc} + 0x22400C]", "fcomp dword ptr [{rc} + 0x214764]", "fnstsw word ptr [{s}]");
    reference!(ref_20(rc) "fld dword ptr [{rc} + 0x214760]", "fdiv dword ptr [{rc} + 0x22400C]", "fmul dword ptr [{rc} + 0x31A17C]", ".byte 0xD9, 0xE0", "fstp dword ptr [{rc} + 0x2148B0]", "fld dword ptr [{rc} + 0x214760]", "fdiv dword ptr [{rc} + 0x22400C]", "fmul dword ptr [{rc} + 0x31A184]", ".byte 0xD9, 0xE0", "fstp dword ptr [{rc} + 0x2148B8]");
    reference!(ref_21(rc) "fld dword ptr [{rc} + 0x2149D0]", "fmul dword ptr [{rc} + 0x1C5E40]", "fadd dword ptr [{rc} + 0x2148D0]", "fstp dword ptr [{rc} + 0x2148D0]", "fld dword ptr [{rc} + 0x2149D0]", "fmul dword ptr [{rc} + 0x1C5E40]", "fadd dword ptr [{rc} + 0x2148F0]", "fstp dword ptr [{rc} + 0x2148F0]", "fld dword ptr [{rc} + 0x2149F0]", "fmul dword ptr [{rc} + 0x1C5E40]", "fadd dword ptr [{rc} + 0x214910]", "fstp dword ptr [{rc} + 0x214910]", "fld dword ptr [{rc} + 0x2149F0]", "fmul dword ptr [{rc} + 0x1C5E40]", "fadd dword ptr [{rc} + 0x214930]", "fstp dword ptr [{rc} + 0x214930]");
    reference!(ref_22(rc) "fld dword ptr [{rc} + 0x31A190]", "fsub dword ptr [{rc} + 0x31A184]", "fst dword ptr [{rc} + 0x31A158]", ".byte 0xD9, 0xE1", "fsub dword ptr [{rc} + 0x213EDC]", "fstp dword ptr [{rc} + 0x22401C]");
    reference!(ref_23(rc) "fld dword ptr [{rc} + 0x31A17C]", ".byte 0xD9, 0xE1", "fsub dword ptr [{rc} + 0x213ED8]", "fstp dword ptr [{rc} + 0x224020]");
    reference!(ref_24(rc) "fld dword ptr [{rc} + 0x22401C]", ".byte 0xD8, 0xC8", "fld dword ptr [{rc} + 0x224020]", ".byte 0xD8, 0xC8", ".byte 0xDE, 0xC1", ".byte 0xD9, 0xFA", "fstp dword ptr [{rc} + 0x31A260]", "fld dword ptr [{rc} + 0x31A17C]", ".byte 0xD9, 0xE1", "fstp dword ptr [{rc} + 0x224020]", "fld dword ptr [{rc} + 0x31A190]", "fsub dword ptr [{rc} + 0x31A184]", ".byte 0xD9, 0xE1", "fstp dword ptr [{rc} + 0x22401C]", "fld dword ptr [{rc} + 0x22401C]", ".byte 0xD8, 0xC8", "fld dword ptr [{rc} + 0x224020]", ".byte 0xD8, 0xC8", ".byte 0xDE, 0xC1", ".byte 0xD9, 0xFA", "fstp dword ptr [{rc} + 0x31AF18]", "fld dword ptr [{rc} + 0x31A194]", "fsub dword ptr [{rc} + 0x31A184]", ".byte 0xD9, 0xE1", "fstp dword ptr [{rc} + 0x22401C]", "fld dword ptr [{rc} + 0x22401C]", ".byte 0xD8, 0xC8", "fld dword ptr [{rc} + 0x224020]", ".byte 0xD8, 0xC8", ".byte 0xDE, 0xC1", ".byte 0xD9, 0xFA", "fstp dword ptr [{rc} + 0x31AF1C]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "drive_wheels_step line 33",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x1C5D38, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1], a[2]) },
                plain: |a| {
                    one_less_abs(
                        &crate::race::owner::race().course,
                        &crate::scene::x87_mem::HostCell(a[0]),
                        &crate::scene::x87_mem::HostCell(a[2]),
                    );
                },
            },
            Block {
                name: "drive_wheels_step line 47",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F), (4, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::H)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1]) },
                plain: |a| a[1].set_word(0, abs_status(*view(a[0], 0))),
            },
            Block {
                name: "drive_wheels_step line 73",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x214BA8, In::F), (0x31A184, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0], a[1], a[2]) },
                plain: |a| {
                    slip_ratio(
                        &crate::scene::x87_mem::HostCell(a[0]),
                        &crate::scene::x87_mem::HostCell(a[2]),
                    );
                },
            },
            Block {
                name: "drive_wheels_step line 87",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[(0, Out::F)],
                    },
                    Operand::RaceBlock {
                        reads: &[
                            (0x1C5D38, In::F),
                            (0x1C5E40, In::F),
                            (0x21388C, In::F),
                            (0x214BA8, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_3(a[0], a[1], a[2], a[3], a[4]) },
                plain: |a| {
                    use crate::scene::x87_mem::HostCell as H;
                    apply_slip_to_drive(
                        &crate::race::owner::race().course,
                        &H(a[0]),
                        &H(a[2]),
                        &H(a[3]),
                        &H(a[4]),
                    );
                },
            },
            Block {
                name: "drive_wheels_step line 106",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x214BA8, In::F), (0x31A184, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_4(a[0], a[1], a[2]) },
                plain: |a| {
                    slip_ratio_size(
                        &crate::scene::x87_mem::HostCell(a[0]),
                        &crate::scene::x87_mem::HostCell(a[2]),
                    );
                },
            },
            Block {
                name: "drive_wheels_step line 127",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x1C5D38, In::F), (0x2140B4, In::F)],
                    writes: &[(0x2140B8, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_5(a[0]) },
                plain: |_| signed_share(&crate::race::owner::race().course),
            },
            Block {
                name: "drive_wheels_step line 147 idx e=0",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[
                            (0x21388C, In::F),
                            (0x214170, In::F),
                            (0x214284, In::F),
                            (0x214288, In::F),
                            (0x21428C, In::F),
                            (0x214290, In::F),
                            (0x214298, In::F),
                            (0x21429C, In::F),
                            (0x2142A0, In::F),
                            (0x2142A4, In::F),
                            (0x214774, In::F),
                            (0x214794, In::F),
                            (0x2147B4, In::F),
                            (0x2147D4, In::F),
                            (0x31A3D8, In::F),
                            (0x31A3DC, In::F),
                        ],
                        writes: &[(0x21423C, Out::F), (0x214248, Out::F), (0x21424C, Out::F)],
                    },
                    Operand::Value(0),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_6(a[0], a[1]) },
                plain: |a| {
                    drive_forces_from_grips(if a[1] == 0 {
                        &crate::race::owner::race().shared.physics_tables.wheel_grips
                    } else {
                        &crate::race::owner::race()
                            .shared
                            .physics_tables
                            .wheel_grips_b
                    });
                },
            },
            Block {
                name: "drive_wheels_step line 147 idx e=0x14 (braking)",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[
                            (0x21388C, In::F),
                            (0x214170, In::F),
                            (0x214284, In::F),
                            (0x214288, In::F),
                            (0x21428C, In::F),
                            (0x214290, In::F),
                            (0x2142AC, In::F),
                            (0x2142B0, In::F),
                            (0x2142B4, In::F),
                            (0x2142B8, In::F),
                            (0x214774, In::F),
                            (0x214794, In::F),
                            (0x2147B4, In::F),
                            (0x2147D4, In::F),
                            (0x31A3D8, In::F),
                            (0x31A3DC, In::F),
                        ],
                        writes: &[(0x21423C, Out::F), (0x214248, Out::F), (0x21424C, Out::F)],
                    },
                    Operand::Value(0x14),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_6(a[0], a[1]) },
                plain: |a| {
                    drive_forces_from_grips(if a[1] == 0 {
                        &crate::race::owner::race().shared.physics_tables.wheel_grips
                    } else {
                        &crate::race::owner::race()
                            .shared
                            .physics_tables
                            .wheel_grips_b
                    });
                },
            },
            Block {
                name: "drive_wheels_step line 180",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x1C5E2C, In::F),
                        (0x213EC4, In::F),
                        (0x214170, In::F),
                        (0x21423C, In::F),
                        (0x214268, In::F),
                        (0x22400C, In::I),
                        (0x31A17C, In::F),
                    ],
                    writes: &[(0x214240, Out::F), (0x22400C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_7(a[0]) },
                plain: |_| throttle_drive_limit(&crate::race::owner::race().course),
            },
            Block {
                name: "drive_wheels_step line 203",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x1C5E40, In::F),
                        (0x21388C, In::F),
                        (0x214750, In::F),
                        (0x214BA8, In::F),
                        (0x31A190, In::F),
                        (0x31A194, In::F),
                    ],
                    writes: &[(0x31A190, Out::F), (0x31A194, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_8(a[0]) },
                plain: |_| push_both_drive_speeds(&crate::race::owner::race().course),
            },
            Block {
                name: "drive_wheels_step line 220",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x21388C, In::F),
                        (0x214750, In::F),
                        (0x214BA8, In::F),
                        (0x31A190, In::F),
                    ],
                    writes: &[(0x31A190, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_9(a[0]) },
                plain: |_| push_drive_speed_a(),
            },
            Block {
                name: "drive_wheels_step line 231",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x1C5E2C, In::F),
                        (0x21388C, In::F),
                        (0x214BA8, In::F),
                        (0x214BB0, In::F),
                        (0x214BF0, In::F),
                        (0x31A3D0, In::F),
                        (0x31A614, In::I),
                    ],
                    writes: &[(0x21422C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_10(a[0]) },
                plain: |_| brake_step_for_a(&crate::race::owner::race().course),
            },
            Block {
                name: "drive_wheels_step line 245",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x1C5E2C, In::F),
                        (0x21388C, In::F),
                        (0x214BA8, In::F),
                        (0x214BB0, In::F),
                        (0x214BF0, In::F),
                        (0x31A3D4, In::F),
                        (0x31A614, In::I),
                    ],
                    writes: &[(0x21422C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_11(a[0]) },
                plain: |_| brake_step_for_b(&crate::race::owner::race().course),
            },
            Block {
                name: "drive_wheels_step line 258",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x1C5E2C, In::F),
                        (0x21388C, In::F),
                        (0x214170, In::F),
                        (0x214BA8, In::F),
                        (0x214BAC, In::F),
                        (0x31A618, In::I),
                    ],
                    writes: &[(0x21422C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_12(a[0]) },
                plain: |_| handbrake_step_for_b(&crate::race::owner::race().course),
            },
            Block {
                name: "drive_wheels_step line 273",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F), (4, In::F), (8, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::H)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_13(a[0], a[1]) },
                plain: |a| a[1].set_word(0, handbrake_stop_status(*view(a[0], 0))),
            },
            Block {
                name: "drive_wheels_step line 292",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x31A184, In::F), (0x31A190, In::F), (0x31A194, In::F)],
                    writes: &[(0x214234, Out::F), (0x214238, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_14(a[0]) },
                plain: |_| slip_gaps(),
            },
            Block {
                name: "drive_wheels_step line 308",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x2140B0, In::F),
                        (0x2140B4, In::F),
                        (0x214244, In::F),
                        (0x214248, In::F),
                        (0x21424C, In::F),
                    ],
                    writes: &[(0x214244, Out::F), (0x214248, Out::F), (0x21424C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_15(a[0]) },
                plain: |_| scale_axle_forces(),
            },
            Block {
                name: "drive_wheels_step line 331",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x1C5DA4, In::F),
                        (0x21388C, In::F),
                        (0x22400C, In::F),
                        (0x31A17C, In::F),
                    ],
                    writes: &[
                        (0x214250, Out::F),
                        (0x2148D0, Out::F),
                        (0x2148F0, Out::F),
                        (0x214910, Out::F),
                        (0x214930, Out::F),
                    ],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_16(a[0]) },
                plain: |_| side_slip_forces(&crate::race::owner::race().course),
            },
            Block {
                name: "drive_wheels_step line 347",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x213ED0, In::F),
                        (0x214250, In::F),
                        (0x214254, In::F),
                        (0x214264, In::F),
                        (0x214BFC, In::F),
                    ],
                    writes: &[(0x2141D4, Out::F), (0x214254, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_17(a[0]) },
                plain: |_| slip_side_share(),
            },
            Block {
                name: "drive_wheels_step line 362",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x213ED4, In::F), (0x214BFC, In::F)],
                    writes: &[(0x2141D8, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_18(a[0]) },
                plain: |_| spin_push_b(),
            },
            Block {
                name: "drive_wheels_step line 372",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[(0x214764, In::F), (0x31A17C, In::F), (0x31A184, In::F)],
                        writes: &[(0x22400C, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::H)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_19(a[0], a[1]) },
                plain: |a| a[1].set_word(0, speed_over_ground()),
            },
            Block {
                name: "drive_wheels_step line 385",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x214760, In::F),
                        (0x22400C, In::F),
                        (0x31A17C, In::F),
                        (0x31A184, In::F),
                    ],
                    writes: &[(0x2148B0, Out::F), (0x2148B8, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_20(a[0]) },
                plain: |_| rolling_drag_forces(),
            },
            Block {
                name: "drive_wheels_step line 403",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x1C5E40, In::F),
                        (0x2148D0, In::F),
                        (0x2148F0, In::F),
                        (0x214910, In::F),
                        (0x214930, In::F),
                        (0x2149D0, In::F),
                        (0x2149F0, In::F),
                    ],
                    writes: &[
                        (0x2148D0, Out::F),
                        (0x2148F0, Out::F),
                        (0x214910, Out::F),
                        (0x214930, Out::F),
                    ],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_21(a[0]) },
                plain: |_| share_wheel_forces(&crate::race::owner::race().course),
            },
            Block {
                name: "drive_wheels_step line 426",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x213EDC, In::F), (0x31A184, In::F), (0x31A190, In::F)],
                    writes: &[(0x22401C, Out::F), (0x31A158, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_22(a[0]) },
                plain: |_| slip_figure_a(),
            },
            Block {
                name: "drive_wheels_step line 436",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x213ED8, In::F), (0x31A17C, In::F)],
                    writes: &[(0x224020, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_23(a[0]) },
                plain: |_| slip_figure_b(),
            },
            Block {
                name: "drive_wheels_step line 444",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x22401C, In::F),
                        (0x224020, In::F),
                        (0x31A17C, In::F),
                        (0x31A184, In::F),
                        (0x31A190, In::F),
                        (0x31A194, In::F),
                    ],
                    writes: &[
                        (0x22401C, Out::F),
                        (0x224020, Out::F),
                        (0x31A260, Out::F),
                        (0x31AF18, Out::F),
                        (0x31AF1C, Out::F),
                    ],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_24(a[0]) },
                plain: |_| slip_lengths(),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
