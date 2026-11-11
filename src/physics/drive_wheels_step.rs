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
//! original's instructions being `asm_reference` in the tests.
//!
//! Caller: `physics::car_physics_step`, every physics step. Refuses (before anything) a gear
//! outside -1 ..= 7 (`revs_from_wheels` would; the gear is not written here).

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87::{abs_compare, abs_of, neg_rounded};
use crate::math::x87_float::X;
use crate::math::x87_float::compare;
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::physics::car_store::Car;
use crate::physics::flow::{ZERO_BITS as ZERO, c0, ease_zero, hold_high, hold_low, hold_low_bits};
use crate::physics::gearing::{GearPlaces, revs_from_wheels_at};
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::scene::matrix3::extended;
use crate::session::{Cell32, Float};

/// |a| below |b|, or one of them not a number (`fld b; fabs; fld a; fabs; fcompp`, C0).
fn abs_below(x: u32, y: u32) -> bool {
    status_below(abs_status([x, y]))
}

/// `fld from; fstp to`.
fn store_loaded(to: &Float, from: &Float) {
    to.set(x87::loaded(from));
}

/// The cells one wheel's slip step works on (`settle_slip`).
#[derive(Clone, Copy)]
struct SlipCells<'a> {
    drive: &'a Float,
    slip: &'a Float,
    ratio: &'a Float,
    f1: &'a Float,
    f2: &'a Float,
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
    let s = abs_of(slip.bits());
    let signed = if c0(ratio.bits(), ZERO) {
        neg_rounded(extended(s.to_bits()))
    } else {
        s
    };
    slip.set_bits(signed.to_bits());
    apply_slip_to_drive(tables, numbers, drive, slip, f1, f2);
}

/// The slip's first limit (0x43E58B / 0x43E649): |drive - speed| / `spec_wheel` into
/// `ratio`, and `slip` the lesser of it and `force` (by `fcompp`).
fn slip_limit(
    car: &Car,
    tables: &PhysicsTables,
    drive: &Float,
    ratio: &Float,
    force: &Float,
    slip: &Float,
) {
    slip_ratio_size(car, tables, drive, ratio);
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
fn steps(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let drawing = &scene.drawing;
    let gear = race.shared.car.gear.get();
    one_less_abs(
        &race.course,
        &race.shared.car.crash_matrix[1],
        &race.shared.physics_tables.drive_shares[0],
    );
    hold_low(
        &race.shared.physics_tables.drive_shares[0],
        &race.course.numbers.tenth,
    );
    hold_high(
        &race.shared.physics_tables.drive_shares[0],
        &race.course.numbers.one,
    );
    one_less_abs(
        &race.course,
        &race.shared.car.crash_matrix[9],
        &race.shared.physics_tables.drive_shares[1],
    );
    hold_low(
        &race.shared.physics_tables.drive_shares[1],
        &race.course.numbers.tenth,
    );
    hold_high(
        &race.shared.physics_tables.drive_shares[1],
        &race.course.numbers.one,
    );
    signed_share(&race.shared.physics_tables, &race.course.numbers);
    hold_low_bits(&race.shared.physics_tables.drive_shares[2], ZERO);
    hold_high(
        &race.shared.physics_tables.drive_shares[2],
        &race.course.numbers.one,
    );
    if race.shared.car.driven_wheels_off.get() == 0 {
        if gear == 0 {
            race.shared.physics_tables.drive_torque.set_bits(0);
        }
        crate::physics::cap_drive_speed::cap_drive_speed(
            &race.shared.car,
            &race.shared.physics_tables,
            scene,
        );
        race.shared.physics_tables.wheels_driven.set(0);
        // the grips at +0 or, braking, +0x14: `wheel_grips_b`
        let mut grips = &race.shared.physics_tables.wheel_grips;
        if race.shared.car.brake.get() != 0 {
            grips = &race.shared.physics_tables.wheel_grips_b;
            race.shared.physics_tables.wheels_driven.set(1);
        }
        drive_forces_from_grips(&race.shared.car, &race.shared.physics_tables, grips);
        drawing
            .scene
            .scratch
            .set(race.shared.car.throttle.get().wrapping_add(0x100));
        throttle_drive_limit(
            &race.shared.car,
            &race.shared.physics_tables,
            &race.course.numbers,
            scene,
        );
        hold_high(
            &race.shared.physics_tables.drive_limits[1],
            &drawing.scene.scratch,
        );
        let pick = if abs_below(
            race.shared.physics_tables.drive_limits[1].bits(),
            race.shared.physics_tables.drive_limits[0].bits(),
        ) {
            &race.shared.physics_tables.drive_limits[0]
        } else {
            &race.shared.physics_tables.drive_limits[1]
        };
        store_loaded(&race.shared.physics_tables.drive_limits[2], pick);
        let kind = race.shared.car.drive_kind.get();
        if gear != 0 && kind == 1 {
            push_both_drive_speeds(
                &race.shared.car,
                &race.shared.physics_tables,
                &race.course.numbers,
            );
        }
        if kind == 0 {
            push_drive_speed_a(&race.shared.car, &race.shared.physics_tables);
        }
        revs(
            &race.shared.car,
            &race.shared.physics_tables,
            &race.course.numbers,
        )?;
        // 0x43E2C2: the wheels' drag on each drive speed.
        brake_step_for_a(
            &race.shared.car,
            &race.shared.physics_tables,
            &race.course.numbers,
        );
        ease_zero(
            &race.shared.car.drive_speed_a,
            &race.shared.physics_tables.wheel_speed_step,
        );
        brake_step_for_b(
            &race.shared.car,
            &race.shared.physics_tables,
            &race.course.numbers,
        );
        ease_zero(
            &race.shared.car.drive_speed_b,
            &race.shared.physics_tables.wheel_speed_step,
        );
        handbrake_step_for_b(
            &race.shared.car,
            &race.shared.physics_tables,
            &race.course.numbers,
        );
        ease_zero(
            &race.shared.car.drive_speed_b,
            &race.shared.physics_tables.wheel_speed_step,
        );
        if race.shared.car.handbrake.get() != 0 {
            let status = handbrake_stop_status([
                race.shared.car.drive_speed_a.bits(),
                race.shared.car.drive_speed_b.bits(),
                race.course.numbers.one.bits(),
            ]);
            if status_below(status) {
                race.shared.car.drive_speed_a.set_bits(0);
                race.shared.car.drive_speed_b.set_bits(0);
            }
        }
        revs(
            &race.shared.car,
            &race.shared.physics_tables,
            &race.course.numbers,
        )?;
        slip_gaps(&race.shared.car, &race.shared.physics_tables);
        crate::physics::cap_reverse_inputs::cap_reverse_inputs(
            &race.shared.car,
            &race.shared.physics_tables,
            &race.course.numbers,
        );
        crate::physics::add_drive_forces::add_drive_forces(
            &race.shared.car,
            &race.shared.physics_tables,
            scene,
        );
        crate::physics::ease_low_speed_forces::ease_low_speed_forces(
            &race.shared.car,
            &race.shared.physics_tables,
        );

        scale_axle_forces(&race.shared.physics_tables);
        if race.shared.car.parts_state.get() != 0 {
            race.shared.physics_tables.drive_forces[0].set(x87::mul(
                &race.shared.physics_tables.drive_forces[0],
                &race.course.numbers.half,
            ));
            race.shared.physics_tables.drive_forces[1].set(x87::mul(
                &race.shared.physics_tables.drive_forces[1],
                &race.course.numbers.half,
            ));
        }
        slip_limit(
            &race.shared.car,
            &race.shared.physics_tables,
            &race.shared.car.drive_speed_a,
            &race.shared.physics_tables.axle_slip_limits[0],
            &race.shared.physics_tables.drive_forces[0],
            &race.shared.physics_tables.axle_slips[0],
        );
        settle_slip(
            &race.shared.car,
            &race.shared.physics_tables,
            &race.course.numbers,
            &SlipCells {
                drive: &race.shared.car.drive_speed_a,
                slip: &race.shared.physics_tables.axle_slips[0],
                ratio: &race.shared.physics_tables.axle_slip_limits[0],
                f1: &race.shared.physics_tables.forces[15].force[2],
                f2: &race.shared.physics_tables.forces[16].force[2],
            },
        );
        slip_limit(
            &race.shared.car,
            &race.shared.physics_tables,
            &race.shared.car.drive_speed_b,
            &race.shared.physics_tables.axle_slip_limits[1],
            &race.shared.physics_tables.drive_forces[1],
            &race.shared.physics_tables.axle_slips[1],
        );
        settle_slip(
            &race.shared.car,
            &race.shared.physics_tables,
            &race.course.numbers,
            &SlipCells {
                drive: &race.shared.car.drive_speed_b,
                slip: &race.shared.physics_tables.axle_slips[1],
                ratio: &race.shared.physics_tables.axle_slip_limits[1],
                f1: &race.shared.physics_tables.forces[17].force[2],
                f2: &race.shared.physics_tables.forces[18].force[2],
            },
        );
        // 0x43E707: the car-frame side speed eased by the side grip.
        drawing
            .scene
            .scratch
            .set(race.shared.car.local_velocity[0].bits());
        ease_zero(
            &drawing.scene.scratch,
            &race.shared.physics_tables.drive_limits[2],
        );
        side_slip_forces(
            &race.shared.car,
            &race.shared.physics_tables,
            &race.course.numbers,
            scene,
        );
        store_loaded(
            &race.shared.physics_tables.slip_limit,
            &race.shared.physics_tables.drive_limits[0],
        );
        store_loaded(
            &race.shared.physics_tables.wheel_spin_push[0],
            &race.shared.physics_tables.handling[4],
        );
        let status = abs_compare(
            race.shared.physics_tables.drive_slip[0].bits(),
            race.shared.physics_tables.slip_limit.bits(),
        );
        if status_above(status) {
            slip_side_share(&race.shared.physics_tables);
        }
        spin_push_b(&race.shared.physics_tables);
        race.shared.physics_tables.forces[10].force[0].set_bits(0);
        race.shared.physics_tables.forces[10].force[2].set_bits(0);
        // 0x43E855: the rolling drag against the motion.
        let status = speed_over_ground(&race.shared.car, &race.shared.physics_tables, scene);
        if status_above(status) {
            rolling_drag_forces(&race.shared.car, &race.shared.physics_tables, scene);
        }
        if race.shared.vars.run_kind.get() != 1
            && (race.shared.car.run_ticks.get().wrapping_sub(0x320) as i32) < 0
        {
            race.shared.car.drive_speed_a.set_bits(0);
            race.shared.car.drive_speed_b.set_bits(0);
        }
        share_wheel_forces(&race.shared.physics_tables, &race.course.numbers);
        race.shared.physics_tables.forces[19].force[0].set_bits(0);
        race.shared.physics_tables.forces[20].force[0].set_bits(0);
    }
    // 0x43E957: the wheels' slip figures.
    slip_figure_a(&race.shared.car, &race.shared.physics_tables, scene);
    hold_low_bits(&drawing.scene.work_float_a, ZERO);
    slip_figure_b(&race.shared.car, &race.shared.physics_tables, scene);
    hold_low_bits(&drawing.scene.work_float_b, ZERO);
    slip_lengths(&race.shared.car, scene);
    Ok(())
}

/// `revs_from_wheels` (the gear checked before).
fn revs(car: &Car, tables: &PhysicsTables, numbers: &Numbers) -> Result<(), Refused> {
    { revs_from_wheels_at(numbers, &GearPlaces::game(car, tables)) }?;
    Ok(())
}

pub fn drive_wheels_step(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let gear = race.shared.car.gear.get();
    if gear != 0 && !(-1..=7).contains(&(gear as i32)) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    steps(race, scene)
}

// ---- the arithmetic that was x87 assembly ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[t + 0] = -(|[v + 0]| - [one])`
fn one_less_abs(course: &Course, v: &dyn Cell32, t: &dyn Cell32) {
    let t0 = ld(v.bits());
    let t1 = t0.abs();
    let t2 = ld(course.numbers.one.bits());
    let t3 = t1 - t2;
    let t4 = -t3;
    t.set_bits(st(t4));
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
fn slip_ratio(car: &Car, tables: &PhysicsTables, d: &dyn Cell32, r: &dyn Cell32) {
    let t0 = ld(d.bits());
    let t1 = ld(car.local_velocity[2].bits());
    let t2 = t0 - t1;
    let t3 = ld(tables.spec_wheel.bits());
    let t4 = t2 / t3;
    r.set_bits(st(t4));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[d + 0] = [d + 0] - (([spec_wheel] - [one]) * [s + 0])`
/// `[f1 + 0] = ([s + 0] / [step_time]) * [half]`
/// `[f2 + 0] = ([s + 0] / [step_time]) * [half]`
fn apply_slip_to_drive(
    tables: &PhysicsTables,
    numbers: &Numbers,
    d: &dyn Cell32,
    s: &dyn Cell32,
    f1: &dyn Cell32,
    f2: &dyn Cell32,
) {
    let t0 = ld(d.bits());
    let t1 = ld(tables.spec_wheel.bits());
    let t2 = ld(numbers.one.bits());
    let t3 = t1 - t2;
    let t4 = ld(s.bits());
    let t5 = t3 * t4;
    let t6 = t0 - t5;
    d.set_bits(st(t6));
    let t7 = ld(s.bits());
    let t8 = ld(tables.step_time.bits());
    let t9 = t7 / t8;
    let t10 = ld(numbers.half.bits());
    let t11 = t9 * t10;
    f1.set_bits(st(t11));
    f2.set_bits(st(t11));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[r + 0] = |(([d + 0] - [rc + 0x31A184]) / [spec_wheel])|`
fn slip_ratio_size(car: &Car, tables: &PhysicsTables, d: &dyn Cell32, r: &dyn Cell32) {
    let t0 = ld(d.bits());
    let t1 = ld(car.local_velocity[2].bits());
    let t2 = t0 - t1;
    let t3 = ld(tables.spec_wheel.bits());
    let t4 = t2 / t3;
    let t5 = t4.abs();
    r.set_bits(st(t5));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2140B8] = ([rc + 0x2140B4] + [rc + 0x2140B4]) - [one]`
fn signed_share(tables: &PhysicsTables, numbers: &Numbers) {
    let t0 = ld(tables.drive_shares[1].bits());
    let t1 = t0 + t0;
    let t2 = ld(numbers.one.bits());
    let t3 = t1 - t2;
    tables.drive_shares[2].set_bits(st(t3));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (a long formula: see the instructions in the tests' `asm_reference`)
/// (a long formula: see the instructions in the tests' `asm_reference`)
/// (a long formula: see the instructions in the tests' `asm_reference`)
fn drive_forces_from_grips(car: &Car, tables: &PhysicsTables, grips: &[Float; 5]) {
    let t0 = ld(grips[0].bits());
    let t1 = ld(tables.forces[0].force[1].bits());
    let t2 = t0 * t1;
    let t3 = ld(grips[1].bits());
    let t4 = ld(tables.forces[1].force[1].bits());
    let t5 = t3 * t4;
    let t6 = t2 + t5;
    let t7 = -t6;
    let t8 = ld(tables.step_time.bits());
    let t9 = t7 * t8;
    let t10 = ld(tables.weight.bits());
    let t11 = t9 * t10;
    let t12 = ld(car.drive_efficiency[0].bits());
    let t13 = t11 * t12;
    tables.drive_forces[0].set_bits(st(t13));
    let t14 = ld(grips[2].bits());
    let t15 = ld(tables.forces[2].force[1].bits());
    let t16 = t14 * t15;
    let t17 = ld(grips[3].bits());
    let t18 = ld(tables.forces[3].force[1].bits());
    let t19 = t17 * t18;
    let t20 = t16 + t19;
    let t21 = -t20;
    let t22 = ld(tables.step_time.bits());
    let t23 = t21 * t22;
    let t24 = ld(tables.weight.bits());
    let t25 = t23 * t24;
    let t26 = ld(car.drive_efficiency[1].bits());
    let t27 = t25 * t26;
    tables.drive_forces[1].set_bits(st(t27));
    let t28 = ld(tables.wheel_loads[0].bits());
    let t29 = ld(tables.wheel_loads[1].bits());
    let t30 = t28 + t29;
    let t31 = ld(tables.wheel_loads[2].bits());
    let t32 = t30 + t31;
    let t33 = ld(tables.wheel_loads[3].bits());
    let t34 = t32 + t33;
    let t35 = ld(tables.forces[0].force[1].bits());
    let t36 = t34 * t35;
    let t37 = -t36;
    let t38 = ld(tables.step_time.bits());
    let t39 = t37 * t38;
    let t40 = ld(tables.weight.bits());
    let t41 = t39 * t40;
    tables.drive_limits[0].set_bits(st(t41));
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
    let t0 = int(drawing.scene.scratch.bits() as i32);
    let t1 = ld(numbers.k256.bits());
    let t2 = t0 / t1;
    let t3 = ld(car.local_velocity[0].bits());
    let t4 = ld(tables.wheel_load.bits());
    let t5 = t3 * t4;
    let t6 = t5.abs();
    let t7 = t2 * t6;
    let t8 = ld(tables.weight.bits());
    let t9 = t7 * t8;
    tables.drive_limits[1].set_bits(st(t9));
    let t10 = ld(tables.drive_limits[0].bits());
    let t11 = ld(tables.drive_speed_scale.bits());
    let t12 = t10 * t11;
    drawing.scene.scratch.set_bits(st(t12));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[drive_speed_a] = ((([drive_torque] * [half]) * [step_time]) * [spec_wheel]) + [drive_speed_a]`
/// `[drive_speed_b] = ((([drive_torque] * [half]) * [step_time]) * [spec_wheel]) + [drive_speed_b]`
fn push_both_drive_speeds(car: &Car, tables: &PhysicsTables, numbers: &Numbers) {
    let t0 = ld(tables.drive_torque.bits());
    let t1 = ld(numbers.half.bits());
    let t2 = t0 * t1;
    let t3 = ld(tables.step_time.bits());
    let t4 = t2 * t3;
    let t5 = ld(tables.spec_wheel.bits());
    let t6 = t4 * t5;
    let t7 = ld(car.drive_speed_a.bits());
    let t8 = t6 + t7;
    car.drive_speed_a.set_bits(st(t8));
    let t9 = ld(tables.drive_torque.bits());
    let t10 = ld(numbers.half.bits());
    let t11 = t9 * t10;
    let t12 = ld(tables.step_time.bits());
    let t13 = t11 * t12;
    let t14 = ld(tables.spec_wheel.bits());
    let t15 = t13 * t14;
    let t16 = ld(car.drive_speed_b.bits());
    let t17 = t15 + t16;
    car.drive_speed_b.set_bits(st(t17));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[drive_speed_a] = (([drive_torque] * [step_time]) * [spec_wheel]) + [drive_speed_a]`
fn push_drive_speed_a(car: &Car, tables: &PhysicsTables) {
    let t0 = ld(tables.drive_torque.bits());
    let t1 = ld(tables.step_time.bits());
    let t2 = t0 * t1;
    let t3 = ld(tables.spec_wheel.bits());
    let t4 = t2 * t3;
    let t5 = ld(car.drive_speed_a.bits());
    let t6 = t4 + t5;
    car.drive_speed_a.set_bits(st(t6));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (a long formula: see the instructions in the tests' `asm_reference`)
fn brake_step_for_a(car: &Car, tables: &PhysicsTables, numbers: &Numbers) {
    let t0 = X(1.0);
    let t1 = ld(tables.handling[1].bits());
    let t2 = t0 - t1;
    let t3 = ld(tables.spec_drive[1].bits());
    let t4 = t2 * t3;
    let t5 = int(car.brake.bits() as i32);
    let t6 = t4 * t5;
    let t7 = ld(numbers.k256.bits());
    let t8 = t6 / t7;
    let t9 = ld(tables.step_time.bits());
    let t10 = t8 * t9;
    let t11 = ld(car.brake_efficiency[0].bits());
    let t12 = t10 * t11;
    let t13 = ld(tables.spec_wheel.bits());
    let t14 = t12 * t13;
    tables.wheel_speed_step.set_bits(st(t14));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (a long formula: see the instructions in the tests' `asm_reference`)
fn brake_step_for_b(car: &Car, tables: &PhysicsTables, numbers: &Numbers) {
    let t0 = ld(tables.handling[1].bits());
    let t1 = ld(tables.spec_drive[1].bits());
    let t2 = t0 * t1;
    let t3 = int(car.brake.bits() as i32);
    let t4 = t2 * t3;
    let t5 = ld(numbers.k256.bits());
    let t6 = t4 / t5;
    let t7 = ld(tables.step_time.bits());
    let t8 = t6 * t7;
    let t9 = ld(car.brake_efficiency[1].bits());
    let t10 = t8 * t9;
    let t11 = ld(tables.spec_wheel.bits());
    let t12 = t10 * t11;
    tables.wheel_speed_step.set_bits(st(t12));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[wheel_speed_step] = (((([spec_drive] * int [handbrake]) / [k256]) * [step_time]) * [spec_wheel]) * [weight]`
fn handbrake_step_for_b(car: &Car, tables: &PhysicsTables, numbers: &Numbers) {
    let t0 = ld(tables.spec_drive[0].bits());
    let t1 = int(car.handbrake.bits() as i32);
    let t2 = t0 * t1;
    let t3 = ld(numbers.k256.bits());
    let t4 = t2 / t3;
    let t5 = ld(tables.step_time.bits());
    let t6 = t4 * t5;
    let t7 = ld(tables.spec_wheel.bits());
    let t8 = t6 * t7;
    let t9 = ld(tables.weight.bits());
    let t10 = t8 * t9;
    tables.wheel_speed_step.set_bits(st(t10));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `status [s + 0] = status`
fn handbrake_stop_status(v: [u32; 3]) -> u16 {
    let t0 = ld(v[0]);
    let t1 = ld(v[1]);
    let t2 = t0 + t1;
    let t3 = t2.abs();
    let t4 = ld(v[2]);
    compare(t3, t4)
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214238] = |([drive_speed_b] - [rc + 0x31A184])|`
/// `[wheel_slips] = |([drive_speed_a] - [rc + 0x31A184])|`
fn slip_gaps(car: &Car, tables: &PhysicsTables) {
    let t0 = ld(car.drive_speed_b.bits());
    let t1 = ld(car.local_velocity[2].bits());
    let t2 = t0 - t1;
    let t3 = t2.abs();
    tables.wheel_slips[1].set_bits(st(t3));
    let t4 = ld(car.drive_speed_a.bits());
    let t5 = ld(car.local_velocity[2].bits());
    let t6 = t4 - t5;
    let t7 = t6.abs();
    tables.wheel_slips[0].set_bits(st(t7));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214244] = [rc + 0x214244] * [drive_shares]`
/// `[drive_forces] = [drive_forces] * [rc + 0x2140B4]`
/// `[rc + 0x21424C] = [rc + 0x21424C] * [rc + 0x2140B4]`
fn scale_axle_forces(tables: &PhysicsTables) {
    let t0 = ld(tables.drive_limits[2].bits());
    let t1 = ld(tables.drive_shares[0].bits());
    let t2 = t0 * t1;
    tables.drive_limits[2].set_bits(st(t2));
    let t3 = ld(tables.drive_forces[0].bits());
    let t4 = ld(tables.drive_shares[1].bits());
    let t5 = t3 * t4;
    tables.drive_forces[0].set_bits(st(t5));
    let t6 = ld(tables.drive_forces[1].bits());
    let t7 = ld(tables.drive_shares[1].bits());
    let t8 = t6 * t7;
    tables.drive_forces[1].set_bits(st(t8));
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
    let t0 = ld(scene.drawing.scene.scratch.bits());
    let t1 = ld(car.local_velocity[0].bits());
    let t2 = t0 - t1;
    tables.drive_slip[0].set_bits(st(t2));
    let t3 = ld(tables.step_time.bits());
    let t4 = t2 / t3;
    let t5 = ld(numbers.quarter.bits());
    let t6 = t4 * t5;
    tables.forces[11].force[0].set_bits(st(t6));
    tables.forces[12].force[0].set_bits(st(t6));
    tables.forces[13].force[0].set_bits(st(t6));
    tables.forces[14].force[0].set_bits(st(t6));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214254] = ((|[drive_slip]| - [slip_limit]) * [slip_blend]) + [slip_limit]`
/// `[wheel_spin_push] = |([rc + 0x214254] / [drive_slip])| * [rc + 0x214BFC]`
fn slip_side_share(tables: &PhysicsTables) {
    let t0 = ld(tables.drive_slip[0].bits());
    let t1 = t0.abs();
    let t2 = ld(tables.slip_limit.bits());
    let t3 = t1 - t2;
    let t4 = ld(tables.slip_blend.bits());
    let t5 = t3 * t4;
    let t6 = ld(tables.slip_limit.bits());
    let t7 = t5 + t6;
    tables.drive_slip[1].set_bits(st(t7));
    let t8 = ld(tables.drive_slip[1].bits());
    let t9 = ld(tables.drive_slip[0].bits());
    let t10 = t8 / t9;
    let t11 = t10.abs();
    let t12 = ld(tables.handling[4].bits());
    let t13 = t11 * t12;
    tables.wheel_spin_push[0].set_bits(st(t13));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2141D8] = [rc + 0x214BFC] * [slip_share]`
fn spin_push_b(tables: &PhysicsTables) {
    let t0 = ld(tables.handling[4].bits());
    let t1 = ld(tables.slip_share.bits());
    let t2 = t0 * t1;
    tables.wheel_spin_push[1].set_bits(st(t2));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[scratch] = sqrt((([local_velocity] * [local_velocity]) + ([rc + 0x31A184] * [rc + 0x31A184])))`
/// `status [s + 0] = status`
fn speed_over_ground(car: &Car, tables: &PhysicsTables, scene: &crate::scene::owner::Scene) -> u16 {
    let t0 = ld(car.local_velocity[0].bits());
    let t1 = t0 * t0;
    let t2 = ld(car.local_velocity[2].bits());
    let t3 = t2 * t2;
    let t4 = t1 + t3;
    let t5 = t4.sqrt();
    scene.drawing.scene.scratch.set_bits(st(t5));
    let t6 = ld(tables.wheel_drag[1].bits());
    compare(t5, t6)
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2148B0] = -(([wheel_drag] / [scratch]) * [local_velocity])`
/// `[rc + 0x2148B8] = -(([wheel_drag] / [scratch]) * [rc + 0x31A184])`
fn rolling_drag_forces(car: &Car, tables: &PhysicsTables, scene: &crate::scene::owner::Scene) {
    let drawing = &scene.drawing;
    let t0 = ld(tables.wheel_drag[0].bits());
    let t1 = ld(drawing.scene.scratch.bits());
    let t2 = t0 / t1;
    let t3 = ld(car.local_velocity[0].bits());
    let t4 = t2 * t3;
    let t5 = -t4;
    tables.forces[10].force[0].set_bits(st(t5));
    let t6 = ld(tables.wheel_drag[0].bits());
    let t7 = ld(drawing.scene.scratch.bits());
    let t8 = t6 / t7;
    let t9 = ld(car.local_velocity[2].bits());
    let t10 = t8 * t9;
    let t11 = -t10;
    tables.forces[10].force[2].set_bits(st(t11));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2148D0] = ([rc + 0x2149D0] * [half]) + [rc + 0x2148D0]`
/// `[rc + 0x2148F0] = ([rc + 0x2149D0] * [half]) + [rc + 0x2148F0]`
/// `[rc + 0x214910] = ([rc + 0x2149F0] * [half]) + [rc + 0x214910]`
/// `[rc + 0x214930] = ([rc + 0x2149F0] * [half]) + [rc + 0x214930]`
fn share_wheel_forces(tables: &PhysicsTables, numbers: &Numbers) {
    let t0 = ld(tables.forces[19].force[0].bits());
    let t1 = ld(numbers.half.bits());
    let t2 = t0 * t1;
    let t3 = ld(tables.forces[11].force[0].bits());
    let t4 = t2 + t3;
    tables.forces[11].force[0].set_bits(st(t4));
    let t5 = ld(tables.forces[19].force[0].bits());
    let t6 = ld(numbers.half.bits());
    let t7 = t5 * t6;
    let t8 = ld(tables.forces[12].force[0].bits());
    let t9 = t7 + t8;
    tables.forces[12].force[0].set_bits(st(t9));
    let t10 = ld(tables.forces[20].force[0].bits());
    let t11 = ld(numbers.half.bits());
    let t12 = t10 * t11;
    let t13 = ld(tables.forces[13].force[0].bits());
    let t14 = t12 + t13;
    tables.forces[13].force[0].set_bits(st(t14));
    let t15 = ld(tables.forces[20].force[0].bits());
    let t16 = ld(numbers.half.bits());
    let t17 = t15 * t16;
    let t18 = ld(tables.forces[14].force[0].bits());
    let t19 = t17 + t18;
    tables.forces[14].force[0].set_bits(st(t19));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[drive_slip_speed] = [drive_speed_a] - [rc + 0x31A184]`
/// `[work_float_a] = |([drive_speed_a] - [rc + 0x31A184])| - [wheel_speed_gap]`
fn slip_figure_a(car: &Car, tables: &PhysicsTables, scene: &crate::scene::owner::Scene) {
    let t0 = ld(car.drive_speed_a.bits());
    let t1 = ld(car.local_velocity[2].bits());
    let t2 = t0 - t1;
    car.drive_slip_speed.set_bits(st(t2));
    let t3 = t2.abs();
    let t4 = ld(tables.wheel_speed_gap.bits());
    let t5 = t3 - t4;
    scene.drawing.scene.work_float_a.set_bits(st(t5));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[work_float_b] = |[local_velocity]| - [wheel_speed_margin]`
fn slip_figure_b(car: &Car, tables: &PhysicsTables, scene: &crate::scene::owner::Scene) {
    let t0 = ld(car.local_velocity[0].bits());
    let t1 = t0.abs();
    let t2 = ld(tables.wheel_speed_margin.bits());
    let t3 = t1 - t2;
    scene.drawing.scene.work_float_b.set_bits(st(t3));
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
    let t0 = ld(drawing.scene.work_float_a.bits());
    let t1 = t0 * t0;
    let t2 = ld(drawing.scene.work_float_b.bits());
    let t3 = t2 * t2;
    let t4 = t1 + t3;
    let t5 = t4.sqrt();
    car.slip_speed.set_bits(st(t5));
    let t6 = ld(car.local_velocity[0].bits());
    let t7 = t6.abs();
    drawing.scene.work_float_b.set_bits(st(t7));
    let t8 = ld(car.drive_speed_a.bits());
    let t9 = ld(car.local_velocity[2].bits());
    let t10 = t8 - t9;
    let t11 = t10.abs();
    drawing.scene.work_float_a.set_bits(st(t11));
    let t12 = ld(drawing.scene.work_float_a.bits());
    let t13 = t12 * t12;
    let t14 = ld(drawing.scene.work_float_b.bits());
    let t15 = t14 * t14;
    let t16 = t13 + t15;
    let t17 = t16.sqrt();
    car.skid_grip[0].set_bits(st(t17));
    let t18 = ld(car.drive_speed_b.bits());
    let t19 = ld(car.local_velocity[2].bits());
    let t20 = t18 - t19;
    let t21 = t20.abs();
    drawing.scene.work_float_a.set_bits(st(t21));
    let t22 = ld(drawing.scene.work_float_a.bits());
    let t23 = t22 * t22;
    let t24 = ld(drawing.scene.work_float_b.bits());
    let t25 = t24 * t24;
    let t26 = t23 + t25;
    let t27 = t26.sqrt();
    car.skid_grip[1].set_bits(st(t27));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::race::owner::race;
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
                        &race().shared.car,
                        &race().shared.physics_tables,
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
                        &race().shared.physics_tables,
                        &race().course.numbers,
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
                        &race().shared.car,
                        &race().shared.physics_tables,
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
                plain: |_| signed_share(&race().shared.physics_tables, &race().course.numbers),
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
                    drive_forces_from_grips(
                        &race().shared.car,
                        &race().shared.physics_tables,
                        if a[1] == 0 {
                            &crate::race::owner::race().shared.physics_tables.wheel_grips
                        } else {
                            &crate::race::owner::race()
                                .shared
                                .physics_tables
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
                reference: |a| unsafe { ref_6(a[0], a[1]) },
                plain: |a| {
                    drive_forces_from_grips(
                        &race().shared.car,
                        &race().shared.physics_tables,
                        if a[1] == 0 {
                            &crate::race::owner::race().shared.physics_tables.wheel_grips
                        } else {
                            &crate::race::owner::race()
                                .shared
                                .physics_tables
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
                reference: |a| unsafe { ref_7(a[0]) },
                plain: |_| {
                    throttle_drive_limit(
                        &race().shared.car,
                        &race().shared.physics_tables,
                        &race().course.numbers,
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
                reference: |a| unsafe { ref_8(a[0]) },
                plain: |_| {
                    push_both_drive_speeds(
                        &race().shared.car,
                        &race().shared.physics_tables,
                        &race().course.numbers,
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
                reference: |a| unsafe { ref_9(a[0]) },
                plain: |_| push_drive_speed_a(&race().shared.car, &race().shared.physics_tables),
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
                plain: |_| {
                    brake_step_for_a(
                        &race().shared.car,
                        &race().shared.physics_tables,
                        &race().course.numbers,
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
                reference: |a| unsafe { ref_11(a[0]) },
                plain: |_| {
                    brake_step_for_b(
                        &race().shared.car,
                        &race().shared.physics_tables,
                        &race().course.numbers,
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
                reference: |a| unsafe { ref_12(a[0]) },
                plain: |_| {
                    handbrake_step_for_b(
                        &race().shared.car,
                        &race().shared.physics_tables,
                        &race().course.numbers,
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
                plain: |_| slip_gaps(&race().shared.car, &race().shared.physics_tables),
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
                plain: |_| scale_axle_forces(&race().shared.physics_tables),
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
                plain: |_| {
                    side_slip_forces(
                        &race().shared.car,
                        &race().shared.physics_tables,
                        &race().course.numbers,
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
                reference: |a| unsafe { ref_17(a[0]) },
                plain: |_| slip_side_share(&race().shared.physics_tables),
            },
            Block {
                name: "drive_wheels_step line 362",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x213ED4, In::F), (0x214BFC, In::F)],
                    writes: &[(0x2141D8, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_18(a[0]) },
                plain: |_| spin_push_b(&race().shared.physics_tables),
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
                plain: |a| {
                    a[1].set_word(
                        0,
                        speed_over_ground(
                            &race().shared.car,
                            &race().shared.physics_tables,
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
                reference: |a| unsafe { ref_20(a[0]) },
                plain: |_| {
                    rolling_drag_forces(
                        &race().shared.car,
                        &race().shared.physics_tables,
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
                reference: |a| unsafe { ref_21(a[0]) },
                plain: |_| {
                    share_wheel_forces(&race().shared.physics_tables, &race().course.numbers);
                },
            },
            Block {
                name: "drive_wheels_step line 426",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x213EDC, In::F), (0x31A184, In::F), (0x31A190, In::F)],
                    writes: &[(0x22401C, Out::F), (0x31A158, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_22(a[0]) },
                plain: |_| {
                    slip_figure_a(
                        &race().shared.car,
                        &race().shared.physics_tables,
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
                reference: |a| unsafe { ref_23(a[0]) },
                plain: |_| {
                    slip_figure_b(
                        &race().shared.car,
                        &race().shared.physics_tables,
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
                reference: |a| unsafe { ref_24(a[0]) },
                plain: |_| slip_lengths(&race().shared.car, crate::scene::owner::scene()),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
