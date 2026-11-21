//! The driven wheels' step: the slip shares from the car-frame wheel speeds (held 0 ..= 1 and
//! turned into a +-1 share), the clutch and the gear deciding the drive force (`drive_torque` 0
//! in neutral) and `physics::cap_drive_speed`; the drive speeds stepped by the drive force less
//! the wheels' drag and held toward the car's speed, with `physics::revs_from_wheels` on the
//! two paths; the `wheel_slips`; `physics::cap_reverse_inputs`, `physics::add_drive_forces` and
//! `physics::ease_low_speed_forces`; and the rest of the wheels' forces into the force records.
//!
//! The flow, the compares, the integer steps and the calls are Rust; each straight x87 run is
//! plain Rust over `X` in the named helpers below.
//!
//! Refuses (before anything) a gear outside -1 ..= 7 (`revs_from_wheels` refuses it; the gear is
//! not written here).

use crate::errors::{BadInput, Refused};
use crate::math::cell_x87::CellX87;
use crate::math::matrix3::extended;
use crate::math::x87;
use crate::math::x87::{abs_compare, abs_of, neg_rounded};
use crate::math::x87_float::compare;
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::{X, st, stf};
use crate::memory::session::FloatLike;
use crate::memory::session::PlainFloat;
use crate::physics::car_store::Car;
use crate::physics::flow::FloatView;
use crate::physics::flow::{ZERO_BITS as ZERO, c0, ease_zero, hold_high, hold_low, hold_low_bits};
use crate::physics::gearing::{GearPlaces, revs_from_wheels_at};
use crate::physics::tables_store::PhysicsTables;
use crate::physics::wheel_work::WheelWork;
use crate::race::numbers_store::Numbers;
use crate::race::owner_course::Course;
use crate::race::vars_store::RaceVars;
use std::cell::RefCell;

/// |a| below |b|, or one of them not a number.
fn abs_below(x: impl x87::Flt, y: impl x87::Flt) -> bool {
    status_below(abs_status([x.bits(), y.bits()]))
}

/// Copies the float `from` into `to` through the x87.
fn store_loaded(to: &PlainFloat, from: &PlainFloat) {
    to.set(x87::loaded(from));
}

/// The cells one wheel's slip step works on (`settle_slip`).
#[derive(Clone, Copy)]
struct SlipCells<'a> {
    drive: &'a PlainFloat,
    ratio: &'a PlainFloat,
    f1: &'a PlainFloat,
    f2: &'a PlainFloat,
}

/// The slip `slip` signed by the slip ratio's sign (returned), the drive speed (`drive`) less it
/// x (`spec_wheel` less 1), and the two force records' parts at `f1`, `f2`.
fn settle_slip(
    car: &Car,
    tables: &PhysicsTables,
    numbers: &Numbers,
    cells: &SlipCells,
    slip: u32,
) -> u32 {
    let SlipCells {
        drive,
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
    apply_slip_to_drive(tables, numbers, drive, signed.to_bits(), f1, f2);
    signed.to_bits()
}

/// The slip's first limit: |drive - speed| / `spec_wheel` into `ratio`; the slip is the lesser
/// of it and `force` (returned).
fn slip_limit(
    car: &Car,
    tables: &PhysicsTables,
    drive: &PlainFloat,
    ratio: &PlainFloat,
    force: u32,
) -> u32 {
    slip_ratio_size(car, tables, drive, ratio);
    let pick = if c0(ratio, force) {
        ratio.bits()
    } else {
        force
    };
    x87::loaded(pick).to_bits()
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
    work: &RefCell<WheelWork>,
    car: &Car,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    // The work value the drive steps pass through; each is stored before it is read.
    let scratch = crate::memory::session::PlainDword::new(0);
    let gear = car.gear.get();
    one_less_abs(
        &course.numbers,
        &car.crash_matrix[1],
        &physics_tables.drive_shares[0],
    );
    hold_low(&physics_tables.drive_shares[0], &course.numbers.tenth);
    hold_high(&physics_tables.drive_shares[0], &course.numbers.one);
    one_less_abs(
        &course.numbers,
        &car.crash_matrix[9],
        &physics_tables.drive_shares[1],
    );
    hold_low(&physics_tables.drive_shares[1], &course.numbers.tenth);
    hold_high(&physics_tables.drive_shares[1], &course.numbers.one);
    signed_share(physics_tables, &course.numbers);
    hold_low_bits(&physics_tables.drive_shares[2], ZERO);
    hold_high(&physics_tables.drive_shares[2], &course.numbers.one);
    if car.driven_wheels_off.get() == 0 {
        if gear == 0 {
            physics_tables.drive_torque.set(0.0);
        }
        crate::physics::cap_drive_speed::cap_drive_speed(car, physics_tables);
        // the grips of `wheel_grips`, or, braking, `wheel_grips_b`
        let mut grips = &physics_tables.wheel_grips;
        if car.brake.get() != 0 {
            grips = &physics_tables.wheel_grips_b;
        }
        drive_forces_from_grips(car, &course.numbers, physics_tables, work, grips);
        scratch.set(car.throttle.get().wrapping_add(0x100));
        throttle_drive_limit(car, physics_tables, &course.numbers, &scratch);
        hold_high(&physics_tables.drive_limits[1], &scratch);
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
            push_both_drive_speeds(car, physics_tables, &course.numbers);
        }
        if kind == 0 {
            push_drive_speed_a(car, &course.numbers, physics_tables);
        }
        revs(car, physics_tables, &course.numbers)?;
        // The wheels' drag on each drive speed.
        brake_step_for_a(car, physics_tables, &course.numbers);
        ease_zero(&car.drive_speed_a, physics_tables.wheel_speed_step.get());
        brake_step_for_b(car, physics_tables, &course.numbers);
        ease_zero(&car.drive_speed_b, physics_tables.wheel_speed_step.get());
        handbrake_step_for_b(car, physics_tables, &course.numbers);
        ease_zero(&car.drive_speed_b, physics_tables.wheel_speed_step.get());
        if car.handbrake.get() != 0 {
            let status =
                handbrake_stop_status(&car.drive_speed_a, &car.drive_speed_b, course.numbers.one);
            if status_below(status) {
                car.drive_speed_a.set(0.0);
                car.drive_speed_b.set(0.0);
            }
        }
        revs(car, physics_tables, &course.numbers)?;
        slip_gaps(car, work);
        crate::physics::cap_reverse_inputs::cap_reverse_inputs(car, work, &course.numbers);
        crate::physics::add_drive_forces::add_drive_forces(car, physics_tables, work);
        crate::physics::ease_low_speed_forces::ease_low_speed_forces(car, physics_tables, work);

        scale_axle_forces(physics_tables, work);
        if car.parts_state.get() != 0 {
            let half = course.numbers.half.get();
            let mut work = work.borrow_mut();
            for force in &mut work.drive_forces {
                *force = x87::mul(*force, half).to_bits();
            }
        }
        for (axle, (drive, forces)) in [
            (&car.drive_speed_a, [15, 16]),
            (&car.drive_speed_b, [17, 18]),
        ]
        .into_iter()
        .enumerate()
        {
            let force = work.borrow().drive_forces[axle];
            let slip = slip_limit(
                car,
                physics_tables,
                drive,
                &physics_tables.axle_slip_limits[axle],
                force,
            );
            let slip = settle_slip(
                car,
                physics_tables,
                &course.numbers,
                &SlipCells {
                    drive,
                    ratio: &physics_tables.axle_slip_limits[axle],
                    f1: &physics_tables.forces[forces[0]].force[2],
                    f2: &physics_tables.forces[forces[1]].force[2],
                },
                slip,
            );
            work.borrow_mut().axle_slips[axle] = slip;
        }
        // The car-frame side speed eased by the side grip.
        scratch.set_float(car.local_velocity[0].get());
        ease_zero(&scratch, physics_tables.drive_limits[2].get());
        side_slip_forces(car, physics_tables, work, &course.numbers, &scratch);
        store_loaded(&physics_tables.slip_limit, &physics_tables.drive_limits[0]);
        store_loaded(
            &physics_tables.wheel_spin_push[0],
            &physics_tables.handling[4],
        );
        let drive_slip = work.borrow().drive_slip[0];
        let status = abs_compare(drive_slip, &physics_tables.slip_limit);
        if status_above(status) {
            slip_side_share(&course.numbers, physics_tables, work);
        }
        spin_push_b(&course.numbers, physics_tables);
        physics_tables.forces[10].force[0].set(0.0);
        physics_tables.forces[10].force[2].set(0.0);
        // The rolling drag against the motion.
        let status = speed_over_ground(car, physics_tables, &scratch);
        if status_above(status) {
            rolling_drag_forces(car, physics_tables, &scratch);
        }
        if vars.run_kind.get() != 1 && (car.run_ticks.get().wrapping_sub(0x320) as i32) < 0 {
            car.drive_speed_a.set(0.0);
            car.drive_speed_b.set(0.0);
        }
        share_wheel_forces(physics_tables, &course.numbers);
        physics_tables.forces[19].force[0].set(0.0);
        physics_tables.forces[20].force[0].set(0.0);
    }
    // The wheels' slip figures.
    slip_figure_a(car, &course.numbers, scene);
    hold_work_float(scene, |work| &mut work.work_float_a);
    slip_figure_b(car, &course.numbers, scene);
    hold_work_float(scene, |work| &mut work.work_float_b);
    slip_lengths(car, scene);
    Ok(())
}

/// A work float raised to 0 when below it (or unordered).
fn hold_work_float(
    scene: &crate::scene::owner::Scene,
    pick: fn(&mut crate::scene::draw_store::DrawWork) -> &mut f32,
) {
    let mut work = scene.draw_work_mut();
    let value = pick(&mut work);
    let cell = crate::memory::session::PlainFloat::new(*value);
    hold_low_bits(&cell, ZERO);
    *value = cell.get();
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
    work: &RefCell<WheelWork>,
    car: &Car,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let gear = car.gear.get();
    if gear != 0 && !(-1..=7).contains(&gear) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    steps(course, vars, physics_tables, work, car, scene)
}

// ---- the x87 arithmetic ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[t + 0] = -(|[v + 0]| - [one])`
fn one_less_abs(numbers: &Numbers, v: &impl FloatLike, t: &impl FloatLike) {
    let t0 = v.ld();
    let t1 = t0.abs();
    let t2 = numbers.one.ld();
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

/// One straight x87 run as plain Rust over `X`.
fn slip_ratio(car: &Car, tables: &PhysicsTables, d: &PlainFloat, r: &PlainFloat) {
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
    d: &PlainFloat,
    s: u32,
    f1: &PlainFloat,
    f2: &PlainFloat,
) {
    let t0 = d.ld();
    let t1 = tables.spec_wheel.ld();
    let t2 = numbers.one.ld();
    let t3 = t1 - t2;
    let t4 = ld(s);
    let t5 = t3 * t4;
    let t6 = t0 - t5;
    d.st(t6);
    let t7 = ld(s);
    let t8 = numbers.step_time.ld();
    let t9 = t7 / t8;
    let t10 = numbers.half.ld();
    let t11 = t9 * t10;
    f1.st(t11);
    f2.st(t11);
}

/// One straight x87 run as plain Rust over `X`.
fn slip_ratio_size(car: &Car, tables: &PhysicsTables, d: &PlainFloat, r: &PlainFloat) {
    let t0 = d.ld();
    let t1 = car.local_velocity[2].ld();
    let t2 = t0 - t1;
    let t3 = tables.spec_wheel.ld();
    let t4 = t2 / t3;
    let t5 = t4.abs();
    r.st(t5);
}

/// One straight x87 run as plain Rust over `X`.
fn signed_share(tables: &PhysicsTables, numbers: &Numbers) {
    let t0 = tables.drive_shares[1].ld();
    let t1 = t0 + t0;
    let t2 = numbers.one.ld();
    let t3 = t1 - t2;
    tables.drive_shares[2].st(t3);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// A long formula, covered by the tests.
fn drive_forces_from_grips(
    car: &Car,
    numbers: &Numbers,
    tables: &PhysicsTables,
    work: &RefCell<WheelWork>,
    grips: &[PlainFloat; 5],
) {
    let t0 = grips[0].ld();
    let t1 = tables.forces[0].force[1].ld();
    let t2 = t0 * t1;
    let t3 = grips[1].ld();
    let t4 = tables.forces[1].force[1].ld();
    let t5 = t3 * t4;
    let t6 = t2 + t5;
    let t7 = -t6;
    let t8 = numbers.step_time.ld();
    let t9 = t7 * t8;
    let t10 = tables.weight.ld();
    let t11 = t9 * t10;
    let t12 = car.drive_efficiency[0].ld();
    let t13 = t11 * t12;
    work.borrow_mut().drive_forces[0] = st(t13);
    let t14 = grips[2].ld();
    let t15 = tables.forces[2].force[1].ld();
    let t16 = t14 * t15;
    let t17 = grips[3].ld();
    let t18 = tables.forces[3].force[1].ld();
    let t19 = t17 * t18;
    let t20 = t16 + t19;
    let t21 = -t20;
    let t22 = numbers.step_time.ld();
    let t23 = t21 * t22;
    let t24 = tables.weight.ld();
    let t25 = t23 * t24;
    let t26 = car.drive_efficiency[1].ld();
    let t27 = t25 * t26;
    work.borrow_mut().drive_forces[1] = st(t27);
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
    let t38 = numbers.step_time.ld();
    let t39 = t37 * t38;
    let t40 = tables.weight.ld();
    let t41 = t39 * t40;
    tables.drive_limits[0].st(t41);
}

/// One straight x87 run as plain Rust over `X`.
fn throttle_drive_limit(
    car: &Car,
    tables: &PhysicsTables,
    numbers: &Numbers,
    scratch: &crate::memory::session::PlainDword,
) {
    let t0 = int(scratch.get() as i32);
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
    let t11 = numbers.drive_speed_scale.ld();
    let t12 = t10 * t11;
    scratch.st(t12);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[drive_speed_a] = ((([drive_torque] * [half]) * [step_time]) * [spec_wheel]) + [drive_speed_a]`
/// `[drive_speed_b] = ((([drive_torque] * [half]) * [step_time]) * [spec_wheel]) + [drive_speed_b]`
fn push_both_drive_speeds(car: &Car, tables: &PhysicsTables, numbers: &Numbers) {
    let t0 = tables.drive_torque.ld();
    let t1 = numbers.half.ld();
    let t2 = t0 * t1;
    let t3 = numbers.step_time.ld();
    let t4 = t2 * t3;
    let t5 = tables.spec_wheel.ld();
    let t6 = t4 * t5;
    let t7 = car.drive_speed_a.ld();
    let t8 = t6 + t7;
    car.drive_speed_a.st(t8);
    let t9 = tables.drive_torque.ld();
    let t10 = numbers.half.ld();
    let t11 = t9 * t10;
    let t12 = numbers.step_time.ld();
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
fn push_drive_speed_a(car: &Car, numbers: &Numbers, tables: &PhysicsTables) {
    let t0 = tables.drive_torque.ld();
    let t1 = numbers.step_time.ld();
    let t2 = t0 * t1;
    let t3 = tables.spec_wheel.ld();
    let t4 = t2 * t3;
    let t5 = car.drive_speed_a.ld();
    let t6 = t4 + t5;
    car.drive_speed_a.st(t6);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// A long formula, covered by the tests.
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
    let t9 = numbers.step_time.ld();
    let t10 = t8 * t9;
    let t11 = car.brake_efficiency[0].ld();
    let t12 = t10 * t11;
    let t13 = tables.spec_wheel.ld();
    let t14 = t12 * t13;
    tables.wheel_speed_step.st(t14);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// A long formula, covered by the tests.
fn brake_step_for_b(car: &Car, tables: &PhysicsTables, numbers: &Numbers) {
    let t0 = tables.handling[1].ld();
    let t1 = tables.spec_drive[1].ld();
    let t2 = t0 * t1;
    let t3 = int(car.brake.get() as i32);
    let t4 = t2 * t3;
    let t5 = numbers.k256.ld();
    let t6 = t4 / t5;
    let t7 = numbers.step_time.ld();
    let t8 = t6 * t7;
    let t9 = car.brake_efficiency[1].ld();
    let t10 = t8 * t9;
    let t11 = tables.spec_wheel.ld();
    let t12 = t10 * t11;
    tables.wheel_speed_step.st(t12);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[wheel_speed_step] = (((([spec_drive] * int [handbrake]) / [k256]) * [step_time]) *
///  [spec_wheel]) * [weight]`
fn handbrake_step_for_b(car: &Car, tables: &PhysicsTables, numbers: &Numbers) {
    let t0 = tables.spec_drive[0].ld();
    let t1 = int(car.handbrake.get() as i32);
    let t2 = t0 * t1;
    let t3 = numbers.k256.ld();
    let t4 = t2 / t3;
    let t5 = numbers.step_time.ld();
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
    let t0 = ld(a.value());
    let t1 = ld(b.value());
    let t2 = t0 + t1;
    let t3 = t2.abs();
    let t4 = ld(limit.value());
    compare(t3, t4)
}

/// One straight x87 run as plain Rust over `X`.
fn slip_gaps(car: &Car, work: &RefCell<WheelWork>) {
    let t0 = car.drive_speed_b.ld();
    let t1 = car.local_velocity[2].ld();
    let t2 = t0 - t1;
    let t3 = t2.abs();
    let rear = st(t3);
    let t4 = car.drive_speed_a.ld();
    let t5 = car.local_velocity[2].ld();
    let t6 = t4 - t5;
    let t7 = t6.abs();
    let front = st(t7);
    work.borrow_mut().wheel_slips = [front, rear];
}

/// One straight x87 run as plain Rust over `X`.
fn scale_axle_forces(tables: &PhysicsTables, work: &RefCell<WheelWork>) {
    let t0 = tables.drive_limits[2].ld();
    let t1 = tables.drive_shares[0].ld();
    let t2 = t0 * t1;
    tables.drive_limits[2].st(t2);
    let mut work = work.borrow_mut();
    let t3 = ld(work.drive_forces[0]);
    let t4 = tables.drive_shares[1].ld();
    let t5 = t3 * t4;
    work.drive_forces[0] = st(t5);
    let t6 = ld(work.drive_forces[1]);
    let t7 = tables.drive_shares[1].ld();
    let t8 = t6 * t7;
    work.drive_forces[1] = st(t8);
}

/// One straight x87 run as plain Rust over `X`.
fn side_slip_forces(
    car: &Car,
    tables: &PhysicsTables,
    work: &RefCell<WheelWork>,
    numbers: &Numbers,
    scratch: &crate::memory::session::PlainDword,
) {
    let t0 = scratch.ld();
    let t1 = car.local_velocity[0].ld();
    let t2 = t0 - t1;
    work.borrow_mut().drive_slip[0] = st(t2);
    let t3 = numbers.step_time.ld();
    let t4 = t2 / t3;
    let t5 = numbers.quarter.ld();
    let t6 = t4 * t5;
    tables.forces[11].force[0].st(t6);
    tables.forces[12].force[0].st(t6);
    tables.forces[13].force[0].st(t6);
    tables.forces[14].force[0].st(t6);
}

/// One straight x87 run as plain Rust over `X`.
fn slip_side_share(numbers: &Numbers, tables: &PhysicsTables, work: &RefCell<WheelWork>) {
    let mut work = work.borrow_mut();
    let t0 = ld(work.drive_slip[0]);
    let t1 = t0.abs();
    let t2 = tables.slip_limit.ld();
    let t3 = t1 - t2;
    let t4 = numbers.slip_blend.ld();
    let t5 = t3 * t4;
    let t6 = tables.slip_limit.ld();
    let t7 = t5 + t6;
    work.drive_slip[1] = st(t7);
    let t8 = ld(work.drive_slip[1]);
    let t9 = ld(work.drive_slip[0]);
    let t10 = t8 / t9;
    let t11 = t10.abs();
    let t12 = tables.handling[4].ld();
    let t13 = t11 * t12;
    tables.wheel_spin_push[0].st(t13);
}

/// One straight x87 run as plain Rust over `X`.
fn spin_push_b(numbers: &Numbers, tables: &PhysicsTables) {
    let t0 = tables.handling[4].ld();
    let t1 = numbers.slip_share.ld();
    let t2 = t0 * t1;
    tables.wheel_spin_push[1].st(t2);
}

/// One straight x87 run as plain Rust over `X`.
fn speed_over_ground(
    car: &Car,
    tables: &PhysicsTables,
    scratch: &crate::memory::session::PlainDword,
) -> u16 {
    let t0 = car.local_velocity[0].ld();
    let t1 = t0 * t0;
    let t2 = car.local_velocity[2].ld();
    let t3 = t2 * t2;
    let t4 = t1 + t3;
    let t5 = t4.sqrt();
    scratch.st(t5);
    let t6 = tables.wheel_drag[1].ld();
    compare(t5, t6)
}

/// One straight x87 run as plain Rust over `X`.
fn rolling_drag_forces(
    car: &Car,
    tables: &PhysicsTables,
    scratch: &crate::memory::session::PlainDword,
) {
    let t0 = tables.wheel_drag[0].ld();
    let t1 = scratch.ld();
    let t2 = t0 / t1;
    let t3 = car.local_velocity[0].ld();
    let t4 = t2 * t3;
    let t5 = -t4;
    tables.forces[10].force[0].st(t5);
    let t6 = tables.wheel_drag[0].ld();
    let t7 = scratch.ld();
    let t8 = t6 / t7;
    let t9 = car.local_velocity[2].ld();
    let t10 = t8 * t9;
    let t11 = -t10;
    tables.forces[10].force[2].st(t11);
}

/// One straight x87 run as plain Rust over `X`.
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

/// One straight x87 run as plain Rust over `X`.
fn slip_figure_a(car: &Car, numbers: &Numbers, scene: &crate::scene::owner::Scene) {
    let t0 = car.drive_speed_a.ld();
    let t1 = car.local_velocity[2].ld();
    let t2 = t0 - t1;
    car.drive_slip_speed.st(t2);
    let t3 = t2.abs();
    let t4 = numbers.wheel_speed_gap.ld();
    let t5 = t3 - t4;
    scene.draw_work_mut().work_float_a = stf(t5);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[work_float_b] = |[local_velocity]| - [wheel_speed_margin]`
fn slip_figure_b(car: &Car, numbers: &Numbers, scene: &crate::scene::owner::Scene) {
    let t0 = car.local_velocity[0].ld();
    let t1 = t0.abs();
    let t2 = numbers.wheel_speed_margin.ld();
    let t3 = t1 - t2;
    scene.draw_work_mut().work_float_b = stf(t3);
}

/// One straight x87 run as plain Rust over `X`.
fn slip_lengths(car: &Car, scene: &crate::scene::owner::Scene) {
    let t0 = ld(scene.draw_work().work_float_a);
    let t1 = t0 * t0;
    let t2 = ld(scene.draw_work().work_float_b);
    let t3 = t2 * t2;
    let t4 = t1 + t3;
    let t5 = t4.sqrt();
    car.slip_speed.st(t5);
    let t6 = car.local_velocity[0].ld();
    let t7 = t6.abs();
    scene.draw_work_mut().work_float_b = stf(t7);
    let t8 = car.drive_speed_a.ld();
    let t9 = car.local_velocity[2].ld();
    let t10 = t8 - t9;
    let t11 = t10.abs();
    scene.draw_work_mut().work_float_a = stf(t11);
    let t12 = ld(scene.draw_work().work_float_a);
    let t13 = t12 * t12;
    let t14 = ld(scene.draw_work().work_float_b);
    let t15 = t14 * t14;
    let t16 = t13 + t15;
    let t17 = t16.sqrt();
    car.skid_grip[0].st(t17);
    let t18 = car.drive_speed_b.ld();
    let t19 = car.local_velocity[2].ld();
    let t20 = t18 - t19;
    let t21 = t20.abs();
    scene.draw_work_mut().work_float_a = stf(t21);
    let t22 = ld(scene.draw_work().work_float_a);
    let t23 = t22 * t22;
    let t24 = ld(scene.draw_work().work_float_b);
    let t25 = t24 * t24;
    let t26 = t23 + t25;
    let t27 = t26.sqrt();
    car.skid_grip[1].st(t27);
}

#[cfg(test)]
mod tests;
