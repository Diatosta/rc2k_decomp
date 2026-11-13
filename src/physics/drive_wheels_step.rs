//! `physics::drive_wheels_step` (0x43DFB1): the driven wheels' step: the slip shares from
//! the car-frame wheel speeds (held 0 ..= 1 and turned into a ±1 share), the clutch and
//! the gear deciding the drive force (`drive_torque` 0 in neutral), `physics::cap_drive_speed`;
//! the drive speeds stepped by the drive force less the wheels' drag and held toward the
//! car's speed, `physics::revs_from_wheels` (twice, on the two paths); the slips
//! `wheel_slips` / `614238`; `physics::cap_reverse_inputs`, `physics::add_drive_forces`,
//! `physics::ease_low_speed_forces`; and the rest of the wheels' forces into the force
//! records.
//!
//! The flow, the compares, the integer steps and the calls are Rust; each straight x87
//! run is plain Rust over `X` (`math/x87_float.rs`) in the named helpers below, the
//! original's instructions being recorded in the tests' `reference_vectors`.
//!
//! Caller: `physics::car_physics_step`, every physics step. Refuses (before anything) a gear
//! outside -1 ..= 7 (`revs_from_wheels` would; the gear is not written here).

use crate::effects::cell_x87::CellX87;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87::{abs_compare, abs_of, neg_rounded};
use crate::math::x87_float::X;
use crate::math::x87_float::compare;
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::physics::car_store::Car;
use crate::physics::flow::FloatView;
use crate::physics::flow::{ZERO_BITS as ZERO, c0, ease_zero, hold_high, hold_low, hold_low_bits};
use crate::physics::gearing::{GearPlaces, revs_from_wheels_at};
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use crate::race::owner_course::Course;
use crate::race::vars_store::RaceVars;
use crate::scene::matrix3::extended;
use crate::session::Cell32;
use crate::session::FloatLike;
use crate::session::PlainFloat;

/// |a| below |b|, or one of them not a number (`fld b; fabs; fld a; fabs; fcompp`, C0).
fn abs_below(x: impl x87::Flt, y: impl x87::Flt) -> bool {
    status_below(abs_status([x.bits(), y.bits()]))
}

/// `fld from; fstp to`.
fn store_loaded(to: &PlainFloat, from: &PlainFloat) {
    to.set(x87::loaded(from));
}

/// The cells one wheel's slip step works on (`settle_slip`).
#[derive(Clone, Copy)]
struct SlipCells<'a> {
    drive: &'a PlainFloat,
    slip: &'a PlainFloat,
    ratio: &'a PlainFloat,
    f1: &'a PlainFloat,
    f2: &'a PlainFloat,
}

/// 0x43E5CF .. 0x43E643 (and the second wheel's at 0x43E68D): the slip `s` (at `slip`)
/// signed by the slip ratio's sign, the drive speed (`drive`) less it x (`spec_wheel`
/// - 1), and the two force records' parts at `f1`, `f2`.
fn settle_slip(car: &Car, tables: &PhysicsTables, numbers: &Numbers, cells: &SlipCells) {
    let SlipCells {
        drive,
        slip,
        ratio,
        f1,
        f2,
    } = *cells;
    slip_ratio(car, tables, drive, ratio);
    let s = abs_of(slip);
    let signed = if c0(ratio, ZERO) {
        neg_rounded(extended(s))
    } else {
        s
    };
    slip.set(signed);
    apply_slip_to_drive(tables, numbers, drive, slip, f1, f2);
}

/// The slip's first limit (0x43E58B / 0x43E649): |drive - speed| / `spec_wheel` into
/// `ratio`, and `slip` the lesser of it and `force` (by `fcompp`).
fn slip_limit(
    car: &Car,
    tables: &PhysicsTables,
    drive: &PlainFloat,
    ratio: &PlainFloat,
    force: &PlainFloat,
    slip: &PlainFloat,
) {
    slip_ratio_size(car, tables, drive, ratio);
    let pick = if c0(ratio, force) { ratio } else { force };
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
fn steps(
    course: &Course,
    vars: &RaceVars,
    physics_tables: &PhysicsTables,
    car: &Car,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let drawing = &scene.drawing;
    let gear = car.gear.get();
    one_less_abs(
        course,
        &car.crash_matrix[1],
        &physics_tables.drive_shares[0],
    );
    hold_low(&physics_tables.drive_shares[0], &course.numbers.get().tenth);
    hold_high(&physics_tables.drive_shares[0], &course.numbers.get().one);
    one_less_abs(
        course,
        &car.crash_matrix[9],
        &physics_tables.drive_shares[1],
    );
    hold_low(&physics_tables.drive_shares[1], &course.numbers.get().tenth);
    hold_high(&physics_tables.drive_shares[1], &course.numbers.get().one);
    signed_share(physics_tables, course.numbers.get());
    hold_low_bits(&physics_tables.drive_shares[2], ZERO);
    hold_high(&physics_tables.drive_shares[2], &course.numbers.get().one);
    if car.driven_wheels_off.get() == 0 {
        if gear == 0 {
            physics_tables.drive_torque.set(0.0);
        }
        crate::physics::cap_drive_speed::cap_drive_speed(car, physics_tables, scene);
        physics_tables.wheels_driven.set(0);
        // the grips at +0 or, braking, +0x14: `wheel_grips_b`
        let mut grips = &physics_tables.wheel_grips;
        if car.brake.get() != 0 {
            grips = &physics_tables.wheel_grips_b;
            physics_tables.wheels_driven.set(1);
        }
        drive_forces_from_grips(car, physics_tables, grips);
        drawing
            .scene
            .get()
            .scratch
            .set(car.throttle.get().wrapping_add(0x100));
        throttle_drive_limit(car, physics_tables, course.numbers.get(), scene);
        hold_high(
            &physics_tables.drive_limits[1],
            &drawing.scene.get().scratch,
        );
        let pick = if abs_below(
            &physics_tables.drive_limits[1],
            &physics_tables.drive_limits[0],
        ) {
            &physics_tables.drive_limits[0]
        } else {
            &physics_tables.drive_limits[1]
        };
        store_loaded(&physics_tables.drive_limits[2], pick);
        let kind = car.drive_kind.get();
        if gear != 0 && kind == 1 {
            push_both_drive_speeds(car, physics_tables, course.numbers.get());
        }
        if kind == 0 {
            push_drive_speed_a(car, physics_tables);
        }
        revs(car, physics_tables, course.numbers.get())?;
        // 0x43E2C2: the wheels' drag on each drive speed.
        brake_step_for_a(car, physics_tables, course.numbers.get());
        ease_zero(&car.drive_speed_a, &physics_tables.wheel_speed_step);
        brake_step_for_b(car, physics_tables, course.numbers.get());
        ease_zero(&car.drive_speed_b, &physics_tables.wheel_speed_step);
        handbrake_step_for_b(car, physics_tables, course.numbers.get());
        ease_zero(&car.drive_speed_b, &physics_tables.wheel_speed_step);
        if car.handbrake.get() != 0 {
            let status = handbrake_stop_status(
                &car.drive_speed_a,
                &car.drive_speed_b,
                &course.numbers.get().one,
            );
            if status_below(status) {
                car.drive_speed_a.set(0.0);
                car.drive_speed_b.set(0.0);
            }
        }
        revs(car, physics_tables, course.numbers.get())?;
        slip_gaps(car, physics_tables);
        crate::physics::cap_reverse_inputs::cap_reverse_inputs(
            car,
            physics_tables,
            course.numbers.get(),
        );
        crate::physics::add_drive_forces::add_drive_forces(car, physics_tables, scene);
        crate::physics::ease_low_speed_forces::ease_low_speed_forces(car, physics_tables);

        scale_axle_forces(physics_tables);
        if car.parts_state.get() != 0 {
            physics_tables.drive_forces[0].set(x87::mul(
                &physics_tables.drive_forces[0],
                &course.numbers.get().half,
            ));
            physics_tables.drive_forces[1].set(x87::mul(
                &physics_tables.drive_forces[1],
                &course.numbers.get().half,
            ));
        }
        slip_limit(
            car,
            physics_tables,
            &car.drive_speed_a,
            &physics_tables.axle_slip_limits[0],
            &physics_tables.drive_forces[0],
            &physics_tables.axle_slips[0],
        );
        settle_slip(
            car,
            physics_tables,
            course.numbers.get(),
            &SlipCells {
                drive: &car.drive_speed_a,
                slip: &physics_tables.axle_slips[0],
                ratio: &physics_tables.axle_slip_limits[0],
                f1: &physics_tables.forces[15].force[2],
                f2: &physics_tables.forces[16].force[2],
            },
        );
        slip_limit(
            car,
            physics_tables,
            &car.drive_speed_b,
            &physics_tables.axle_slip_limits[1],
            &physics_tables.drive_forces[1],
            &physics_tables.axle_slips[1],
        );
        settle_slip(
            car,
            physics_tables,
            course.numbers.get(),
            &SlipCells {
                drive: &car.drive_speed_b,
                slip: &physics_tables.axle_slips[1],
                ratio: &physics_tables.axle_slip_limits[1],
                f1: &physics_tables.forces[17].force[2],
                f2: &physics_tables.forces[18].force[2],
            },
        );
        // 0x43E707: the car-frame side speed eased by the side grip.
        drawing
            .scene
            .get()
            .scratch
            .set_float(car.local_velocity[0].get());
        ease_zero(
            &drawing.scene.get().scratch,
            &physics_tables.drive_limits[2],
        );
        side_slip_forces(car, physics_tables, course.numbers.get(), scene);
        store_loaded(&physics_tables.slip_limit, &physics_tables.drive_limits[0]);
        store_loaded(
            &physics_tables.wheel_spin_push[0],
            &physics_tables.handling[4],
        );
        let status = abs_compare(&physics_tables.drive_slip[0], &physics_tables.slip_limit);
        if status_above(status) {
            slip_side_share(physics_tables);
        }
        spin_push_b(physics_tables);
        physics_tables.forces[10].force[0].set(0.0);
        physics_tables.forces[10].force[2].set(0.0);
        // 0x43E855: the rolling drag against the motion.
        let status = speed_over_ground(car, physics_tables, scene);
        if status_above(status) {
            rolling_drag_forces(car, physics_tables, scene);
        }
        if vars.run_kind.get() != 1 && (car.run_ticks.get().wrapping_sub(0x320) as i32) < 0 {
            car.drive_speed_a.set(0.0);
            car.drive_speed_b.set(0.0);
        }
        share_wheel_forces(physics_tables, course.numbers.get());
        physics_tables.forces[19].force[0].set(0.0);
        physics_tables.forces[20].force[0].set(0.0);
    }
    // 0x43E957: the wheels' slip figures.
    slip_figure_a(car, physics_tables, scene);
    hold_low_bits(&drawing.scene.get().work_float_a, ZERO);
    slip_figure_b(car, physics_tables, scene);
    hold_low_bits(&drawing.scene.get().work_float_b, ZERO);
    slip_lengths(car, scene);
    Ok(())
}

/// `revs_from_wheels` (the gear checked before).
fn revs(car: &Car, tables: &PhysicsTables, numbers: &Numbers) -> Result<(), Refused> {
    { revs_from_wheels_at(numbers, &GearPlaces::game(car, tables)) }?;
    Ok(())
}

pub fn drive_wheels_step(
    course: &Course,
    vars: &RaceVars,
    physics_tables: &PhysicsTables,
    car: &Car,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let gear = car.gear.get();
    if gear != 0 && !(-1..=7).contains(&gear) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    steps(course, vars, physics_tables, car, scene)
}

// ---- the arithmetic that was x87 assembly ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[t + 0] = -(|[v + 0]| - [one])`
fn one_less_abs(course: &Course, v: &impl FloatLike, t: &impl FloatLike) {
    let t0 = v.ld();
    let t1 = t0.abs();
    let t2 = course.numbers.get().one.ld();
    let t3 = t1 - t2;
    let t4 = -t3;
    t.st(t4);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `status [s + 0] = status`
fn abs_status(v: [u32; 2]) -> u16 {
    let t0 = ld(v[1]);
    let t1 = t0.abs();
    let t2 = ld(v[0]);
    let t3 = t2.abs();
    compare(t3, t1)
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[r + 0] = ([d + 0] - [rc + 0x31A184]) / [spec_wheel]`
fn slip_ratio(car: &Car, tables: &PhysicsTables, d: &impl Cell32, r: &PlainFloat) {
    let t0 = d.ld();
    let t1 = car.local_velocity[2].ld();
    let t2 = t0 - t1;
    let t3 = tables.spec_wheel.ld();
    let t4 = t2 / t3;
    r.st(t4);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[d + 0] = [d + 0] - (([spec_wheel] - [one]) * [s + 0])`
/// `[f1 + 0] = ([s + 0] / [step_time]) * [half]`
/// `[f2 + 0] = ([s + 0] / [step_time]) * [half]`
fn apply_slip_to_drive(
    tables: &PhysicsTables,
    numbers: &Numbers,
    d: &impl Cell32,
    s: &PlainFloat,
    f1: &PlainFloat,
    f2: &PlainFloat,
) {
    let t0 = d.ld();
    let t1 = tables.spec_wheel.ld();
    let t2 = numbers.one.ld();
    let t3 = t1 - t2;
    let t4 = s.ld();
    let t5 = t3 * t4;
    let t6 = t0 - t5;
    d.st(t6);
    let t7 = s.ld();
    let t8 = tables.step_time.ld();
    let t9 = t7 / t8;
    let t10 = numbers.half.ld();
    let t11 = t9 * t10;
    f1.st(t11);
    f2.st(t11);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[r + 0] = |(([d + 0] - [rc + 0x31A184]) / [spec_wheel])|`
fn slip_ratio_size(car: &Car, tables: &PhysicsTables, d: &impl Cell32, r: &PlainFloat) {
    let t0 = d.ld();
    let t1 = car.local_velocity[2].ld();
    let t2 = t0 - t1;
    let t3 = tables.spec_wheel.ld();
    let t4 = t2 / t3;
    let t5 = t4.abs();
    r.st(t5);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2140B8] = ([rc + 0x2140B4] + [rc + 0x2140B4]) - [one]`
fn signed_share(tables: &PhysicsTables, numbers: &Numbers) {
    let t0 = tables.drive_shares[1].ld();
    let t1 = t0 + t0;
    let t2 = numbers.one.ld();
    let t3 = t1 - t2;
    tables.drive_shares[2].st(t3);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (a long formula: checked by the tests' `reference_vectors`)
/// (a long formula: checked by the tests' `reference_vectors`)
/// (a long formula: checked by the tests' `reference_vectors`)
fn drive_forces_from_grips(car: &Car, tables: &PhysicsTables, grips: &[PlainFloat; 5]) {
    let t0 = grips[0].ld();
    let t1 = tables.forces[0].force[1].ld();
    let t2 = t0 * t1;
    let t3 = grips[1].ld();
    let t4 = tables.forces[1].force[1].ld();
    let t5 = t3 * t4;
    let t6 = t2 + t5;
    let t7 = -t6;
    let t8 = tables.step_time.ld();
    let t9 = t7 * t8;
    let t10 = tables.weight.ld();
    let t11 = t9 * t10;
    let t12 = car.drive_efficiency[0].ld();
    let t13 = t11 * t12;
    tables.drive_forces[0].st(t13);
    let t14 = grips[2].ld();
    let t15 = tables.forces[2].force[1].ld();
    let t16 = t14 * t15;
    let t17 = grips[3].ld();
    let t18 = tables.forces[3].force[1].ld();
    let t19 = t17 * t18;
    let t20 = t16 + t19;
    let t21 = -t20;
    let t22 = tables.step_time.ld();
    let t23 = t21 * t22;
    let t24 = tables.weight.ld();
    let t25 = t23 * t24;
    let t26 = car.drive_efficiency[1].ld();
    let t27 = t25 * t26;
    tables.drive_forces[1].st(t27);
    let t28 = tables.wheel_loads[0].ld();
    let t29 = tables.wheel_loads[1].ld();
    let t30 = t28 + t29;
    let t31 = tables.wheel_loads[2].ld();
    let t32 = t30 + t31;
    let t33 = tables.wheel_loads[3].ld();
    let t34 = t32 + t33;
    let t35 = tables.forces[0].force[1].ld();
    let t36 = t34 * t35;
    let t37 = -t36;
    let t38 = tables.step_time.ld();
    let t39 = t37 * t38;
    let t40 = tables.weight.ld();
    let t41 = t39 * t40;
    tables.drive_limits[0].st(t41);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214240] = ((int [scratch] / [k256]) * |([local_velocity] * [wheel_load])|) * [weight]`
/// `[scratch] = [drive_limits] * [drive_speed_scale]`
fn throttle_drive_limit(
    car: &Car,
    tables: &PhysicsTables,
    numbers: &Numbers,
    scene: &crate::scene::owner::Scene,
) {
    let drawing = &scene.drawing;
    let t0 = int(drawing.scene.get().scratch.get() as i32);
    let t1 = numbers.k256.ld();
    let t2 = t0 / t1;
    let t3 = car.local_velocity[0].ld();
    let t4 = tables.wheel_load.ld();
    let t5 = t3 * t4;
    let t6 = t5.abs();
    let t7 = t2 * t6;
    let t8 = tables.weight.ld();
    let t9 = t7 * t8;
    tables.drive_limits[1].st(t9);
    let t10 = tables.drive_limits[0].ld();
    let t11 = tables.drive_speed_scale.ld();
    let t12 = t10 * t11;
    drawing.scene.get().scratch.st(t12);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[drive_speed_a] = ((([drive_torque] * [half]) * [step_time]) * [spec_wheel]) + [drive_speed_a]`
/// `[drive_speed_b] = ((([drive_torque] * [half]) * [step_time]) * [spec_wheel]) + [drive_speed_b]`
fn push_both_drive_speeds(car: &Car, tables: &PhysicsTables, numbers: &Numbers) {
    let t0 = tables.drive_torque.ld();
    let t1 = numbers.half.ld();
    let t2 = t0 * t1;
    let t3 = tables.step_time.ld();
    let t4 = t2 * t3;
    let t5 = tables.spec_wheel.ld();
    let t6 = t4 * t5;
    let t7 = car.drive_speed_a.ld();
    let t8 = t6 + t7;
    car.drive_speed_a.st(t8);
    let t9 = tables.drive_torque.ld();
    let t10 = numbers.half.ld();
    let t11 = t9 * t10;
    let t12 = tables.step_time.ld();
    let t13 = t11 * t12;
    let t14 = tables.spec_wheel.ld();
    let t15 = t13 * t14;
    let t16 = car.drive_speed_b.ld();
    let t17 = t15 + t16;
    car.drive_speed_b.st(t17);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[drive_speed_a] = (([drive_torque] * [step_time]) * [spec_wheel]) + [drive_speed_a]`
fn push_drive_speed_a(car: &Car, tables: &PhysicsTables) {
    let t0 = tables.drive_torque.ld();
    let t1 = tables.step_time.ld();
    let t2 = t0 * t1;
    let t3 = tables.spec_wheel.ld();
    let t4 = t2 * t3;
    let t5 = car.drive_speed_a.ld();
    let t6 = t4 + t5;
    car.drive_speed_a.st(t6);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (a long formula: checked by the tests' `reference_vectors`)
fn brake_step_for_a(car: &Car, tables: &PhysicsTables, numbers: &Numbers) {
    let t0 = X(1.0);
    let t1 = tables.handling[1].ld();
    let t2 = t0 - t1;
    let t3 = tables.spec_drive[1].ld();
    let t4 = t2 * t3;
    let t5 = int(car.brake.get() as i32);
    let t6 = t4 * t5;
    let t7 = numbers.k256.ld();
    let t8 = t6 / t7;
    let t9 = tables.step_time.ld();
    let t10 = t8 * t9;
    let t11 = car.brake_efficiency[0].ld();
    let t12 = t10 * t11;
    let t13 = tables.spec_wheel.ld();
    let t14 = t12 * t13;
    tables.wheel_speed_step.st(t14);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (a long formula: checked by the tests' `reference_vectors`)
fn brake_step_for_b(car: &Car, tables: &PhysicsTables, numbers: &Numbers) {
    let t0 = tables.handling[1].ld();
    let t1 = tables.spec_drive[1].ld();
    let t2 = t0 * t1;
    let t3 = int(car.brake.get() as i32);
    let t4 = t2 * t3;
    let t5 = numbers.k256.ld();
    let t6 = t4 / t5;
    let t7 = tables.step_time.ld();
    let t8 = t6 * t7;
    let t9 = car.brake_efficiency[1].ld();
    let t10 = t8 * t9;
    let t11 = tables.spec_wheel.ld();
    let t12 = t10 * t11;
    tables.wheel_speed_step.st(t12);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[wheel_speed_step] = (((([spec_drive] * int [handbrake]) / [k256]) * [step_time]) * [spec_wheel]) * [weight]`
fn handbrake_step_for_b(car: &Car, tables: &PhysicsTables, numbers: &Numbers) {
    let t0 = tables.spec_drive[0].ld();
    let t1 = int(car.handbrake.get() as i32);
    let t2 = t0 * t1;
    let t3 = numbers.k256.ld();
    let t4 = t2 / t3;
    let t5 = tables.step_time.ld();
    let t6 = t4 * t5;
    let t7 = tables.spec_wheel.ld();
    let t8 = t6 * t7;
    let t9 = tables.weight.ld();
    let t10 = t8 * t9;
    tables.wheel_speed_step.st(t10);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `status [s + 0] = status`
fn handbrake_stop_status(a: impl x87::Flt, b: impl x87::Flt, limit: impl x87::Flt) -> u16 {
    let t0 = ld(a.bits());
    let t1 = ld(b.bits());
    let t2 = t0 + t1;
    let t3 = t2.abs();
    let t4 = ld(limit.bits());
    compare(t3, t4)
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214238] = |([drive_speed_b] - [rc + 0x31A184])|`
/// `[wheel_slips] = |([drive_speed_a] - [rc + 0x31A184])|`
fn slip_gaps(car: &Car, tables: &PhysicsTables) {
    let t0 = car.drive_speed_b.ld();
    let t1 = car.local_velocity[2].ld();
    let t2 = t0 - t1;
    let t3 = t2.abs();
    tables.wheel_slips[1].st(t3);
    let t4 = car.drive_speed_a.ld();
    let t5 = car.local_velocity[2].ld();
    let t6 = t4 - t5;
    let t7 = t6.abs();
    tables.wheel_slips[0].st(t7);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214244] = [rc + 0x214244] * [drive_shares]`
/// `[drive_forces] = [drive_forces] * [rc + 0x2140B4]`
/// `[rc + 0x21424C] = [rc + 0x21424C] * [rc + 0x2140B4]`
fn scale_axle_forces(tables: &PhysicsTables) {
    let t0 = tables.drive_limits[2].ld();
    let t1 = tables.drive_shares[0].ld();
    let t2 = t0 * t1;
    tables.drive_limits[2].st(t2);
    let t3 = tables.drive_forces[0].ld();
    let t4 = tables.drive_shares[1].ld();
    let t5 = t3 * t4;
    tables.drive_forces[0].st(t5);
    let t6 = tables.drive_forces[1].ld();
    let t7 = tables.drive_shares[1].ld();
    let t8 = t6 * t7;
    tables.drive_forces[1].st(t8);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[drive_slip] = [scratch] - [local_velocity]`
/// `[rc + 0x2148D0] = (([scratch] - [local_velocity]) / [step_time]) * [quarter]`
/// `[rc + 0x2148F0] = (([scratch] - [local_velocity]) / [step_time]) * [quarter]`
/// `[rc + 0x214910] = (([scratch] - [local_velocity]) / [step_time]) * [quarter]`
/// `[rc + 0x214930] = (([scratch] - [local_velocity]) / [step_time]) * [quarter]`
fn side_slip_forces(
    car: &Car,
    tables: &PhysicsTables,
    numbers: &Numbers,
    scene: &crate::scene::owner::Scene,
) {
    let t0 = scene.drawing.scene.get().scratch.ld();
    let t1 = car.local_velocity[0].ld();
    let t2 = t0 - t1;
    tables.drive_slip[0].st(t2);
    let t3 = tables.step_time.ld();
    let t4 = t2 / t3;
    let t5 = numbers.quarter.ld();
    let t6 = t4 * t5;
    tables.forces[11].force[0].st(t6);
    tables.forces[12].force[0].st(t6);
    tables.forces[13].force[0].st(t6);
    tables.forces[14].force[0].st(t6);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214254] = ((|[drive_slip]| - [slip_limit]) * [slip_blend]) + [slip_limit]`
/// `[wheel_spin_push] = |([rc + 0x214254] / [drive_slip])| * [rc + 0x214BFC]`
fn slip_side_share(tables: &PhysicsTables) {
    let t0 = tables.drive_slip[0].ld();
    let t1 = t0.abs();
    let t2 = tables.slip_limit.ld();
    let t3 = t1 - t2;
    let t4 = tables.slip_blend.ld();
    let t5 = t3 * t4;
    let t6 = tables.slip_limit.ld();
    let t7 = t5 + t6;
    tables.drive_slip[1].st(t7);
    let t8 = tables.drive_slip[1].ld();
    let t9 = tables.drive_slip[0].ld();
    let t10 = t8 / t9;
    let t11 = t10.abs();
    let t12 = tables.handling[4].ld();
    let t13 = t11 * t12;
    tables.wheel_spin_push[0].st(t13);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2141D8] = [rc + 0x214BFC] * [slip_share]`
fn spin_push_b(tables: &PhysicsTables) {
    let t0 = tables.handling[4].ld();
    let t1 = tables.slip_share.ld();
    let t2 = t0 * t1;
    tables.wheel_spin_push[1].st(t2);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[scratch] = sqrt((([local_velocity] * [local_velocity]) + ([rc + 0x31A184] * [rc + 0x31A184])))`
/// `status [s + 0] = status`
fn speed_over_ground(car: &Car, tables: &PhysicsTables, scene: &crate::scene::owner::Scene) -> u16 {
    let t0 = car.local_velocity[0].ld();
    let t1 = t0 * t0;
    let t2 = car.local_velocity[2].ld();
    let t3 = t2 * t2;
    let t4 = t1 + t3;
    let t5 = t4.sqrt();
    scene.drawing.scene.get().scratch.st(t5);
    let t6 = tables.wheel_drag[1].ld();
    compare(t5, t6)
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2148B0] = -(([wheel_drag] / [scratch]) * [local_velocity])`
/// `[rc + 0x2148B8] = -(([wheel_drag] / [scratch]) * [rc + 0x31A184])`
fn rolling_drag_forces(car: &Car, tables: &PhysicsTables, scene: &crate::scene::owner::Scene) {
    let drawing = &scene.drawing;
    let t0 = tables.wheel_drag[0].ld();
    let t1 = drawing.scene.get().scratch.ld();
    let t2 = t0 / t1;
    let t3 = car.local_velocity[0].ld();
    let t4 = t2 * t3;
    let t5 = -t4;
    tables.forces[10].force[0].st(t5);
    let t6 = tables.wheel_drag[0].ld();
    let t7 = drawing.scene.get().scratch.ld();
    let t8 = t6 / t7;
    let t9 = car.local_velocity[2].ld();
    let t10 = t8 * t9;
    let t11 = -t10;
    tables.forces[10].force[2].st(t11);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2148D0] = ([rc + 0x2149D0] * [half]) + [rc + 0x2148D0]`
/// `[rc + 0x2148F0] = ([rc + 0x2149D0] * [half]) + [rc + 0x2148F0]`
/// `[rc + 0x214910] = ([rc + 0x2149F0] * [half]) + [rc + 0x214910]`
/// `[rc + 0x214930] = ([rc + 0x2149F0] * [half]) + [rc + 0x214930]`
fn share_wheel_forces(tables: &PhysicsTables, numbers: &Numbers) {
    let t0 = tables.forces[19].force[0].ld();
    let t1 = numbers.half.ld();
    let t2 = t0 * t1;
    let t3 = tables.forces[11].force[0].ld();
    let t4 = t2 + t3;
    tables.forces[11].force[0].st(t4);
    let t5 = tables.forces[19].force[0].ld();
    let t6 = numbers.half.ld();
    let t7 = t5 * t6;
    let t8 = tables.forces[12].force[0].ld();
    let t9 = t7 + t8;
    tables.forces[12].force[0].st(t9);
    let t10 = tables.forces[20].force[0].ld();
    let t11 = numbers.half.ld();
    let t12 = t10 * t11;
    let t13 = tables.forces[13].force[0].ld();
    let t14 = t12 + t13;
    tables.forces[13].force[0].st(t14);
    let t15 = tables.forces[20].force[0].ld();
    let t16 = numbers.half.ld();
    let t17 = t15 * t16;
    let t18 = tables.forces[14].force[0].ld();
    let t19 = t17 + t18;
    tables.forces[14].force[0].st(t19);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[drive_slip_speed] = [drive_speed_a] - [rc + 0x31A184]`
/// `[work_float_a] = |([drive_speed_a] - [rc + 0x31A184])| - [wheel_speed_gap]`
fn slip_figure_a(car: &Car, tables: &PhysicsTables, scene: &crate::scene::owner::Scene) {
    let t0 = car.drive_speed_a.ld();
    let t1 = car.local_velocity[2].ld();
    let t2 = t0 - t1;
    car.drive_slip_speed.st(t2);
    let t3 = t2.abs();
    let t4 = tables.wheel_speed_gap.ld();
    let t5 = t3 - t4;
    scene.drawing.scene.get().work_float_a.st(t5);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[work_float_b] = |[local_velocity]| - [wheel_speed_margin]`
fn slip_figure_b(car: &Car, tables: &PhysicsTables, scene: &crate::scene::owner::Scene) {
    let t0 = car.local_velocity[0].ld();
    let t1 = t0.abs();
    let t2 = tables.wheel_speed_margin.ld();
    let t3 = t1 - t2;
    scene.drawing.scene.get().work_float_b.st(t3);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[slip_speed] = sqrt((([work_float_a] * [work_float_a]) + ([work_float_b] * [work_float_b])))`
/// `[work_float_b] = |[local_velocity]|`
/// `[work_float_a] = |([drive_speed_a] - [rc + 0x31A184])|`
/// `[skid_grip] = sqrt((([work_float_a] * [work_float_a]) + ([work_float_b] * [work_float_b])))`
/// `[work_float_a] = |([drive_speed_b] - [rc + 0x31A184])|`
/// `[rc + 0x31AF1C] = sqrt((([work_float_a] * [work_float_a]) + ([work_float_b] * [work_float_b])))`
fn slip_lengths(car: &Car, scene: &crate::scene::owner::Scene) {
    let drawing = &scene.drawing;
    let t0 = drawing.scene.get().work_float_a.ld();
    let t1 = t0 * t0;
    let t2 = drawing.scene.get().work_float_b.ld();
    let t3 = t2 * t2;
    let t4 = t1 + t3;
    let t5 = t4.sqrt();
    car.slip_speed.st(t5);
    let t6 = car.local_velocity[0].ld();
    let t7 = t6.abs();
    drawing.scene.get().work_float_b.st(t7);
    let t8 = car.drive_speed_a.ld();
    let t9 = car.local_velocity[2].ld();
    let t10 = t8 - t9;
    let t11 = t10.abs();
    drawing.scene.get().work_float_a.st(t11);
    let t12 = drawing.scene.get().work_float_a.ld();
    let t13 = t12 * t12;
    let t14 = drawing.scene.get().work_float_b.ld();
    let t15 = t14 * t14;
    let t16 = t13 + t15;
    let t17 = t16.sqrt();
    car.skid_grip[0].st(t17);
    let t18 = car.drive_speed_b.ld();
    let t19 = car.local_velocity[2].ld();
    let t20 = t18 - t19;
    let t21 = t20.abs();
    drawing.scene.get().work_float_a.st(t21);
    let t22 = drawing.scene.get().work_float_a.ld();
    let t23 = t22 * t22;
    let t24 = drawing.scene.get().work_float_b.ld();
    let t25 = t24 * t24;
    let t26 = t23 + t25;
    let t27 = t26.sqrt();
    car.skid_grip[1].st(t27);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::race::owner::race;
    use crate::scene::x87_mem::{Base, view};
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

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
                plain: |a| {
                    one_less_abs(
                        &crate::race::owner::race().course,
                        view::<PlainFloat>(a[0], 0),
                        view::<PlainFloat>(a[2], 0),
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
                plain: |a| {
                    slip_ratio(
                        race().shared.car(),
                        race().shared.physics_tables.get(),
                        view::<PlainFloat>(a[0], 0),
                        view::<PlainFloat>(a[2], 0),
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
                plain: |a| {
                    apply_slip_to_drive(
                        race().shared.physics_tables.get(),
                        race().course.numbers.get(),
                        view::<PlainFloat>(a[0], 0),
                        view::<PlainFloat>(a[2], 0),
                        view::<PlainFloat>(a[3], 0),
                        view::<PlainFloat>(a[4], 0),
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
                plain: |a| {
                    slip_ratio_size(
                        race().shared.car(),
                        race().shared.physics_tables.get(),
                        view::<PlainFloat>(a[0], 0),
                        view::<PlainFloat>(a[2], 0),
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
                plain: |_| {
                    signed_share(
                        race().shared.physics_tables.get(),
                        race().course.numbers.get(),
                    );
                },
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
                plain: |a| {
                    drive_forces_from_grips(
                        race().shared.car(),
                        race().shared.physics_tables.get(),
                        if a[1] == 0 {
                            &crate::race::owner::race()
                                .shared
                                .physics_tables
                                .get()
                                .wheel_grips
                        } else {
                            &crate::race::owner::race()
                                .shared
                                .physics_tables
                                .get()
                                .wheel_grips_b
                        },
                    );
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
                plain: |a| {
                    drive_forces_from_grips(
                        race().shared.car(),
                        race().shared.physics_tables.get(),
                        if a[1] == 0 {
                            &crate::race::owner::race()
                                .shared
                                .physics_tables
                                .get()
                                .wheel_grips
                        } else {
                            &crate::race::owner::race()
                                .shared
                                .physics_tables
                                .get()
                                .wheel_grips_b
                        },
                    );
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
                plain: |_| {
                    throttle_drive_limit(
                        race().shared.car(),
                        race().shared.physics_tables.get(),
                        race().course.numbers.get(),
                        crate::scene::owner::scene(),
                    );
                },
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
                plain: |_| {
                    push_both_drive_speeds(
                        race().shared.car(),
                        race().shared.physics_tables.get(),
                        race().course.numbers.get(),
                    );
                },
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
                plain: |_| {
                    push_drive_speed_a(race().shared.car(), race().shared.physics_tables.get());
                },
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
                plain: |_| {
                    brake_step_for_a(
                        race().shared.car(),
                        race().shared.physics_tables.get(),
                        race().course.numbers.get(),
                    );
                },
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
                plain: |_| {
                    brake_step_for_b(
                        race().shared.car(),
                        race().shared.physics_tables.get(),
                        race().course.numbers.get(),
                    );
                },
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
                plain: |_| {
                    handbrake_step_for_b(
                        race().shared.car(),
                        race().shared.physics_tables.get(),
                        race().course.numbers.get(),
                    );
                },
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
                plain: |a| {
                    let v: [u32; 3] = *view(a[0], 0);
                    a[1].set_word(0, handbrake_stop_status(v[0], v[1], v[2]));
                },
            },
            Block {
                name: "drive_wheels_step line 292",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x31A184, In::F), (0x31A190, In::F), (0x31A194, In::F)],
                    writes: &[(0x214234, Out::F), (0x214238, Out::F)],
                }],
                ulps: 0,
                plain: |_| slip_gaps(race().shared.car(), race().shared.physics_tables.get()),
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
                plain: |_| scale_axle_forces(race().shared.physics_tables.get()),
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
                plain: |_| {
                    side_slip_forces(
                        race().shared.car(),
                        race().shared.physics_tables.get(),
                        race().course.numbers.get(),
                        crate::scene::owner::scene(),
                    );
                },
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
                plain: |_| slip_side_share(race().shared.physics_tables.get()),
            },
            Block {
                name: "drive_wheels_step line 362",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x213ED4, In::F), (0x214BFC, In::F)],
                    writes: &[(0x2141D8, Out::F)],
                }],
                ulps: 0,
                plain: |_| spin_push_b(race().shared.physics_tables.get()),
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
                plain: |a| {
                    a[1].set_word(
                        0,
                        speed_over_ground(
                            race().shared.car(),
                            race().shared.physics_tables.get(),
                            crate::scene::owner::scene(),
                        ),
                    );
                },
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
                plain: |_| {
                    rolling_drag_forces(
                        race().shared.car(),
                        race().shared.physics_tables.get(),
                        crate::scene::owner::scene(),
                    );
                },
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
                plain: |_| {
                    share_wheel_forces(
                        race().shared.physics_tables.get(),
                        race().course.numbers.get(),
                    );
                },
            },
            Block {
                name: "drive_wheels_step line 426",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x213EDC, In::F), (0x31A184, In::F), (0x31A190, In::F)],
                    writes: &[(0x22401C, Out::F), (0x31A158, Out::F)],
                }],
                ulps: 0,
                plain: |_| {
                    slip_figure_a(
                        race().shared.car(),
                        race().shared.physics_tables.get(),
                        crate::scene::owner::scene(),
                    );
                },
            },
            Block {
                name: "drive_wheels_step line 436",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x213ED8, In::F), (0x31A17C, In::F)],
                    writes: &[(0x224020, Out::F)],
                }],
                ulps: 0,
                plain: |_| {
                    slip_figure_b(
                        race().shared.car(),
                        race().shared.physics_tables.get(),
                        crate::scene::owner::scene(),
                    );
                },
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
                plain: |_| slip_lengths(race().shared.car(), crate::scene::owner::scene()),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.drive_wheels_step", &blocks(), 550, 100);
    }
}
