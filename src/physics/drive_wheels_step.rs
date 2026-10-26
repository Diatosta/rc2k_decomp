//! `physics::drive_wheels_step` (0x43DFB1): the driven wheels' step: the slip shares from
//! the car-frame wheel speeds (+0xB8, +0xD8; held 0 ..= 1 and turned into a ±1 share), the
//! clutch (+0x250) and the gear deciding the drive force (`flt_614750` 0 in neutral),
//! `physics::cap_drive_speed`; the drive speeds stepped by the drive force less the
//! wheels' drag and held toward the car's speed, `physics::revs_from_wheels` (twice, on
//! the two paths); the slips `flt_614234` / `614238`; `physics::cap_reverse_inputs`,
//! `physics::add_drive_forces`, `physics::ease_low_speed_forces`; and the rest of the
//! wheels' forces into the force records.
//!
//! The flow, the compares, the integer steps and the calls are Rust; each straight x87
//! run is plain Rust over `X` (`math/x87_float.rs`) in the `x87_N` functions below, the
//! original's instructions being `asm_reference` in the tests.
//!
//! In:  nothing
//! Out: eax and ebx as the instructions and calls leave them; the rest kept; no flags
//!      read (the caller calls on)
//!
//! Caller: `sub_43B635`, every physics step. Refuses (before anything) a gear outside -1
//! ..= 7 (`revs_from_wheels` would; the gear is not written here).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_float::X;
use crate::math::x87_float::compare;
use crate::math::x87_run::{abs_compare, abs_of, neg_rounded};
use crate::physics::car_store::CAR;
use crate::physics::flow::{c0, ease_zero, hold_high, hold_low};
use crate::physics::gearing::{GearPlaces, revs_from_wheels_at};
use crate::physics::tables_store::PHYSICS;
use crate::race::numbers_store::NUMBERS;
use crate::race::vars_store::RACE_VARS;
use crate::scene::draw_store::DRAW;
use crate::scene::matrix3::extended;
use crate::scene::x87_mem::{fild, fld, fnstsw, fstp};
use crate::session::Float;

/// `fld v; fabs; fsub one; fchs; fstp to` (0x43DFB1, 0x43E00B): 1 less |v|.
fn one_less_abs(v: &Float, to: &Float) {
    x87_1(
        v.addr() as usize,
        crate::race::store::asm_base() as usize,
        to.addr() as usize,
    );
}

/// `fld b; fabs; fld a; fabs; fcompp; fnstsw`: C0, |a| below |b| or unordered.
fn abs_below(x: u32, y: u32) -> bool {
    let (v, mut status) = ([x, y], 0u16);
    x87_2(
        (crate::pointers::game_address(v.as_ptr())) as usize,
        crate::pointers::game_address(&raw mut status) as usize,
    );
    status & 0x100 != 0
}

/// `fld from; fstp to`.
fn store_loaded(to: &Float, from: &Float) {
    to.set_bits(x87::loaded(from.bits()));
}

/// 0x43E5CF .. 0x43E643 (and the second wheel's at 0x43E68D): the slip `s` (at `slip`)
/// signed by the slip ratio's sign, the drive speed (`drive`) less it x (`dword_614BA8`
/// - 1), and the two force records' parts at `f1`, `f2`.
fn settle_slip(drive: &Float, slip: &Float, ratio: &Float, f1: &Float, f2: &Float) {
    x87_3(
        drive.addr() as usize,
        crate::race::store::asm_base() as usize,
        ratio.addr() as usize,
    );
    let s = abs_of(slip.bits());
    let signed = if c0(ratio.bits(), NUMBERS.zero.bits()) {
        neg_rounded(extended(s))
    } else {
        s
    };
    slip.set_bits(signed);
    x87_4(
        drive.addr() as usize,
        crate::race::store::asm_base() as usize,
        slip.addr() as usize,
        f1.addr() as usize,
        f2.addr() as usize,
    );
}

/// The slip's first limit (0x43E58B / 0x43E649): |drive - speed| / `dword_614BA8` into
/// `ratio`, and `slip` the lesser of it and `force` (by `fcompp`).
fn slip_limit(drive: &Float, ratio: &Float, force: &Float, slip: &Float) {
    x87_5(
        drive.addr() as usize,
        crate::race::store::asm_base() as usize,
        ratio.addr() as usize,
    );
    let pick = if c0(ratio.bits(), force.bits()) {
        ratio
    } else {
        force
    };
    store_loaded(slip, pick);
}

/// The steps.
fn steps() -> Result<(), Refused> {
    let gear = CAR.gear.get();
    one_less_abs(&CAR.crash_matrix[1], &PHYSICS.drive_shares[0]);
    hold_low(&PHYSICS.drive_shares[0], &NUMBERS.tenth);
    hold_high(&PHYSICS.drive_shares[0], &NUMBERS.one);
    one_less_abs(&CAR.crash_matrix[9], &PHYSICS.drive_shares[1]);
    hold_low(&PHYSICS.drive_shares[1], &NUMBERS.tenth);
    hold_high(&PHYSICS.drive_shares[1], &NUMBERS.one);
    x87_6(crate::race::store::asm_base() as usize);
    hold_low(&PHYSICS.drive_shares[2], &NUMBERS.zero);
    hold_high(&PHYSICS.drive_shares[2], &NUMBERS.one);
    if CAR.driven_wheels_off.get() == 0 {
        if gear == 0 {
            PHYSICS.drive_torque.set_bits(0);
        }
        crate::physics::cap_drive_speed::cap_drive_speed();
        PHYSICS.wheels_driven.set(0);
        let mut offset = 0;
        if CAR.brake.get() != 0 {
            offset = 0x14;
            PHYSICS.wheels_driven.set(1);
        }
        x87_7(crate::race::store::asm_base() as usize, offset as usize);
        DRAW.scratch.set(CAR.throttle.get().wrapping_add(0x100));
        x87_8(crate::race::store::asm_base() as usize);
        hold_high(&PHYSICS.drive_limits[1], &DRAW.scratch);
        let pick = if abs_below(
            PHYSICS.drive_limits[1].bits(),
            PHYSICS.drive_limits[0].bits(),
        ) {
            &PHYSICS.drive_limits[0]
        } else {
            &PHYSICS.drive_limits[1]
        };
        store_loaded(&PHYSICS.drive_limits[2], pick);
        let kind = CAR.drive_kind.get();
        if gear != 0 && kind == 1 {
            x87_9(crate::race::store::asm_base() as usize);
        }
        if kind == 0 {
            x87_10(crate::race::store::asm_base() as usize);
        }
        revs()?;
        // 0x43E2C2: the wheels' drag on each drive speed.
        x87_11(crate::race::store::asm_base() as usize);
        ease_zero(
            &crate::physics::car_store::CAR.drive_speed_a,
            &PHYSICS.wheel_speed_step,
        );
        x87_12(crate::race::store::asm_base() as usize);
        ease_zero(
            &crate::physics::car_store::CAR.drive_speed_b,
            &PHYSICS.wheel_speed_step,
        );
        x87_13(crate::race::store::asm_base() as usize);
        ease_zero(
            &crate::physics::car_store::CAR.drive_speed_b,
            &PHYSICS.wheel_speed_step,
        );
        if CAR.handbrake.get() != 0 {
            let (v, mut status) = (
                [
                    CAR.drive_speed_a.bits(),
                    CAR.drive_speed_b.bits(),
                    NUMBERS.one.bits(),
                ],
                0u16,
            );
            x87_14(
                (crate::pointers::game_address(v.as_ptr())) as usize,
                crate::pointers::game_address(&raw mut status) as usize,
            );
            if status & 0x100 != 0 {
                CAR.drive_speed_a.set_bits(0);
                CAR.drive_speed_b.set_bits(0);
            }
        }
        revs()?;
        x87_15(crate::race::store::asm_base() as usize);
        unsafe {
            crate::physics::cap_reverse_inputs::cap_reverse_inputs();
            crate::physics::add_drive_forces::add_drive_forces();
            crate::physics::ease_low_speed_forces::ease_low_speed_forces();
        }
        x87_16(crate::race::store::asm_base() as usize);
        if CAR.parts_state.get() != 0 {
            PHYSICS.drive_forces[0].set_bits(x87::mul(
                crate::physics::tables_store::PHYSICS.drive_forces[0].bits(),
                NUMBERS.half.bits(),
            ));
            PHYSICS.drive_forces[1].set_bits(x87::mul(
                crate::physics::tables_store::PHYSICS.drive_forces[1].bits(),
                NUMBERS.half.bits(),
            ));
        }
        slip_limit(
            &CAR.drive_speed_a,
            &PHYSICS.axle_slip_limits[0],
            &PHYSICS.drive_forces[0],
            &PHYSICS.axle_slips[0],
        );
        settle_slip(
            &CAR.drive_speed_a,
            &PHYSICS.axle_slips[0],
            &PHYSICS.axle_slip_limits[0],
            &PHYSICS.forces[15].force[2],
            &PHYSICS.forces[16].force[2],
        );
        slip_limit(
            &CAR.drive_speed_b,
            &PHYSICS.axle_slip_limits[1],
            &PHYSICS.drive_forces[1],
            &PHYSICS.axle_slips[1],
        );
        settle_slip(
            &CAR.drive_speed_b,
            &PHYSICS.axle_slips[1],
            &PHYSICS.axle_slip_limits[1],
            &PHYSICS.forces[17].force[2],
            &PHYSICS.forces[18].force[2],
        );
        // 0x43E707: the car-frame side speed eased by the side grip.
        DRAW.scratch.set(CAR.local_velocity[0].bits());
        ease_zero(&DRAW.scratch, &PHYSICS.drive_limits[2]);
        x87_17(crate::race::store::asm_base() as usize);
        store_loaded(&PHYSICS.slip_limit, &PHYSICS.drive_limits[0]);
        store_loaded(&PHYSICS.wheel_spin_push[0], &PHYSICS.handling[4]);
        let status = abs_compare(PHYSICS.drive_slip[0].bits(), PHYSICS.slip_limit.bits());
        if status & 0x4100 == 0 {
            x87_18(crate::race::store::asm_base() as usize);
        }
        x87_19(crate::race::store::asm_base() as usize);
        PHYSICS.forces[10].force[0].set_bits(0);
        PHYSICS.forces[10].force[2].set_bits(0);
        // 0x43E855: the rolling drag against the motion.
        let mut status = 0u16;
        x87_20(
            crate::race::store::asm_base() as usize,
            crate::pointers::game_address(&raw mut status) as usize,
        );
        if status & 0x4100 == 0 {
            x87_21(crate::race::store::asm_base() as usize);
        }
        if RACE_VARS.run_kind.get() != 1 && (CAR.run_ticks.get().wrapping_sub(0x320) as i32) < 0 {
            CAR.drive_speed_a.set_bits(0);
            CAR.drive_speed_b.set_bits(0);
        }
        x87_22(crate::race::store::asm_base() as usize);
        PHYSICS.forces[19].force[0].set_bits(0);
        PHYSICS.forces[20].force[0].set_bits(0);
    }
    // 0x43E957: the wheels' slip figures.
    x87_23(crate::race::store::asm_base() as usize);
    hold_low(&DRAW.work_float_a, &NUMBERS.zero);
    x87_24(crate::race::store::asm_base() as usize);
    hold_low(&DRAW.work_float_b, &NUMBERS.zero);
    x87_25(crate::race::store::asm_base() as usize);
    Ok(())
}

/// `revs_from_wheels` (the gear checked before; the ebx it leaves is read by no one).
fn revs() -> Result<(), Refused> {
    { revs_from_wheels_at(&GearPlaces::game()) }?;
    Ok(())
}

/// The Rust version. The eax and ebx it leaves are read by none of its callers.
pub fn drive_wheels_step() -> Result<(), Refused> {
    let gear = CAR.gear.get();
    if gear != 0 && !(-1..=7).contains(&(gear as i32)) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    steps()
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[t + 0] = -(|[v + 0]| - [rc + 0x1C5D38])`
fn x87_1(v: usize, rc: usize, t: usize) {
    let t0 = fld(v, 0);
    let t1 = t0.abs();
    let t2 = fld(rc, 0x1C5D38);
    let t3 = t1 - t2;
    let t4 = -t3;
    fstp(t, 0, t4);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `status [s + 0] = status`
fn x87_2(v: usize, s: usize) {
    let t0 = fld(v, 4);
    let t1 = t0.abs();
    let t2 = fld(v, 0);
    let t3 = t2.abs();
    let t4 = compare(t3, t1);
    fnstsw(s, 0, t4);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[r + 0] = ([d + 0] - [rc + 0x31A184]) / [rc + 0x214BA8]`
fn x87_3(d: usize, rc: usize, r: usize) {
    let t0 = fld(d, 0);
    let t1 = fld(rc, 0x31A184);
    let t2 = t0 - t1;
    let t3 = fld(rc, 0x214BA8);
    let t4 = t2 / t3;
    fstp(r, 0, t4);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[d + 0] = [d + 0] - (([rc + 0x214BA8] - [rc + 0x1C5D38]) * [s + 0])`
/// `[f1 + 0] = ([s + 0] / [rc + 0x21388C]) * [rc + 0x1C5E40]`
/// `[f2 + 0] = ([s + 0] / [rc + 0x21388C]) * [rc + 0x1C5E40]`
fn x87_4(d: usize, rc: usize, s: usize, f1: usize, f2: usize) {
    let t0 = fld(d, 0);
    let t1 = fld(rc, 0x214BA8);
    let t2 = fld(rc, 0x1C5D38);
    let t3 = t1 - t2;
    let t4 = fld(s, 0);
    let t5 = t3 * t4;
    let t6 = t0 - t5;
    fstp(d, 0, t6);
    let t7 = fld(s, 0);
    let t8 = fld(rc, 0x21388C);
    let t9 = t7 / t8;
    let t10 = fld(rc, 0x1C5E40);
    let t11 = t9 * t10;
    fstp(f1, 0, t11);
    fstp(f2, 0, t11);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[r + 0] = |(([d + 0] - [rc + 0x31A184]) / [rc + 0x214BA8])|`
fn x87_5(d: usize, rc: usize, r: usize) {
    let t0 = fld(d, 0);
    let t1 = fld(rc, 0x31A184);
    let t2 = t0 - t1;
    let t3 = fld(rc, 0x214BA8);
    let t4 = t2 / t3;
    let t5 = t4.abs();
    fstp(r, 0, t5);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2140B8] = ([rc + 0x2140B4] + [rc + 0x2140B4]) - [rc + 0x1C5D38]`
fn x87_6(rc: usize) {
    let t0 = fld(rc, 0x2140B4);
    let t1 = t0 + t0;
    let t2 = fld(rc, 0x1C5D38);
    let t3 = t1 - t2;
    fstp(rc, 0x2140B8, t3);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (a long formula: see the instructions in the tests' `asm_reference`)
/// (a long formula: see the instructions in the tests' `asm_reference`)
/// (a long formula: see the instructions in the tests' `asm_reference`)
fn x87_7(rc: usize, e: usize) {
    let t0 = fld(rc, e + 0x214298);
    let t1 = fld(rc, 0x214774);
    let t2 = t0 * t1;
    let t3 = fld(rc, e + 0x21429C);
    let t4 = fld(rc, 0x214794);
    let t5 = t3 * t4;
    let t6 = t2 + t5;
    let t7 = -t6;
    let t8 = fld(rc, 0x21388C);
    let t9 = t7 * t8;
    let t10 = fld(rc, 0x214170);
    let t11 = t9 * t10;
    let t12 = fld(rc, 0x31A3D8);
    let t13 = t11 * t12;
    fstp(rc, 0x214248, t13);
    let t14 = fld(rc, e + 0x2142A0);
    let t15 = fld(rc, 0x2147B4);
    let t16 = t14 * t15;
    let t17 = fld(rc, e + 0x2142A4);
    let t18 = fld(rc, 0x2147D4);
    let t19 = t17 * t18;
    let t20 = t16 + t19;
    let t21 = -t20;
    let t22 = fld(rc, 0x21388C);
    let t23 = t21 * t22;
    let t24 = fld(rc, 0x214170);
    let t25 = t23 * t24;
    let t26 = fld(rc, 0x31A3DC);
    let t27 = t25 * t26;
    fstp(rc, 0x21424C, t27);
    let t28 = fld(rc, 0x214284);
    let t29 = fld(rc, 0x214288);
    let t30 = t28 + t29;
    let t31 = fld(rc, 0x21428C);
    let t32 = t30 + t31;
    let t33 = fld(rc, 0x214290);
    let t34 = t32 + t33;
    let t35 = fld(rc, 0x214774);
    let t36 = t34 * t35;
    let t37 = -t36;
    let t38 = fld(rc, 0x21388C);
    let t39 = t37 * t38;
    let t40 = fld(rc, 0x214170);
    let t41 = t39 * t40;
    fstp(rc, 0x21423C, t41);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214240] = ((int [rc + 0x22400C] / [rc + 0x1C5E2C]) * |([rc + 0x31A17C] * [rc + 0x214268])|) * [rc + 0x214170]`
/// `[rc + 0x22400C] = [rc + 0x21423C] * [rc + 0x213EC4]`
fn x87_8(rc: usize) {
    let t0 = fild(rc, 0x22400C);
    let t1 = fld(rc, 0x1C5E2C);
    let t2 = t0 / t1;
    let t3 = fld(rc, 0x31A17C);
    let t4 = fld(rc, 0x214268);
    let t5 = t3 * t4;
    let t6 = t5.abs();
    let t7 = t2 * t6;
    let t8 = fld(rc, 0x214170);
    let t9 = t7 * t8;
    fstp(rc, 0x214240, t9);
    let t10 = fld(rc, 0x21423C);
    let t11 = fld(rc, 0x213EC4);
    let t12 = t10 * t11;
    fstp(rc, 0x22400C, t12);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A190] = ((([rc + 0x214750] * [rc + 0x1C5E40]) * [rc + 0x21388C]) * [rc + 0x214BA8]) + [rc + 0x31A190]`
/// `[rc + 0x31A194] = ((([rc + 0x214750] * [rc + 0x1C5E40]) * [rc + 0x21388C]) * [rc + 0x214BA8]) + [rc + 0x31A194]`
fn x87_9(rc: usize) {
    let t0 = fld(rc, 0x214750);
    let t1 = fld(rc, 0x1C5E40);
    let t2 = t0 * t1;
    let t3 = fld(rc, 0x21388C);
    let t4 = t2 * t3;
    let t5 = fld(rc, 0x214BA8);
    let t6 = t4 * t5;
    let t7 = fld(rc, 0x31A190);
    let t8 = t6 + t7;
    fstp(rc, 0x31A190, t8);
    let t9 = fld(rc, 0x214750);
    let t10 = fld(rc, 0x1C5E40);
    let t11 = t9 * t10;
    let t12 = fld(rc, 0x21388C);
    let t13 = t11 * t12;
    let t14 = fld(rc, 0x214BA8);
    let t15 = t13 * t14;
    let t16 = fld(rc, 0x31A194);
    let t17 = t15 + t16;
    fstp(rc, 0x31A194, t17);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A190] = (([rc + 0x214750] * [rc + 0x21388C]) * [rc + 0x214BA8]) + [rc + 0x31A190]`
fn x87_10(rc: usize) {
    let t0 = fld(rc, 0x214750);
    let t1 = fld(rc, 0x21388C);
    let t2 = t0 * t1;
    let t3 = fld(rc, 0x214BA8);
    let t4 = t2 * t3;
    let t5 = fld(rc, 0x31A190);
    let t6 = t4 + t5;
    fstp(rc, 0x31A190, t6);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (a long formula: see the instructions in the tests' `asm_reference`)
fn x87_11(rc: usize) {
    let t0 = X(1.0);
    let t1 = fld(rc, 0x214BF0);
    let t2 = t0 - t1;
    let t3 = fld(rc, 0x214BB0);
    let t4 = t2 * t3;
    let t5 = fild(rc, 0x31A614);
    let t6 = t4 * t5;
    let t7 = fld(rc, 0x1C5E2C);
    let t8 = t6 / t7;
    let t9 = fld(rc, 0x21388C);
    let t10 = t8 * t9;
    let t11 = fld(rc, 0x31A3D0);
    let t12 = t10 * t11;
    let t13 = fld(rc, 0x214BA8);
    let t14 = t12 * t13;
    fstp(rc, 0x21422C, t14);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (a long formula: see the instructions in the tests' `asm_reference`)
fn x87_12(rc: usize) {
    let t0 = fld(rc, 0x214BF0);
    let t1 = fld(rc, 0x214BB0);
    let t2 = t0 * t1;
    let t3 = fild(rc, 0x31A614);
    let t4 = t2 * t3;
    let t5 = fld(rc, 0x1C5E2C);
    let t6 = t4 / t5;
    let t7 = fld(rc, 0x21388C);
    let t8 = t6 * t7;
    let t9 = fld(rc, 0x31A3D4);
    let t10 = t8 * t9;
    let t11 = fld(rc, 0x214BA8);
    let t12 = t10 * t11;
    fstp(rc, 0x21422C, t12);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x21422C] = (((([rc + 0x214BAC] * int [rc + 0x31A618]) / [rc + 0x1C5E2C]) * [rc + 0x21388C]) * [rc + 0x214BA8]) * [rc + 0x214170]`
fn x87_13(rc: usize) {
    let t0 = fld(rc, 0x214BAC);
    let t1 = fild(rc, 0x31A618);
    let t2 = t0 * t1;
    let t3 = fld(rc, 0x1C5E2C);
    let t4 = t2 / t3;
    let t5 = fld(rc, 0x21388C);
    let t6 = t4 * t5;
    let t7 = fld(rc, 0x214BA8);
    let t8 = t6 * t7;
    let t9 = fld(rc, 0x214170);
    let t10 = t8 * t9;
    fstp(rc, 0x21422C, t10);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `status [s + 0] = status`
fn x87_14(v: usize, s: usize) {
    let t0 = fld(v, 0);
    let t1 = fld(v, 4);
    let t2 = t0 + t1;
    let t3 = t2.abs();
    let t4 = fld(v, 8);
    let t5 = compare(t3, t4);
    fnstsw(s, 0, t5);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214238] = |([rc + 0x31A194] - [rc + 0x31A184])|`
/// `[rc + 0x214234] = |([rc + 0x31A190] - [rc + 0x31A184])|`
fn x87_15(rc: usize) {
    let t0 = fld(rc, 0x31A194);
    let t1 = fld(rc, 0x31A184);
    let t2 = t0 - t1;
    let t3 = t2.abs();
    fstp(rc, 0x214238, t3);
    let t4 = fld(rc, 0x31A190);
    let t5 = fld(rc, 0x31A184);
    let t6 = t4 - t5;
    let t7 = t6.abs();
    fstp(rc, 0x214234, t7);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214244] = [rc + 0x214244] * [rc + 0x2140B0]`
/// `[rc + 0x214248] = [rc + 0x214248] * [rc + 0x2140B4]`
/// `[rc + 0x21424C] = [rc + 0x21424C] * [rc + 0x2140B4]`
fn x87_16(rc: usize) {
    let t0 = fld(rc, 0x214244);
    let t1 = fld(rc, 0x2140B0);
    let t2 = t0 * t1;
    fstp(rc, 0x214244, t2);
    let t3 = fld(rc, 0x214248);
    let t4 = fld(rc, 0x2140B4);
    let t5 = t3 * t4;
    fstp(rc, 0x214248, t5);
    let t6 = fld(rc, 0x21424C);
    let t7 = fld(rc, 0x2140B4);
    let t8 = t6 * t7;
    fstp(rc, 0x21424C, t8);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214250] = [rc + 0x22400C] - [rc + 0x31A17C]`
/// `[rc + 0x2148D0] = (([rc + 0x22400C] - [rc + 0x31A17C]) / [rc + 0x21388C]) * [rc + 0x1C5DA4]`
/// `[rc + 0x2148F0] = (([rc + 0x22400C] - [rc + 0x31A17C]) / [rc + 0x21388C]) * [rc + 0x1C5DA4]`
/// `[rc + 0x214910] = (([rc + 0x22400C] - [rc + 0x31A17C]) / [rc + 0x21388C]) * [rc + 0x1C5DA4]`
/// `[rc + 0x214930] = (([rc + 0x22400C] - [rc + 0x31A17C]) / [rc + 0x21388C]) * [rc + 0x1C5DA4]`
fn x87_17(rc: usize) {
    let t0 = fld(rc, 0x22400C);
    let t1 = fld(rc, 0x31A17C);
    let t2 = t0 - t1;
    fstp(rc, 0x214250, t2);
    let t3 = fld(rc, 0x21388C);
    let t4 = t2 / t3;
    let t5 = fld(rc, 0x1C5DA4);
    let t6 = t4 * t5;
    fstp(rc, 0x2148D0, t6);
    fstp(rc, 0x2148F0, t6);
    fstp(rc, 0x214910, t6);
    fstp(rc, 0x214930, t6);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214254] = ((|[rc + 0x214250]| - [rc + 0x214264]) * [rc + 0x213ED0]) + [rc + 0x214264]`
/// `[rc + 0x2141D4] = |([rc + 0x214254] / [rc + 0x214250])| * [rc + 0x214BFC]`
fn x87_18(rc: usize) {
    let t0 = fld(rc, 0x214250);
    let t1 = t0.abs();
    let t2 = fld(rc, 0x214264);
    let t3 = t1 - t2;
    let t4 = fld(rc, 0x213ED0);
    let t5 = t3 * t4;
    let t6 = fld(rc, 0x214264);
    let t7 = t5 + t6;
    fstp(rc, 0x214254, t7);
    let t8 = fld(rc, 0x214254);
    let t9 = fld(rc, 0x214250);
    let t10 = t8 / t9;
    let t11 = t10.abs();
    let t12 = fld(rc, 0x214BFC);
    let t13 = t11 * t12;
    fstp(rc, 0x2141D4, t13);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2141D8] = [rc + 0x214BFC] * [rc + 0x213ED4]`
fn x87_19(rc: usize) {
    let t0 = fld(rc, 0x214BFC);
    let t1 = fld(rc, 0x213ED4);
    let t2 = t0 * t1;
    fstp(rc, 0x2141D8, t2);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x22400C] = sqrt((([rc + 0x31A17C] * [rc + 0x31A17C]) + ([rc + 0x31A184] * [rc + 0x31A184])))`
/// `status [s + 0] = status`
fn x87_20(rc: usize, s: usize) {
    let t0 = fld(rc, 0x31A17C);
    let t1 = t0 * t0;
    let t2 = fld(rc, 0x31A184);
    let t3 = t2 * t2;
    let t4 = t1 + t3;
    let t5 = t4.sqrt();
    fstp(rc, 0x22400C, t5);
    let t6 = fld(rc, 0x214764);
    let t7 = compare(t5, t6);
    fnstsw(s, 0, t7);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2148B0] = -(([rc + 0x214760] / [rc + 0x22400C]) * [rc + 0x31A17C])`
/// `[rc + 0x2148B8] = -(([rc + 0x214760] / [rc + 0x22400C]) * [rc + 0x31A184])`
fn x87_21(rc: usize) {
    let t0 = fld(rc, 0x214760);
    let t1 = fld(rc, 0x22400C);
    let t2 = t0 / t1;
    let t3 = fld(rc, 0x31A17C);
    let t4 = t2 * t3;
    let t5 = -t4;
    fstp(rc, 0x2148B0, t5);
    let t6 = fld(rc, 0x214760);
    let t7 = fld(rc, 0x22400C);
    let t8 = t6 / t7;
    let t9 = fld(rc, 0x31A184);
    let t10 = t8 * t9;
    let t11 = -t10;
    fstp(rc, 0x2148B8, t11);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x2148D0] = ([rc + 0x2149D0] * [rc + 0x1C5E40]) + [rc + 0x2148D0]`
/// `[rc + 0x2148F0] = ([rc + 0x2149D0] * [rc + 0x1C5E40]) + [rc + 0x2148F0]`
/// `[rc + 0x214910] = ([rc + 0x2149F0] * [rc + 0x1C5E40]) + [rc + 0x214910]`
/// `[rc + 0x214930] = ([rc + 0x2149F0] * [rc + 0x1C5E40]) + [rc + 0x214930]`
fn x87_22(rc: usize) {
    let t0 = fld(rc, 0x2149D0);
    let t1 = fld(rc, 0x1C5E40);
    let t2 = t0 * t1;
    let t3 = fld(rc, 0x2148D0);
    let t4 = t2 + t3;
    fstp(rc, 0x2148D0, t4);
    let t5 = fld(rc, 0x2149D0);
    let t6 = fld(rc, 0x1C5E40);
    let t7 = t5 * t6;
    let t8 = fld(rc, 0x2148F0);
    let t9 = t7 + t8;
    fstp(rc, 0x2148F0, t9);
    let t10 = fld(rc, 0x2149F0);
    let t11 = fld(rc, 0x1C5E40);
    let t12 = t10 * t11;
    let t13 = fld(rc, 0x214910);
    let t14 = t12 + t13;
    fstp(rc, 0x214910, t14);
    let t15 = fld(rc, 0x2149F0);
    let t16 = fld(rc, 0x1C5E40);
    let t17 = t15 * t16;
    let t18 = fld(rc, 0x214930);
    let t19 = t17 + t18;
    fstp(rc, 0x214930, t19);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A158] = [rc + 0x31A190] - [rc + 0x31A184]`
/// `[rc + 0x22401C] = |([rc + 0x31A190] - [rc + 0x31A184])| - [rc + 0x213EDC]`
fn x87_23(rc: usize) {
    let t0 = fld(rc, 0x31A190);
    let t1 = fld(rc, 0x31A184);
    let t2 = t0 - t1;
    fstp(rc, 0x31A158, t2);
    let t3 = t2.abs();
    let t4 = fld(rc, 0x213EDC);
    let t5 = t3 - t4;
    fstp(rc, 0x22401C, t5);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x224020] = |[rc + 0x31A17C]| - [rc + 0x213ED8]`
fn x87_24(rc: usize) {
    let t0 = fld(rc, 0x31A17C);
    let t1 = t0.abs();
    let t2 = fld(rc, 0x213ED8);
    let t3 = t1 - t2;
    fstp(rc, 0x224020, t3);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A260] = sqrt((([rc + 0x22401C] * [rc + 0x22401C]) + ([rc + 0x224020] * [rc + 0x224020])))`
/// `[rc + 0x224020] = |[rc + 0x31A17C]|`
/// `[rc + 0x22401C] = |([rc + 0x31A190] - [rc + 0x31A184])|`
/// `[rc + 0x31AF18] = sqrt((([rc + 0x22401C] * [rc + 0x22401C]) + ([rc + 0x224020] * [rc + 0x224020])))`
/// `[rc + 0x22401C] = |([rc + 0x31A194] - [rc + 0x31A184])|`
/// `[rc + 0x31AF1C] = sqrt((([rc + 0x22401C] * [rc + 0x22401C]) + ([rc + 0x224020] * [rc + 0x224020])))`
fn x87_25(rc: usize) {
    let t0 = fld(rc, 0x22401C);
    let t1 = t0 * t0;
    let t2 = fld(rc, 0x224020);
    let t3 = t2 * t2;
    let t4 = t1 + t3;
    let t5 = t4.sqrt();
    fstp(rc, 0x31A260, t5);
    let t6 = fld(rc, 0x31A17C);
    let t7 = t6.abs();
    fstp(rc, 0x224020, t7);
    let t8 = fld(rc, 0x31A190);
    let t9 = fld(rc, 0x31A184);
    let t10 = t8 - t9;
    let t11 = t10.abs();
    fstp(rc, 0x22401C, t11);
    let t12 = fld(rc, 0x22401C);
    let t13 = t12 * t12;
    let t14 = fld(rc, 0x224020);
    let t15 = t14 * t14;
    let t16 = t13 + t15;
    let t17 = t16.sqrt();
    fstp(rc, 0x31AF18, t17);
    let t18 = fld(rc, 0x31A194);
    let t19 = fld(rc, 0x31A184);
    let t20 = t18 - t19;
    let t21 = t20.abs();
    fstp(rc, 0x22401C, t21);
    let t22 = fld(rc, 0x22401C);
    let t23 = t22 * t22;
    let t24 = fld(rc, 0x224020);
    let t25 = t24 * t24;
    let t26 = t23 + t25;
    let t27 = t26.sqrt();
    fstp(rc, 0x31AF1C, t27);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

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
                    Operand::Buffer {
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
                plain: |a| x87_1(a[0], a[1], a[2]),
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
                plain: |a| x87_2(a[0], a[1]),
            },
            Block {
                name: "drive_wheels_step line 73",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
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
                plain: |a| x87_3(a[0], a[1], a[2]),
            },
            Block {
                name: "drive_wheels_step line 87",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[(0, Out::F)],
                    },
                    Operand::Buffer {
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
                plain: |a| x87_4(a[0], a[1], a[2], a[3], a[4]),
            },
            Block {
                name: "drive_wheels_step line 106",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
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
                plain: |a| x87_5(a[0], a[1], a[2]),
            },
            Block {
                name: "drive_wheels_step line 127",
                operands: vec![Operand::Buffer {
                    reads: &[(0x1C5D38, In::F), (0x2140B4, In::F)],
                    writes: &[(0x2140B8, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_5(a[0]) },
                plain: |a| x87_6(a[0]),
            },
            Block {
                name: "drive_wheels_step line 147 idx e=0",
                operands: vec![
                    Operand::Buffer {
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
                plain: |a| x87_7(a[0], a[1]),
            },
            Block {
                name: "drive_wheels_step line 147 idx e=4",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (0x21388C, In::F),
                            (0x214170, In::F),
                            (0x214284, In::F),
                            (0x214288, In::F),
                            (0x21428C, In::F),
                            (0x214290, In::F),
                            (0x21429C, In::F),
                            (0x2142A0, In::F),
                            (0x2142A4, In::F),
                            (0x2142A8, In::F),
                            (0x214774, In::F),
                            (0x214794, In::F),
                            (0x2147B4, In::F),
                            (0x2147D4, In::F),
                            (0x31A3D8, In::F),
                            (0x31A3DC, In::F),
                        ],
                        writes: &[(0x21423C, Out::F), (0x214248, Out::F), (0x21424C, Out::F)],
                    },
                    Operand::Value(4),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_6(a[0], a[1]) },
                plain: |a| x87_7(a[0], a[1]),
            },
            Block {
                name: "drive_wheels_step line 147 idx e=36",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (0x21388C, In::F),
                            (0x214170, In::F),
                            (0x214284, In::F),
                            (0x214288, In::F),
                            (0x21428C, In::F),
                            (0x214290, In::F),
                            (0x2142BC, In::F),
                            (0x2142C0, In::F),
                            (0x2142C4, In::F),
                            (0x2142C8, In::F),
                            (0x214774, In::F),
                            (0x214794, In::F),
                            (0x2147B4, In::F),
                            (0x2147D4, In::F),
                            (0x31A3D8, In::F),
                            (0x31A3DC, In::F),
                        ],
                        writes: &[(0x21423C, Out::F), (0x214248, Out::F), (0x21424C, Out::F)],
                    },
                    Operand::Value(36),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_6(a[0], a[1]) },
                plain: |a| x87_7(a[0], a[1]),
            },
            Block {
                name: "drive_wheels_step line 180",
                operands: vec![Operand::Buffer {
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
                plain: |a| x87_8(a[0]),
            },
            Block {
                name: "drive_wheels_step line 203",
                operands: vec![Operand::Buffer {
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
                plain: |a| x87_9(a[0]),
            },
            Block {
                name: "drive_wheels_step line 220",
                operands: vec![Operand::Buffer {
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
                plain: |a| x87_10(a[0]),
            },
            Block {
                name: "drive_wheels_step line 231",
                operands: vec![Operand::Buffer {
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
                plain: |a| x87_11(a[0]),
            },
            Block {
                name: "drive_wheels_step line 245",
                operands: vec![Operand::Buffer {
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
                plain: |a| x87_12(a[0]),
            },
            Block {
                name: "drive_wheels_step line 258",
                operands: vec![Operand::Buffer {
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
                plain: |a| x87_13(a[0]),
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
                plain: |a| x87_14(a[0], a[1]),
            },
            Block {
                name: "drive_wheels_step line 292",
                operands: vec![Operand::Buffer {
                    reads: &[(0x31A184, In::F), (0x31A190, In::F), (0x31A194, In::F)],
                    writes: &[(0x214234, Out::F), (0x214238, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_14(a[0]) },
                plain: |a| x87_15(a[0]),
            },
            Block {
                name: "drive_wheels_step line 308",
                operands: vec![Operand::Buffer {
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
                plain: |a| x87_16(a[0]),
            },
            Block {
                name: "drive_wheels_step line 331",
                operands: vec![Operand::Buffer {
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
                plain: |a| x87_17(a[0]),
            },
            Block {
                name: "drive_wheels_step line 347",
                operands: vec![Operand::Buffer {
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
                plain: |a| x87_18(a[0]),
            },
            Block {
                name: "drive_wheels_step line 362",
                operands: vec![Operand::Buffer {
                    reads: &[(0x213ED4, In::F), (0x214BFC, In::F)],
                    writes: &[(0x2141D8, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_18(a[0]) },
                plain: |a| x87_19(a[0]),
            },
            Block {
                name: "drive_wheels_step line 372",
                operands: vec![
                    Operand::Buffer {
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
                plain: |a| x87_20(a[0], a[1]),
            },
            Block {
                name: "drive_wheels_step line 385",
                operands: vec![Operand::Buffer {
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
                plain: |a| x87_21(a[0]),
            },
            Block {
                name: "drive_wheels_step line 403",
                operands: vec![Operand::Buffer {
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
                plain: |a| x87_22(a[0]),
            },
            Block {
                name: "drive_wheels_step line 426",
                operands: vec![Operand::Buffer {
                    reads: &[(0x213EDC, In::F), (0x31A184, In::F), (0x31A190, In::F)],
                    writes: &[(0x22401C, Out::F), (0x31A158, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_22(a[0]) },
                plain: |a| x87_23(a[0]),
            },
            Block {
                name: "drive_wheels_step line 436",
                operands: vec![Operand::Buffer {
                    reads: &[(0x213ED8, In::F), (0x31A17C, In::F)],
                    writes: &[(0x224020, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_23(a[0]) },
                plain: |a| x87_24(a[0]),
            },
            Block {
                name: "drive_wheels_step line 444",
                operands: vec![Operand::Buffer {
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
                plain: |a| x87_25(a[0]),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
