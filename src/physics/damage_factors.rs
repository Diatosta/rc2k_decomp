//! `physics::damage_factors` (0x4AB5E6): the damage model's figures for this step
//! (`physics/damage_model.rs`). Three car figures / `flt_75B35C` into `flt_75B3F8` ..;
//! for the four gauges `unk_71A30C` .. `71A318`: a table lerp (`unk_75B4C8`) into
//! `flt_75B474` .., the factor `unk_71A420` + 4k (held 0 ..= `flt_75B34C`) and
//! `unk_71A410` + 4k (both set when the gauge is at the top); their sum x `flt_75B364`
//! into `unk_71A430`; then shares of the engine's and the car's speeds against their
//! limits (`flt_75B398` .. `flt_75B3AC`), the switches `flt_75B3F4`, `flt_75B3B0`
//! (`unk_71A614` at 0x10 or more and moving), `flt_75B3C0` (the engine past its limit x
//! `flt_75B37C`), `flt_75B3B4` (`unk_71A908` at 5), copies into `dword_75B3B8` ..
//! (the gear `dword_75B3BC`), the side hit `dword_75B3C4` (1 .. 4 by `unk_71A1FC`,
//! `unk_71A204`, when `unk_71A208` is set) and its size `flt_75B3C8`, the four wheels'
//! loads (`dword_75B3CC` .., blended by `flt_75B360`), and the hit time `unk_71A438`.
//!
//! In:  nothing
//! Out: eax the side hit's figure, ebx 4, esi the last table; the rest kept; no flags
//!      read (the caller calls on)
//!
//! Caller: `physics::damage_step`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::math::x87::mul;
use crate::math::x87_float::{X, compare};
#[allow(unused_imports)] // the generated body uses some of them
use crate::math::x87_float::{int, ld, st, to_int};
#[allow(unused_imports)]
use crate::math::x87_run::{abs_compare, compare_full};
use crate::physics::car_store::CAR;
use crate::physics::damage_model::ONE;
#[allow(unused_imports)]
use crate::physics::flow::{g, hold_high, hold_low, p, table_lerp};
use crate::physics::store::{DAMAGE, DamageModel};
use crate::session::Cell32;

#[allow(dead_code)]
fn rd(x: u32) -> u32 {
    crate::pointers::read_or_end(x, "a car block's field")
}

/// The steps (generated from the disassembly by idiom; the flagged places by hand).
pub fn steps() {
    let _ = X(1.0);
    x87_1(&DAMAGE);
    table_lerp(
        &CAR.suspension_damage[0],
        &DAMAGE.lerp_tables[2],
        &DAMAGE.gauge_low,
        &DAMAGE.flt_75b474,
    );
    x87_2(&DAMAGE, &DAMAGE.flt_75b474, &CAR.wheel_stiffness[0]);
    hold_low(&CAR.wheel_stiffness[0], &DAMAGE.gauge_low);
    hold_high(&CAR.wheel_stiffness[0], &DAMAGE.gauge_top);
    x87_3(&DAMAGE, &CAR.wheel_ride_height[0]);
    if compare_full(CAR.suspension_damage[0].bits(), DAMAGE.gauge_top.bits()) & 0x4000 != 0 {
        x87_4(&DAMAGE, &CAR.wheel_stiffness[0], &CAR.wheel_ride_height[0]);
    }
    table_lerp(
        &CAR.suspension_damage[1],
        &DAMAGE.lerp_tables[2],
        &DAMAGE.gauge_low,
        &DAMAGE.flt_75b478,
    );
    x87_2(&DAMAGE, &DAMAGE.flt_75b478, &CAR.wheel_stiffness[1]);
    hold_low(&CAR.wheel_stiffness[1], &DAMAGE.gauge_low);
    hold_high(&CAR.wheel_stiffness[1], &DAMAGE.gauge_top);
    x87_3(&DAMAGE, &CAR.wheel_ride_height[1]);
    if compare_full(CAR.suspension_damage[1].bits(), DAMAGE.gauge_top.bits()) & 0x4000 != 0 {
        x87_4(&DAMAGE, &CAR.wheel_stiffness[1], &CAR.wheel_ride_height[1]);
    }
    table_lerp(
        &CAR.suspension_damage[2],
        &DAMAGE.lerp_tables[2],
        &DAMAGE.gauge_low,
        &DAMAGE.flt_75b47c,
    );
    x87_2(&DAMAGE, &DAMAGE.flt_75b47c, &CAR.wheel_stiffness[2]);
    hold_low(&CAR.wheel_stiffness[2], &DAMAGE.gauge_low);
    hold_high(&CAR.wheel_stiffness[2], &DAMAGE.gauge_top);
    x87_3(&DAMAGE, &CAR.wheel_ride_height[2]);
    if compare_full(CAR.suspension_damage[2].bits(), DAMAGE.gauge_top.bits()) & 0x4000 != 0 {
        x87_4(&DAMAGE, &CAR.wheel_stiffness[2], &CAR.wheel_ride_height[2]);
    }
    table_lerp(
        &CAR.suspension_damage[3],
        &DAMAGE.lerp_tables[2],
        &DAMAGE.gauge_low,
        &DAMAGE.flt_75b480,
    );
    x87_2(&DAMAGE, &DAMAGE.flt_75b480, &CAR.wheel_stiffness[3]);
    hold_low(&CAR.wheel_stiffness[3], &DAMAGE.gauge_low);
    hold_high(&CAR.wheel_stiffness[3], &DAMAGE.gauge_top);
    x87_3(&DAMAGE, &CAR.wheel_ride_height[3]);
    if compare_full(CAR.suspension_damage[3].bits(), DAMAGE.gauge_top.bits()) & 0x4000 != 0 {
        x87_4(&DAMAGE, &CAR.wheel_stiffness[3], &CAR.wheel_ride_height[3]);
    }
    x87_5(&DAMAGE);
    hold_high(&DAMAGE.scratch, &CAR.engine_speed_limit);
    x87_6(&DAMAGE);
    hold_high(&DAMAGE.scratch, &DAMAGE.flt_75b378);
    x87_7(&DAMAGE);
    hold_high(&DAMAGE.scratch, &DAMAGE.flt_75b378);
    x87_8(&DAMAGE);
    hold_high(&DAMAGE.scratch, &DAMAGE.flt_75b378);
    x87_9(
        &DAMAGE,
        &DAMAGE.flt_75b3a4,
        &CAR.drive_speed_a,
        &CAR.drive_speed_a,
    );
    hold_high(&DAMAGE.scratch, &DAMAGE.flt_75b378);
    x87_9(
        &DAMAGE,
        &DAMAGE.flt_75b3a8,
        &CAR.drive_speed_b,
        &CAR.drive_speed_b,
    );
    hold_high(&DAMAGE.scratch, &DAMAGE.flt_75b378);
    x87_10(&DAMAGE);
    if CAR.all_wheels_off.get() == 0x0 {
        x87_11(&DAMAGE);
    }
    x87_12(&DAMAGE.flt_75b3b0);
    if abs_compare(CAR.local_velocity[2].bits(), DAMAGE.grade_high.bits()) & 0x100 == 0
        && (CAR.brake.get().wrapping_sub(0x10) as i32) >= 0
    {
        x87_13(&DAMAGE);
    }
    // 0x4ABCAA: 1 when the engine's speed is above the limit times `flt_75B37C`.
    DAMAGE.flt_75b3c0.set_bits(0);
    let status = x87_14(&DAMAGE);
    if status & 0x100 != 0 {
        DAMAGE.flt_75b3c0.set_bits(ONE);
    }
    DAMAGE.dword_75b3b8.set_bits(CAR.gear_changed.get());
    DAMAGE.dword_75b3bc.set_bits(CAR.gear.get());
    DAMAGE.dword_75b3e8.set_bits(CAR.tyre_kind.get());
    let mode = CAR.wheel_surface_a[4].get();
    DAMAGE.dword_75b3ec.set_bits(mode);
    x87_12(&DAMAGE.flt_75b3b4);
    if mode == 5 {
        DAMAGE.flt_75b3b4.set_bits(ONE);
    }
    DAMAGE.dword_75b3f0.set_bits(CAR.surface_blend.bits());
    let mut hit = CAR.contact_depth.bits();
    if hit != 0 {
        hit = 1;
        if compare_full(CAR.contact_point[0].bits(), DAMAGE.gauge_low.bits()) & 0x100 == 0 {
            hit += 1;
        }
        if compare_full(CAR.contact_point[2].bits(), DAMAGE.gauge_low.bits()) & 0x100 != 0 {
            hit += 2;
        }
    }
    DAMAGE.dword_75b3c4.set_bits(hit);
    x87_15(&DAMAGE);
    hold_low(&DAMAGE.scratch, &DAMAGE.gauge_low);
    hold_high(&DAMAGE.scratch, &DAMAGE.flt_75b354);
    x87_16(&DAMAGE);
    for k in 0..4usize {
        DAMAGE.wheel_loads[k].set_bits(0);
        if CAR.wheel_squeeze_last[k].bits() == 0 && CAR.wheel_squeeze[k].bits() != 0 {
            DAMAGE.wheel_loads[k].set_bits(mul(CAR.wheel_squeeze[k].bits(), DAMAGE.scratch.bits()));
        }
    }
    x87_17(&DAMAGE);
    if DAMAGE.flt_75b3c8.bits() != 0x0 && CAR.hit_cooldown.get() != 0x0 {
        DAMAGE.flt_75b3c8.set_bits(0x0);
        CAR.hit_cooldown.set(0x64);
    }
    // retn
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3F8] = int [rc + 0x31A008] / [m + 0x35B35C]`
/// `[m + 0x35B3FC] = int [rc + 0x31A00C] / [m + 0x35B35C]`
/// `[m + 0x35B400] = int [rc + 0x319FF8] / [m + 0x35B35C]`
fn x87_1(m: &DamageModel) {
    let t0 = int(CAR.ride_height.bits() as i32);
    let t1 = ld(m.flt_75b35c.bits());
    let t2 = t0 / t1;
    m.flt_75b3f8.set_bits(st(t2));
    let t3 = int(CAR.suspension_stiffness.bits() as i32);
    let t4 = ld(m.flt_75b35c.bits());
    let t5 = t3 / t4;
    m.flt_75b3fc.set_bits(st(t5));
    let t6 = int(CAR.brake_balance.bits() as i32);
    let t7 = ld(m.flt_75b35c.bits());
    let t8 = t6 / t7;
    m.flt_75b400.set_bits(st(t8));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o3] = ([m + 0x35B3FC] * [m + 0x35B34C]) - [m + o2]`
fn x87_2(m: &DamageModel, o2: &impl Cell32, o3: &impl crate::session::Cell32) {
    let t0 = ld(m.flt_75b3fc.bits());
    let t1 = ld(m.gauge_top.bits());
    let t2 = t0 * t1;
    let t3 = ld(o2.bits());
    let t4 = t2 - t3;
    o3.set_bits(st(t4));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o2] = [m + 0x35B3F8] * [m + 0x35B34C]`
fn x87_3(m: &DamageModel, o2: &impl crate::session::Cell32) {
    let t0 = ld(m.flt_75b3f8.bits());
    let t1 = ld(m.gauge_top.bits());
    let t2 = t0 * t1;
    o2.set_bits(st(t2));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o1] = [m + 0x35B34C]`
/// `[rc + o3] = [m + 0x35B350]`
fn x87_4(m: &DamageModel, o1: &impl crate::session::Cell32, o3: &impl crate::session::Cell32) {
    let t0 = ld(m.gauge_top.bits());
    o1.set_bits(st(t0));
    let t1 = ld(m.gauge_low.bits());
    o3.set_bits(st(t1));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A434] = ((([rc + 0x31A410] + [rc + 0x31A414]) + [rc + 0x31A418]) + [rc + 0x31A41C]) * [m + 0x35B364]`
/// `[rc + 0x31A430] = ((([rc + 0x31A420] + [rc + 0x31A424]) + [rc + 0x31A428]) + [rc + 0x31A42C]) * [m + 0x35B364]`
/// `[m + 0x35B3E4] = [rc + 0x31A018]`
/// `[m + 0x35B36C] = |[rc + 0x31A1B0]|`
fn x87_5(m: &DamageModel) {
    let t0 = ld(CAR.wheel_ride_height[0].bits());
    let t1 = ld(CAR.wheel_ride_height[1].bits());
    let t2 = t0 + t1;
    let t3 = ld(CAR.wheel_ride_height[2].bits());
    let t4 = t2 + t3;
    let t5 = ld(CAR.wheel_ride_height[3].bits());
    let t6 = t4 + t5;
    let t7 = ld(m.flt_75b364.bits());
    let t8 = t6 * t7;
    CAR.ride_height_mean.set_bits(st(t8));
    let t9 = ld(CAR.wheel_stiffness[0].bits());
    let t10 = ld(CAR.wheel_stiffness[1].bits());
    let t11 = t9 + t10;
    let t12 = ld(CAR.wheel_stiffness[2].bits());
    let t13 = t11 + t12;
    let t14 = ld(CAR.wheel_stiffness[3].bits());
    let t15 = t13 + t14;
    let t16 = ld(m.flt_75b364.bits());
    let t17 = t15 * t16;
    CAR.stiffness_mean.set_bits(st(t17));
    let t18 = ld(CAR.drive_kind.bits());
    m.flt_75b3e4.set_bits(st(t18));
    let t19 = ld(CAR.engine_speed.bits());
    let t20 = t19.abs();
    m.scratch.set_bits(st(t20));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B398] = ([m + 0x35B36C] / [rc + 0x31A1B4]) * [m + 0x35B34C]`
/// `[m + 0x35B36C] = |[rc + 0x31A184]|`
fn x87_6(m: &DamageModel) {
    let t0 = ld(m.scratch.bits());
    let t1 = ld(CAR.engine_speed_limit.bits());
    let t2 = t0 / t1;
    let t3 = ld(m.gauge_top.bits());
    let t4 = t2 * t3;
    m.flt_75b398.set_bits(st(t4));
    let t5 = ld(CAR.local_velocity[2].bits());
    let t6 = t5.abs();
    m.scratch.set_bits(st(t6));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B39C] = ([m + 0x35B36C] / [m + 0x35B378]) * [m + 0x35B34C]`
/// `[m + 0x35B3E0] = [rc + 0x31A930]`
/// `[m + 0x35B36C] = |([rc + 0x31A184] - [rc + 0x31A190])|`
fn x87_7(m: &DamageModel) {
    let t0 = ld(m.scratch.bits());
    let t1 = ld(m.flt_75b378.bits());
    let t2 = t0 / t1;
    let t3 = ld(m.gauge_top.bits());
    let t4 = t2 * t3;
    m.flt_75b39c.set_bits(st(t4));
    let t5 = ld(CAR.unknown_948[4].bits());
    m.flt_75b3e0.set_bits(st(t5));
    let t6 = ld(CAR.local_velocity[2].bits());
    let t7 = ld(CAR.drive_speed_a.bits());
    let t8 = t6 - t7;
    let t9 = t8.abs();
    m.scratch.set_bits(st(t9));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3A0] = [m + 0x35B36C] / [m + 0x35B378]`
/// `[m + 0x35B36C] = |([rc + 0x31A184] - [rc + 0x31A194])|`
fn x87_8(m: &DamageModel) {
    let t0 = ld(m.scratch.bits());
    let t1 = ld(m.flt_75b378.bits());
    let t2 = t0 / t1;
    m.flt_75b3a0.set_bits(st(t2));
    let t3 = ld(CAR.local_velocity[2].bits());
    let t4 = ld(CAR.drive_speed_b.bits());
    let t5 = t3 - t4;
    let t6 = t5.abs();
    m.scratch.set_bits(st(t6));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + o2] = [m + 0x35B36C] / [m + 0x35B378]`
/// `[m + 0x35B36C] = |((([rc + o3] - [rc + 0x31A184]) * [m + 0x35B6F4]) + [rc + o6])|`
fn x87_9(
    m: &DamageModel,
    o2: &impl Cell32,
    o3: &impl crate::session::Cell32,
    o6: &impl crate::session::Cell32,
) {
    let t0 = ld(m.scratch.bits());
    let t1 = ld(m.flt_75b378.bits());
    let t2 = t0 / t1;
    o2.set_bits(st(t2));
    let t3 = ld(o3.bits());
    let t4 = ld(CAR.local_velocity[2].bits());
    let t5 = t3 - t4;
    let t6 = ld(m.figures[31].bits());
    let t7 = t5 * t6;
    let t8 = ld(o6.bits());
    let t9 = t7 + t8;
    let t10 = t9.abs();
    m.scratch.set_bits(st(t10));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3AC] = [m + 0x35B36C] / [m + 0x35B378]`
/// `[m + 0x35B3F4] = 0`
fn x87_10(m: &DamageModel) {
    let t0 = ld(m.scratch.bits());
    let t1 = ld(m.flt_75b378.bits());
    let t2 = t0 / t1;
    m.flt_75b3ac.set_bits(st(t2));
    let t3 = X(0.0);
    m.flt_75b3f4.set_bits(st(t3));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3F4] = 1`
fn x87_11(m: &DamageModel) {
    let t0 = X(1.0);
    m.flt_75b3f4.set_bits(st(t0));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + o0] = 0`
fn x87_12(o0: &impl Cell32) {
    let t0 = X(0.0);
    o0.set_bits(st(t0));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3B0] = int [rc + 0x31A614] / [m + 0x35B35C]`
fn x87_13(m: &DamageModel) {
    let t0 = int(CAR.brake.bits() as i32);
    let t1 = ld(m.flt_75b35c.bits());
    let t2 = t0 / t1;
    m.flt_75b3b0.set_bits(st(t2));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `status [s + 0] = status`
fn x87_14(m: &DamageModel) -> u16 {
    let t0 = ld(CAR.engine_speed_limit.bits());
    let t1 = ld(m.flt_75b37c.bits());
    let t2 = t0 * t1;
    let t3 = ld(CAR.engine_speed.bits());
    compare(t2, t3)
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3C8] = (([rc + 0x31A208] * [rc + 0x31A208]) / [m + 0x35B394]) / [m + 0x35B378]`
/// `[m + 0x35B36C] = -[rc + 0x31A180] - [m + 0x35B624]`
fn x87_15(m: &DamageModel) {
    let t0 = ld(CAR.contact_depth.bits());
    let t1 = ld(CAR.contact_depth.bits());
    let t2 = t0 * t1;
    let t3 = ld(m.flt_75b394.bits());
    let t4 = t2 / t3;
    let t5 = ld(m.flt_75b378.bits());
    let t6 = t4 / t5;
    m.flt_75b3c8.set_bits(st(t6));
    let t7 = ld(CAR.local_velocity[1].bits());
    let t8 = -t7;
    let t9 = ld(m.gauge_rates[21].bits());
    let t10 = t8 - t9;
    m.scratch.set_bits(st(t10));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B36C] = ([m + 0x35B36C] * [m + 0x35B36C]) * [m + 0x35B620]`
fn x87_16(m: &DamageModel) {
    let t0 = ld(m.scratch.bits());
    let t1 = t0 * t0;
    let t2 = ld(m.gauge_rates[20].bits());
    let t3 = t1 * t2;
    m.scratch.set_bits(st(t3));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3DC] = ((([m + 0x35B3CC] + [m + 0x35B3D0]) + [m + 0x35B3D4]) + [m + 0x35B3D8]) * [m + 0x35B364]`
/// `[m + 0x35B36C] = [m + 0x35B3DC] * [m + 0x35B360]`
/// `[m + 0x35B3CC] = ([m + 0x35B3CC] * [m + 0x35B360]) + [m + 0x35B36C]`
/// `[m + 0x35B3D0] = ([m + 0x35B3D0] * [m + 0x35B360]) + [m + 0x35B36C]`
/// `[m + 0x35B3D4] = ([m + 0x35B3D4] * [m + 0x35B360]) + [m + 0x35B36C]`
/// `[m + 0x35B3D8] = ([m + 0x35B3D8] * [m + 0x35B360]) + [m + 0x35B36C]`
fn x87_17(m: &DamageModel) {
    let t0 = ld(m.wheel_loads[0].bits());
    let t1 = ld(m.wheel_loads[1].bits());
    let t2 = t0 + t1;
    let t3 = ld(m.wheel_loads[2].bits());
    let t4 = t2 + t3;
    let t5 = ld(m.wheel_loads[3].bits());
    let t6 = t4 + t5;
    let t7 = ld(m.flt_75b364.bits());
    let t8 = t6 * t7;
    m.flt_75b3dc.set_bits(st(t8));
    let t9 = ld(m.flt_75b3dc.bits());
    let t10 = ld(m.flt_75b360.bits());
    let t11 = t9 * t10;
    m.scratch.set_bits(st(t11));
    let t12 = ld(m.wheel_loads[0].bits());
    let t13 = ld(m.flt_75b360.bits());
    let t14 = t12 * t13;
    let t15 = ld(m.scratch.bits());
    let t16 = t14 + t15;
    m.wheel_loads[0].set_bits(st(t16));
    let t17 = ld(m.wheel_loads[1].bits());
    let t18 = ld(m.flt_75b360.bits());
    let t19 = t17 * t18;
    let t20 = ld(m.scratch.bits());
    let t21 = t19 + t20;
    m.wheel_loads[1].set_bits(st(t21));
    let t22 = ld(m.wheel_loads[2].bits());
    let t23 = ld(m.flt_75b360.bits());
    let t24 = t22 * t23;
    let t25 = ld(m.scratch.bits());
    let t26 = t24 + t25;
    m.wheel_loads[2].set_bits(st(t26));
    let t27 = ld(m.wheel_loads[3].bits());
    let t28 = ld(m.flt_75b360.bits());
    let t29 = t27 * t28;
    let t30 = ld(m.scratch.bits());
    let t31 = t29 + t30;
    m.wheel_loads[3].set_bits(st(t31));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;

    /// The damage model as the test's buffer holds it (the buffer's base plus the RVA of
    /// 0x75B340).
    fn dm(base: usize) -> &'static DamageModel {
        unsafe { &*(crate::pointers::host_at(base + 0x35B340) as *const DamageModel) }
    }
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};
    use crate::scene::x87_mem::Base;

    // The original's instructions, one function per run.
    reference!(ref_0(rc, m) "fild dword ptr [{rc} + 0x31A008]", "fdiv dword ptr [{m} + 0x35B35C]", "fstp dword ptr [{m} + 0x35B3F8]", "fild dword ptr [{rc} + 0x31A00C]", "fdiv dword ptr [{m} + 0x35B35C]", "fstp dword ptr [{m} + 0x35B3FC]", "fild dword ptr [{rc} + 0x319FF8]", "fdiv dword ptr [{m} + 0x35B35C]", "fstp dword ptr [{m} + 0x35B400]");
    reference!(ref_1(m, rc) "fld dword ptr [{m} + 0x35B3FC]", "fmul dword ptr [{m} + 0x35B34C]", "fsub dword ptr [{m} + 0x35B474]", "fstp dword ptr [{rc} + 0x31A420]");
    reference!(ref_2(m, rc) "fld dword ptr [{m} + 0x35B3F8]", "fmul dword ptr [{m} + 0x35B34C]", "fstp dword ptr [{rc} + 0x31A410]");
    reference!(ref_3(m, rc) "fld dword ptr [{m} + 0x35B34C]", "fstp dword ptr [{rc} + 0x31A420]", "fld dword ptr [{m} + 0x35B350]", "fstp dword ptr [{rc} + 0x31A410]");
    reference!(ref_4(m, rc) "fld dword ptr [{m} + 0x35B3FC]", "fmul dword ptr [{m} + 0x35B34C]", "fsub dword ptr [{m} + 0x35B478]", "fstp dword ptr [{rc} + 0x31A424]");
    reference!(ref_5(m, rc) "fld dword ptr [{m} + 0x35B3F8]", "fmul dword ptr [{m} + 0x35B34C]", "fstp dword ptr [{rc} + 0x31A414]");
    reference!(ref_6(m, rc) "fld dword ptr [{m} + 0x35B34C]", "fstp dword ptr [{rc} + 0x31A424]", "fld dword ptr [{m} + 0x35B350]", "fstp dword ptr [{rc} + 0x31A414]");
    reference!(ref_7(m, rc) "fld dword ptr [{m} + 0x35B3FC]", "fmul dword ptr [{m} + 0x35B34C]", "fsub dword ptr [{m} + 0x35B47C]", "fstp dword ptr [{rc} + 0x31A428]");
    reference!(ref_8(m, rc) "fld dword ptr [{m} + 0x35B3F8]", "fmul dword ptr [{m} + 0x35B34C]", "fstp dword ptr [{rc} + 0x31A418]");
    reference!(ref_9(m, rc) "fld dword ptr [{m} + 0x35B34C]", "fstp dword ptr [{rc} + 0x31A428]", "fld dword ptr [{m} + 0x35B350]", "fstp dword ptr [{rc} + 0x31A418]");
    reference!(ref_10(m, rc) "fld dword ptr [{m} + 0x35B3FC]", "fmul dword ptr [{m} + 0x35B34C]", "fsub dword ptr [{m} + 0x35B480]", "fstp dword ptr [{rc} + 0x31A42C]");
    reference!(ref_11(m, rc) "fld dword ptr [{m} + 0x35B3F8]", "fmul dword ptr [{m} + 0x35B34C]", "fstp dword ptr [{rc} + 0x31A41C]");
    reference!(ref_12(m, rc) "fld dword ptr [{m} + 0x35B34C]", "fstp dword ptr [{rc} + 0x31A42C]", "fld dword ptr [{m} + 0x35B350]", "fstp dword ptr [{rc} + 0x31A41C]");
    reference!(ref_13(rc, m) "fld dword ptr [{rc} + 0x31A410]", "fadd dword ptr [{rc} + 0x31A414]", "fadd dword ptr [{rc} + 0x31A418]", "fadd dword ptr [{rc} + 0x31A41C]", "fmul dword ptr [{m} + 0x35B364]", "fstp dword ptr [{rc} + 0x31A434]", "fld dword ptr [{rc} + 0x31A420]", "fadd dword ptr [{rc} + 0x31A424]", "fadd dword ptr [{rc} + 0x31A428]", "fadd dword ptr [{rc} + 0x31A42C]", "fmul dword ptr [{m} + 0x35B364]", "fstp dword ptr [{rc} + 0x31A430]", "fld dword ptr [{rc} + 0x31A018]", "fstp dword ptr [{m} + 0x35B3E4]", "fld dword ptr [{rc} + 0x31A1B0]", ".byte 0xD9, 0xE1", "fstp dword ptr [{m} + 0x35B36C]");
    reference!(ref_14(m, rc) "fld dword ptr [{m} + 0x35B36C]", "fdiv dword ptr [{rc} + 0x31A1B4]", "fmul dword ptr [{m} + 0x35B34C]", "fstp dword ptr [{m} + 0x35B398]", "fld dword ptr [{rc} + 0x31A184]", ".byte 0xD9, 0xE1", "fstp dword ptr [{m} + 0x35B36C]");
    reference!(ref_15(m, rc) "fld dword ptr [{m} + 0x35B36C]", "fdiv dword ptr [{m} + 0x35B378]", "fmul dword ptr [{m} + 0x35B34C]", "fstp dword ptr [{m} + 0x35B39C]", "fld dword ptr [{rc} + 0x31A930]", "fstp dword ptr [{m} + 0x35B3E0]", "fld dword ptr [{rc} + 0x31A184]", "fsub dword ptr [{rc} + 0x31A190]", ".byte 0xD9, 0xE1", "fstp dword ptr [{m} + 0x35B36C]");
    reference!(ref_16(m, rc) "fld dword ptr [{m} + 0x35B36C]", "fdiv dword ptr [{m} + 0x35B378]", "fstp dword ptr [{m} + 0x35B3A0]", "fld dword ptr [{rc} + 0x31A184]", "fsub dword ptr [{rc} + 0x31A194]", ".byte 0xD9, 0xE1", "fstp dword ptr [{m} + 0x35B36C]");
    reference!(ref_17(m, rc) "fld dword ptr [{m} + 0x35B36C]", "fdiv dword ptr [{m} + 0x35B378]", "fstp dword ptr [{m} + 0x35B3A4]", "fld dword ptr [{rc} + 0x31A190]", "fsub dword ptr [{rc} + 0x31A184]", "fmul dword ptr [{m} + 0x35B6F4]", "fadd dword ptr [{rc} + 0x31A190]", ".byte 0xD9, 0xE1", "fstp dword ptr [{m} + 0x35B36C]");
    reference!(ref_18(m, rc) "fld dword ptr [{m} + 0x35B36C]", "fdiv dword ptr [{m} + 0x35B378]", "fstp dword ptr [{m} + 0x35B3A8]", "fld dword ptr [{rc} + 0x31A194]", "fsub dword ptr [{rc} + 0x31A184]", "fmul dword ptr [{m} + 0x35B6F4]", "fadd dword ptr [{rc} + 0x31A194]", ".byte 0xD9, 0xE1", "fstp dword ptr [{m} + 0x35B36C]");
    reference!(ref_19(m) "fld dword ptr [{m} + 0x35B36C]", "fdiv dword ptr [{m} + 0x35B378]", "fstp dword ptr [{m} + 0x35B3AC]", ".byte 0xD9, 0xEE", "fstp dword ptr [{m} + 0x35B3F4]");
    reference!(ref_20(m) ".byte 0xD9, 0xE8", "fstp dword ptr [{m} + 0x35B3F4]");
    reference!(ref_21(m) ".byte 0xD9, 0xEE", "fstp dword ptr [{m} + 0x35B3B0]");
    reference!(ref_22(rc, m) "fild dword ptr [{rc} + 0x31A614]", "fdiv dword ptr [{m} + 0x35B35C]", "fstp dword ptr [{m} + 0x35B3B0]");
    reference!(ref_23(rc, m, s) "fld dword ptr [{rc} + 0x31A1B4]", "fmul dword ptr [{m} + 0x35B37C]", "fcomp dword ptr [{rc} + 0x31A1B0]", "fnstsw word ptr [{s}]");
    reference!(ref_24(m) ".byte 0xD9, 0xEE", "fstp dword ptr [{m} + 0x35B3B4]");
    reference!(ref_25(rc, m) "fld dword ptr [{rc} + 0x31A208]", "fmul dword ptr [{rc} + 0x31A208]", "fdiv dword ptr [{m} + 0x35B394]", "fdiv dword ptr [{m} + 0x35B378]", "fstp dword ptr [{m} + 0x35B3C8]", "fld dword ptr [{rc} + 0x31A180]", ".byte 0xD9, 0xE0", "fsub dword ptr [{m} + 0x35B624]", "fstp dword ptr [{m} + 0x35B36C]");
    reference!(ref_26(m) "fld dword ptr [{m} + 0x35B36C]", ".byte 0xD8, 0xC8", "fmul dword ptr [{m} + 0x35B620]", "fstp dword ptr [{m} + 0x35B36C]");
    reference!(ref_27(m) "fld dword ptr [{m} + 0x35B3CC]", "fadd dword ptr [{m} + 0x35B3D0]", "fadd dword ptr [{m} + 0x35B3D4]", "fadd dword ptr [{m} + 0x35B3D8]", "fmul dword ptr [{m} + 0x35B364]", "fstp dword ptr [{m} + 0x35B3DC]", "fld dword ptr [{m} + 0x35B3DC]", "fmul dword ptr [{m} + 0x35B360]", "fstp dword ptr [{m} + 0x35B36C]", "fld dword ptr [{m} + 0x35B3CC]", "fmul dword ptr [{m} + 0x35B360]", "fadd dword ptr [{m} + 0x35B36C]", "fstp dword ptr [{m} + 0x35B3CC]", "fld dword ptr [{m} + 0x35B3D0]", "fmul dword ptr [{m} + 0x35B360]", "fadd dword ptr [{m} + 0x35B36C]", "fstp dword ptr [{m} + 0x35B3D0]", "fld dword ptr [{m} + 0x35B3D4]", "fmul dword ptr [{m} + 0x35B360]", "fadd dword ptr [{m} + 0x35B36C]", "fstp dword ptr [{m} + 0x35B3D4]", "fld dword ptr [{m} + 0x35B3D8]", "fmul dword ptr [{m} + 0x35B360]", "fadd dword ptr [{m} + 0x35B36C]", "fstp dword ptr [{m} + 0x35B3D8]");

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
                        reads: &[(0x35B35C, In::F)],
                        writes: &[(0x35B3F8, Out::F), (0x35B3FC, Out::F), (0x35B400, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1]) },
                plain: |a| x87_1(dm(a[1])),
            },
            Block {
                name: "damage_factors line 53",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B3FC, In::F), (0x35B474, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A420, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1]) },
                plain: |a| x87_2(dm(a[0]), &dm(a[0]).flt_75b474, &CAR.wheel_stiffness[0]),
            },
            Block {
                name: "damage_factors line 63",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B3F8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A410, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0], a[1]) },
                plain: |a| x87_3(dm(a[0]), &CAR.wheel_ride_height[0]),
            },
            Block {
                name: "damage_factors line 71",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B350, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A410, Out::F), (0x31A420, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_3(a[0], a[1]) },
                plain: |a| x87_4(dm(a[0]), &CAR.wheel_stiffness[0], &CAR.wheel_ride_height[0]),
            },
            Block {
                name: "damage_factors line 81",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B3FC, In::F), (0x35B478, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A424, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_4(a[0], a[1]) },
                plain: |a| x87_2(dm(a[0]), &dm(a[0]).flt_75b478, &CAR.wheel_stiffness[1]),
            },
            Block {
                name: "damage_factors line 91",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B3F8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A414, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_5(a[0], a[1]) },
                plain: |a| x87_3(dm(a[0]), &CAR.wheel_ride_height[1]),
            },
            Block {
                name: "damage_factors line 99",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B350, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A414, Out::F), (0x31A424, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_6(a[0], a[1]) },
                plain: |a| x87_4(dm(a[0]), &CAR.wheel_stiffness[1], &CAR.wheel_ride_height[1]),
            },
            Block {
                name: "damage_factors line 109",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B3FC, In::F), (0x35B47C, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A428, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_7(a[0], a[1]) },
                plain: |a| x87_2(dm(a[0]), &dm(a[0]).flt_75b47c, &CAR.wheel_stiffness[2]),
            },
            Block {
                name: "damage_factors line 119",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B3F8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A418, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_8(a[0], a[1]) },
                plain: |a| x87_3(dm(a[0]), &CAR.wheel_ride_height[2]),
            },
            Block {
                name: "damage_factors line 127",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B350, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A418, Out::F), (0x31A428, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_9(a[0], a[1]) },
                plain: |a| x87_4(dm(a[0]), &CAR.wheel_stiffness[2], &CAR.wheel_ride_height[2]),
            },
            Block {
                name: "damage_factors line 137",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B3FC, In::F), (0x35B480, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A42C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_10(a[0], a[1]) },
                plain: |a| x87_2(dm(a[0]), &dm(a[0]).flt_75b480, &CAR.wheel_stiffness[3]),
            },
            Block {
                name: "damage_factors line 147",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B3F8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A41C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_11(a[0], a[1]) },
                plain: |a| x87_3(dm(a[0]), &CAR.wheel_ride_height[3]),
            },
            Block {
                name: "damage_factors line 155",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B350, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x31A41C, Out::F), (0x31A42C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_12(a[0], a[1]) },
                plain: |a| x87_4(dm(a[0]), &CAR.wheel_stiffness[3], &CAR.wheel_ride_height[3]),
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
                        reads: &[(0x35B364, In::F)],
                        writes: &[(0x35B36C, Out::F), (0x35B3E4, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_13(a[0], a[1]) },
                plain: |a| x87_5(dm(a[1])),
            },
            Block {
                name: "damage_factors line 186",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B36C, In::F)],
                        writes: &[(0x35B36C, Out::F), (0x35B398, Out::F)],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A184, In::F), (0x31A1B4, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_14(a[0], a[1]) },
                plain: |a| x87_6(dm(a[0])),
            },
            Block {
                name: "damage_factors line 198",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B34C, In::F), (0x35B36C, In::F), (0x35B378, In::F)],
                        writes: &[(0x35B36C, Out::F), (0x35B39C, Out::F), (0x35B3E0, Out::F)],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A184, In::F), (0x31A190, In::F), (0x31A930, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_15(a[0], a[1]) },
                plain: |a| x87_7(dm(a[0])),
            },
            Block {
                name: "damage_factors line 213",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B36C, In::F), (0x35B378, In::F)],
                        writes: &[(0x35B36C, Out::F), (0x35B3A0, Out::F)],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A184, In::F), (0x31A194, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_16(a[0], a[1]) },
                plain: |a| x87_8(dm(a[0])),
            },
            Block {
                name: "damage_factors line 225",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B36C, In::F), (0x35B378, In::F), (0x35B6F4, In::F)],
                        writes: &[(0x35B36C, Out::F), (0x35B3A4, Out::F)],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A184, In::F), (0x31A190, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_17(a[0], a[1]) },
                plain: |a| {
                    x87_9(
                        dm(a[0]),
                        &dm(a[0]).flt_75b3a4,
                        &CAR.drive_speed_a,
                        &CAR.drive_speed_a,
                    );
                },
            },
            Block {
                name: "damage_factors line 239",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B36C, In::F), (0x35B378, In::F), (0x35B6F4, In::F)],
                        writes: &[(0x35B36C, Out::F), (0x35B3A8, Out::F)],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A184, In::F), (0x31A194, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_18(a[0], a[1]) },
                plain: |a| {
                    x87_9(
                        dm(a[0]),
                        &dm(a[0]).flt_75b3a8,
                        &CAR.drive_speed_b,
                        &CAR.drive_speed_b,
                    );
                },
            },
            Block {
                name: "damage_factors line 253",
                operands: vec![Operand::Buffer {
                    reads: &[(0x35B36C, In::F), (0x35B378, In::F)],
                    writes: &[(0x35B3AC, Out::F), (0x35B3F4, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_19(a[0]) },
                plain: |a| x87_10(dm(a[0])),
            },
            Block {
                name: "damage_factors line 262",
                operands: vec![Operand::Buffer {
                    reads: &[],
                    writes: &[(0x35B3F4, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_20(a[0]) },
                plain: |a| x87_11(dm(a[0])),
            },
            Block {
                name: "damage_factors line 268",
                operands: vec![Operand::Buffer {
                    reads: &[],
                    writes: &[(0x35B3B0, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_21(a[0]) },
                plain: |a| x87_12(&dm(a[0]).flt_75b3b0),
            },
            Block {
                name: "damage_factors line 276",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[(0x31A614, In::I)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x35B35C, In::F)],
                        writes: &[(0x35B3B0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_22(a[0], a[1]) },
                plain: |a| x87_13(dm(a[1])),
            },
            Block {
                name: "damage_factors line 287",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[(0x31A1B0, In::F), (0x31A1B4, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x35B37C, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::H)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_23(a[0], a[1], a[2]) },
                plain: |a| a[2].set_word(0, x87_14(dm(a[1]))),
            },
            Block {
                name: "damage_factors line 302",
                operands: vec![Operand::Buffer {
                    reads: &[],
                    writes: &[(0x35B3B4, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_24(a[0]) },
                plain: |a| x87_12(&dm(a[0]).flt_75b3b4),
            },
            Block {
                name: "damage_factors line 322",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[(0x31A180, In::F), (0x31A208, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x35B378, In::F), (0x35B394, In::F), (0x35B624, In::F)],
                        writes: &[(0x35B36C, Out::F), (0x35B3C8, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_25(a[0], a[1]) },
                plain: |a| x87_15(dm(a[1])),
            },
            Block {
                name: "damage_factors line 337",
                operands: vec![Operand::Buffer {
                    reads: &[(0x35B36C, In::F), (0x35B620, In::F)],
                    writes: &[(0x35B36C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_26(a[0]) },
                plain: |a| x87_16(dm(a[0])),
            },
            Block {
                name: "damage_factors line 350",
                operands: vec![Operand::Buffer {
                    reads: &[
                        (0x35B360, In::F),
                        (0x35B364, In::F),
                        (0x35B36C, In::F),
                        (0x35B3CC, In::F),
                        (0x35B3D0, In::F),
                        (0x35B3D4, In::F),
                        (0x35B3D8, In::F),
                        (0x35B3DC, In::F),
                    ],
                    writes: &[
                        (0x35B36C, Out::F),
                        (0x35B3CC, Out::F),
                        (0x35B3D0, Out::F),
                        (0x35B3D4, Out::F),
                        (0x35B3D8, Out::F),
                        (0x35B3DC, Out::F),
                    ],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_27(a[0]) },
                plain: |a| x87_17(dm(a[0])),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
