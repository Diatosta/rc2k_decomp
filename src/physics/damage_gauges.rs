//! `physics::damage_gauges` (0x4AC817): the damage gauges' step (`physics/damage_model.rs`):
//! each gauge (`unk_71A284` .. `unk_71A33C`, the gearbox's eight `unk_71A2AC[gear]` by
//! `dword_75B3BC`) raised by its rate (`flt_75B5D0` ..) times this step's shares and hit
//! figures (`flt_75B38C`, `flt_75B370`, `flt_75B398`, `flt_75B3B4`, `flt_75B450` ..), each
//! held 0 ..= `flt_75B34C` after (`fcompp`, below or unordered raised); the hit's side
//! (`dword_75B3C4` 1 .. 4) choosing gauges, its size (`flt_75B3C8`) scaled by
//! `flt_71C708` past `flt_75B628`, the direction `dword_75B3B8` (1 / -1) adding fixed
//! steps; the kind tables `flt_75B630` / `flt_75B638` by `flt_75B3E4`'s bits.
//!
//! In:  nothing
//! Out: esi and ebx the gear (where the original loads them), the rest as the steps leave
//!      them; no flags read
//!
//! Caller: `physics::damage_step`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::math::x87::loaded;
use crate::math::x87_float::X;
#[allow(unused_imports)] // the generated body uses some of them
use crate::math::x87_float::{int, ld, st, to_int};
#[allow(unused_imports)]
use crate::math::x87_run::{abs_compare, compare_full};
use crate::physics::car_store::CAR;
#[allow(unused_imports)]
use crate::physics::flow::{a, c0, g, hold_high, hold_low, p, table_lerp};
use crate::physics::store::{DAMAGE, DamageModel};
use crate::race::work_store::RACE_WORK;
use crate::scene::x87_mem::{fld, fstp};
use crate::session::Cell32;

/// `unk_71A2AC[gear]` (the gear -1 .. 7 as the original indexes it).
fn gear_gauge(gear: u32) -> u32 {
    (CAR.gear_wear[1].addr()).wrapping_add(gear.wrapping_mul(4))
}

/// The gauge at `at` held 0 ..= `flt_75B34C` (the two `fcompp` holds).
fn hold_at(at: u32) {
    let rd = || crate::pointers::read_or_end::<u32>(at, "a damage gauge");
    let put = |v: u32| crate::pointers::write_or_end(at, v, "a damage gauge");
    if c0(rd(), DAMAGE.gauge_low.bits()) {
        put(loaded(DAMAGE.gauge_low.bits()));
    }
    if !c0(rd(), DAMAGE.gauge_top.bits()) {
        put(loaded(DAMAGE.gauge_top.bits()));
    }
}

/// The steps (generated from the disassembly by idiom; the indexed gauges by hand).
pub fn steps() {
    x87_1(
        &DAMAGE.gauge_rates[0],
        &DAMAGE.flt_75b38c,
        &CAR.engine_damage,
        &CAR.engine_damage,
    );
    hold_low(&CAR.engine_damage, &DAMAGE.gauge_low);
    hold_high(&CAR.engine_damage, &DAMAGE.gauge_top);
    x87_1(
        &DAMAGE.gauge_rates[1],
        &DAMAGE.flt_75b38c,
        &CAR.exhaust_damage,
        &CAR.exhaust_damage,
    );
    hold_low(&CAR.exhaust_damage, &DAMAGE.gauge_low);
    hold_high(&CAR.exhaust_damage, &DAMAGE.gauge_top);
    x87_1(
        &DAMAGE.gauge_rates[2],
        &DAMAGE.flt_75b38c,
        &CAR.cooling_damage,
        &CAR.cooling_damage,
    );
    hold_low(&CAR.cooling_damage, &DAMAGE.gauge_low);
    hold_high(&CAR.cooling_damage, &DAMAGE.gauge_top);
    x87_1(
        &DAMAGE.gauge_rates[3],
        &DAMAGE.flt_75b38c,
        &CAR.clutch_damage,
        &CAR.clutch_damage,
    );
    hold_low(&CAR.clutch_damage, &DAMAGE.gauge_low);
    hold_high(&CAR.clutch_damage, &DAMAGE.gauge_top);
    x87_1(
        &DAMAGE.gauge_rates[4],
        &DAMAGE.flt_75b38c,
        &CAR.differential_damage,
        &CAR.differential_damage,
    );
    hold_low(&CAR.differential_damage, &DAMAGE.gauge_low);
    hold_high(&CAR.differential_damage, &DAMAGE.gauge_top);
    x87_1(
        &DAMAGE.gauge_rates[5],
        &DAMAGE.flt_75b38c,
        &CAR.electrics_damage,
        &CAR.electrics_damage,
    );
    hold_low(&CAR.electrics_damage, &DAMAGE.gauge_low);
    hold_high(&CAR.electrics_damage, &DAMAGE.gauge_top);
    x87_1(
        &DAMAGE.gauge_rates[6],
        &DAMAGE.flt_75b38c,
        &CAR.light_damage[0],
        &CAR.light_damage[0],
    );
    hold_low(&CAR.light_damage[0], &DAMAGE.gauge_low);
    hold_high(&CAR.light_damage[0], &DAMAGE.gauge_top);
    x87_1(
        &DAMAGE.gauge_rates[6],
        &DAMAGE.flt_75b38c,
        &CAR.light_damage[1],
        &CAR.light_damage[1],
    );
    hold_low(&CAR.light_damage[1], &DAMAGE.gauge_low);
    hold_high(&CAR.light_damage[1], &DAMAGE.gauge_top);
    x87_1(
        &DAMAGE.gauge_rates[6],
        &DAMAGE.flt_75b38c,
        &CAR.light_damage[2],
        &CAR.light_damage[2],
    );
    hold_low(&CAR.light_damage[2], &DAMAGE.gauge_low);
    hold_high(&CAR.light_damage[2], &DAMAGE.gauge_top);
    x87_1(
        &DAMAGE.gauge_rates[6],
        &DAMAGE.flt_75b38c,
        &CAR.light_damage[3],
        &CAR.light_damage[3],
    );
    hold_low(&CAR.light_damage[3], &DAMAGE.gauge_low);
    hold_high(&CAR.light_damage[3], &DAMAGE.gauge_top);
    x87_1(
        &DAMAGE.gauge_rates[6],
        &DAMAGE.flt_75b38c,
        &CAR.light_damage[4],
        &CAR.light_damage[4],
    );
    hold_low(&CAR.light_damage[4], &DAMAGE.gauge_low);
    hold_high(&CAR.light_damage[4], &DAMAGE.gauge_top);
    x87_1(
        &DAMAGE.gauge_rates[6],
        &DAMAGE.flt_75b38c,
        &CAR.light_damage[5],
        &CAR.light_damage[5],
    );
    hold_low(&CAR.light_damage[5], &DAMAGE.gauge_low);
    hold_high(&CAR.light_damage[5], &DAMAGE.gauge_top);
    x87_1(
        &DAMAGE.gauge_rates[7],
        &DAMAGE.flt_75b38c,
        &CAR.steering_damage,
        &CAR.steering_damage,
    );
    hold_low(&CAR.steering_damage, &DAMAGE.gauge_low);
    hold_high(&CAR.steering_damage, &DAMAGE.gauge_top);
    x87_1(
        &DAMAGE.gauge_rates[8],
        &DAMAGE.flt_75b38c,
        &CAR.suspension_damage[0],
        &CAR.suspension_damage[0],
    );
    hold_low(&CAR.suspension_damage[0], &DAMAGE.gauge_low);
    hold_high(&CAR.suspension_damage[0], &DAMAGE.gauge_top);
    x87_1(
        &DAMAGE.gauge_rates[8],
        &DAMAGE.flt_75b38c,
        &CAR.suspension_damage[1],
        &CAR.suspension_damage[1],
    );
    hold_low(&CAR.suspension_damage[1], &DAMAGE.gauge_low);
    hold_high(&CAR.suspension_damage[1], &DAMAGE.gauge_top);
    x87_1(
        &DAMAGE.gauge_rates[8],
        &DAMAGE.flt_75b38c,
        &CAR.suspension_damage[2],
        &CAR.suspension_damage[2],
    );
    hold_low(&CAR.suspension_damage[2], &DAMAGE.gauge_low);
    hold_high(&CAR.suspension_damage[2], &DAMAGE.gauge_top);
    x87_1(
        &DAMAGE.gauge_rates[8],
        &DAMAGE.flt_75b38c,
        &CAR.suspension_damage[3],
        &CAR.suspension_damage[3],
    );
    hold_low(&CAR.suspension_damage[3], &DAMAGE.gauge_low);
    hold_high(&CAR.suspension_damage[3], &DAMAGE.gauge_top);
    x87_2(
        &DAMAGE,
        &DAMAGE.wear_rates[11],
        &DAMAGE.flt_75b3c0,
        &DAMAGE.flt_75b398,
        &CAR.turbo_damage,
        &CAR.turbo_damage,
    );
    hold_low(&CAR.turbo_damage, &DAMAGE.gauge_low);
    hold_high(&CAR.turbo_damage, &DAMAGE.gauge_top);
    let gear = DAMAGE.dword_75b3bc.bits();
    let gauge = gear_gauge(gear);
    x87_3(
        &DAMAGE,
        ((DAMAGE.flt_75b5b0.addr()).wrapping_add(gear.wrapping_mul(4))) as usize,
        gauge as usize,
    );
    hold_at(gauge);
    x87_4(&DAMAGE);
    hold_low(&CAR.brake_wear_a, &DAMAGE.gauge_low);
    hold_high(&CAR.brake_wear_a, &DAMAGE.gauge_top);
    x87_2(
        &DAMAGE,
        &DAMAGE.gauge_rates[9],
        &DAMAGE.flt_75b43c,
        &DAMAGE.flt_75b3b0,
        &CAR.brake_wear_b,
        &CAR.brake_wear_b,
    );
    hold_low(&CAR.brake_wear_b, &DAMAGE.gauge_low);
    hold_high(&CAR.brake_wear_b, &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.gauge_rates[10],
        &DAMAGE.flt_75b3b4,
        &DAMAGE.flt_75b38c,
        &CAR.electrics_damage,
        &CAR.electrics_damage,
    );
    hold_low(&CAR.electrics_damage, &DAMAGE.gauge_low);
    hold_high(&CAR.electrics_damage, &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.gauge_rates[11],
        &DAMAGE.flt_75b3b4,
        &DAMAGE.flt_75b38c,
        &CAR.light_damage[0],
        &CAR.light_damage[0],
    );
    hold_low(&CAR.light_damage[0], &DAMAGE.gauge_low);
    hold_high(&CAR.light_damage[0], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.gauge_rates[11],
        &DAMAGE.flt_75b3b4,
        &DAMAGE.flt_75b38c,
        &CAR.light_damage[1],
        &CAR.light_damage[1],
    );
    hold_low(&CAR.light_damage[1], &DAMAGE.gauge_low);
    hold_high(&CAR.light_damage[1], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.gauge_rates[11],
        &DAMAGE.flt_75b3b4,
        &DAMAGE.flt_75b38c,
        &CAR.light_damage[2],
        &CAR.light_damage[2],
    );
    hold_low(&CAR.light_damage[2], &DAMAGE.gauge_low);
    hold_high(&CAR.light_damage[2], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.gauge_rates[11],
        &DAMAGE.flt_75b3b4,
        &DAMAGE.flt_75b38c,
        &CAR.light_damage[3],
        &CAR.light_damage[3],
    );
    hold_low(&CAR.light_damage[3], &DAMAGE.gauge_low);
    hold_high(&CAR.light_damage[3], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.gauge_rates[11],
        &DAMAGE.flt_75b3b4,
        &DAMAGE.flt_75b38c,
        &CAR.light_damage[4],
        &CAR.light_damage[4],
    );
    hold_low(&CAR.light_damage[4], &DAMAGE.gauge_low);
    hold_high(&CAR.light_damage[4], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.gauge_rates[11],
        &DAMAGE.flt_75b3b4,
        &DAMAGE.flt_75b38c,
        &CAR.light_damage[5],
        &CAR.light_damage[5],
    );
    hold_low(&CAR.light_damage[5], &DAMAGE.gauge_low);
    hold_high(&CAR.light_damage[5], &DAMAGE.gauge_top);
    x87_6(&DAMAGE);
    let part = DAMAGE.flt_75b3e4.bits();
    x87_7(
        &DAMAGE,
        ((DAMAGE.kind_table_a[0].addr()).wrapping_add(part.wrapping_mul(4))) as usize,
        ((DAMAGE.kind_table_b[0].addr()).wrapping_add(part.wrapping_mul(4))) as usize,
    );
    x87_5(
        &DAMAGE.flt_75b46c,
        &DAMAGE.flt_75b488,
        &DAMAGE.flt_75b370,
        &CAR.part_hit_damage[0],
        &CAR.part_hit_damage[0],
    );
    hold_low(&CAR.part_hit_damage[0], &DAMAGE.gauge_low);
    hold_high(&CAR.part_hit_damage[0], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b46c,
        &DAMAGE.flt_75b488,
        &DAMAGE.flt_75b370,
        &CAR.part_hit_damage[1],
        &CAR.part_hit_damage[1],
    );
    hold_low(&CAR.part_hit_damage[1], &DAMAGE.gauge_low);
    hold_high(&CAR.part_hit_damage[1], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b470,
        &DAMAGE.flt_75b48c,
        &DAMAGE.flt_75b370,
        &CAR.part_hit_damage[2],
        &CAR.part_hit_damage[2],
    );
    hold_low(&CAR.part_hit_damage[2], &DAMAGE.gauge_low);
    hold_high(&CAR.part_hit_damage[2], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b470,
        &DAMAGE.flt_75b48c,
        &DAMAGE.flt_75b370,
        &CAR.part_hit_damage[3],
        &CAR.part_hit_damage[3],
    );
    hold_low(&CAR.part_hit_damage[3], &DAMAGE.gauge_low);
    hold_high(&CAR.part_hit_damage[3], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b45c,
        &DAMAGE.gauge_rates[13],
        &DAMAGE.flt_75b370,
        &CAR.engine_damage,
        &CAR.engine_damage,
    );
    hold_low(&CAR.engine_damage, &DAMAGE.gauge_low);
    hold_high(&CAR.engine_damage, &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b45c,
        &DAMAGE.gauge_rates[12],
        &DAMAGE.flt_75b370,
        &CAR.cooling_damage,
        &CAR.cooling_damage,
    );
    hold_low(&CAR.cooling_damage, &DAMAGE.gauge_low);
    hold_high(&CAR.cooling_damage, &DAMAGE.gauge_top);
    if compare_full(DAMAGE.flt_75b464.bits(), DAMAGE.gauge_low.bits()) & 0x100 == 0 {
        x87_8(&DAMAGE);
        hold_low(&CAR.engine_damage, &DAMAGE.gauge_low);
        hold_high(&CAR.engine_damage, &DAMAGE.gauge_top);
        x87_5(
            &DAMAGE.flt_75b464,
            &DAMAGE.gauge_rates[15],
            &DAMAGE.flt_75b370,
            &CAR.cooling_damage,
            &CAR.cooling_damage,
        );
        hold_low(&CAR.cooling_damage, &DAMAGE.gauge_low);
        hold_high(&CAR.cooling_damage, &DAMAGE.gauge_top);
    }
    if DAMAGE.dword_75b3b8.bits() == 0x1 {
        x87_9(&DAMAGE.gauge_rates[16]);
    }
    if DAMAGE.dword_75b3b8.bits() == 0xFFFFFFFF {
        x87_9(&DAMAGE.gauge_rates[17]);
    }
    let gear = DAMAGE.dword_75b3bc.bits();
    if DAMAGE.dword_75b3b8.bits() == 0x1 {
        let gauge = gear_gauge(gear);
        x87_10(gauge as usize, &DAMAGE.gauge_rates[18]);
    }
    let gear = DAMAGE.dword_75b3bc.bits();
    if DAMAGE.dword_75b3b8.bits() == 0xFFFFFFFF {
        let gauge = gear_gauge(gear);
        x87_10(gauge as usize, &DAMAGE.gauge_rates[19]);
    }
    x87_5(
        &DAMAGE.flt_75b450,
        &DAMAGE.flt_75b670,
        &DAMAGE.flt_75b370,
        &CAR.wheel_hit_damage[0],
        &CAR.wheel_hit_damage[0],
    );
    hold_low(&CAR.wheel_hit_damage[0], &DAMAGE.gauge_low);
    hold_high(&CAR.wheel_hit_damage[0], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b450,
        &DAMAGE.flt_75b670,
        &DAMAGE.flt_75b370,
        &CAR.wheel_hit_damage[1],
        &CAR.wheel_hit_damage[1],
    );
    hold_low(&CAR.wheel_hit_damage[1], &DAMAGE.gauge_low);
    hold_high(&CAR.wheel_hit_damage[1], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b450,
        &DAMAGE.flt_75b670,
        &DAMAGE.flt_75b370,
        &CAR.wheel_hit_damage[2],
        &CAR.wheel_hit_damage[2],
    );
    hold_low(&CAR.wheel_hit_damage[2], &DAMAGE.gauge_low);
    hold_high(&CAR.wheel_hit_damage[2], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b450,
        &DAMAGE.flt_75b670,
        &DAMAGE.flt_75b370,
        &CAR.wheel_hit_damage[3],
        &CAR.wheel_hit_damage[3],
    );
    hold_low(&CAR.wheel_hit_damage[3], &DAMAGE.gauge_low);
    hold_high(&CAR.wheel_hit_damage[3], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b458,
        &DAMAGE.figures[0],
        &DAMAGE.flt_75b370,
        &CAR.suspension_damage[0],
        &CAR.suspension_damage[0],
    );
    hold_low(&CAR.suspension_damage[0], &DAMAGE.gauge_low);
    hold_high(&CAR.suspension_damage[0], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b458,
        &DAMAGE.figures[0],
        &DAMAGE.flt_75b370,
        &CAR.suspension_damage[1],
        &CAR.suspension_damage[1],
    );
    hold_low(&CAR.suspension_damage[1], &DAMAGE.gauge_low);
    hold_high(&CAR.suspension_damage[1], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b458,
        &DAMAGE.figures[0],
        &DAMAGE.flt_75b370,
        &CAR.suspension_damage[2],
        &CAR.suspension_damage[2],
    );
    hold_low(&CAR.suspension_damage[2], &DAMAGE.gauge_low);
    hold_high(&CAR.suspension_damage[2], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b458,
        &DAMAGE.figures[0],
        &DAMAGE.flt_75b370,
        &CAR.suspension_damage[3],
        &CAR.suspension_damage[3],
    );
    hold_low(&CAR.suspension_damage[3], &DAMAGE.gauge_low);
    hold_high(&CAR.suspension_damage[3], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b454,
        &DAMAGE.figures[1],
        &DAMAGE.flt_75b370,
        &CAR.body_hit_damage,
        &CAR.body_hit_damage,
    );
    hold_low(&CAR.body_hit_damage, &DAMAGE.gauge_low);
    hold_high(&CAR.body_hit_damage, &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b454,
        &DAMAGE.figures[2],
        &DAMAGE.flt_75b370,
        &CAR.steering_damage,
        &CAR.steering_damage,
    );
    hold_low(&CAR.steering_damage, &DAMAGE.gauge_low);
    hold_high(&CAR.steering_damage, &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b454,
        &DAMAGE.figures[3],
        &DAMAGE.flt_75b370,
        &CAR.engine_damage,
        &CAR.engine_damage,
    );
    hold_low(&CAR.engine_damage, &DAMAGE.gauge_low);
    hold_high(&CAR.engine_damage, &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b454,
        &DAMAGE.figures[4],
        &DAMAGE.flt_75b370,
        &CAR.exhaust_damage,
        &CAR.exhaust_damage,
    );
    hold_low(&CAR.exhaust_damage, &DAMAGE.gauge_low);
    hold_high(&CAR.exhaust_damage, &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b454,
        &DAMAGE.figures[5],
        &DAMAGE.flt_75b370,
        &CAR.gear_wear[0],
        &CAR.gear_wear[0],
    );
    hold_low(&CAR.gear_wear[0], &DAMAGE.gauge_low);
    hold_high(&CAR.gear_wear[0], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b454,
        &DAMAGE.figures[5],
        &DAMAGE.flt_75b370,
        &CAR.gear_wear[2],
        &CAR.gear_wear[2],
    );
    hold_low(&CAR.gear_wear[2], &DAMAGE.gauge_low);
    hold_high(&CAR.gear_wear[2], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b454,
        &DAMAGE.figures[5],
        &DAMAGE.flt_75b370,
        &CAR.gear_wear[3],
        &CAR.gear_wear[3],
    );
    hold_low(&CAR.gear_wear[3], &DAMAGE.gauge_low);
    hold_high(&CAR.gear_wear[3], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b454,
        &DAMAGE.figures[5],
        &DAMAGE.flt_75b370,
        &CAR.gear_wear[4],
        &CAR.gear_wear[4],
    );
    hold_low(&CAR.gear_wear[4], &DAMAGE.gauge_low);
    hold_high(&CAR.gear_wear[4], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b454,
        &DAMAGE.figures[5],
        &DAMAGE.flt_75b370,
        &CAR.gear_wear[5],
        &CAR.gear_wear[5],
    );
    hold_low(&CAR.gear_wear[5], &DAMAGE.gauge_low);
    hold_high(&CAR.gear_wear[5], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b454,
        &DAMAGE.figures[5],
        &DAMAGE.flt_75b370,
        &CAR.gear_wear[6],
        &CAR.gear_wear[6],
    );
    hold_low(&CAR.gear_wear[6], &DAMAGE.gauge_low);
    hold_high(&CAR.gear_wear[6], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b454,
        &DAMAGE.figures[5],
        &DAMAGE.flt_75b370,
        &CAR.gear_wear[7],
        &CAR.gear_wear[7],
    );
    hold_low(&CAR.gear_wear[7], &DAMAGE.gauge_low);
    hold_high(&CAR.gear_wear[7], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b454,
        &DAMAGE.figures[5],
        &DAMAGE.flt_75b370,
        &CAR.gear_wear[8],
        &CAR.gear_wear[8],
    );
    hold_low(&CAR.gear_wear[8], &DAMAGE.gauge_low);
    hold_high(&CAR.gear_wear[8], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b454,
        &DAMAGE.figures[6],
        &DAMAGE.flt_75b370,
        &CAR.differential_damage,
        &CAR.differential_damage,
    );
    hold_low(&CAR.differential_damage, &DAMAGE.gauge_low);
    hold_high(&CAR.differential_damage, &DAMAGE.gauge_top);
    if compare_full(DAMAGE.flt_75b3c8.bits(), DAMAGE.gauge_rates[23].bits()) & 0x100 == 0 {
        x87_1(
            &DAMAGE.flt_75b3c8,
            &DAMAGE.figures[13],
            &CAR.body_hit_damage,
            &CAR.body_hit_damage,
        );
        hold_low(&CAR.body_hit_damage, &DAMAGE.gauge_low);
        hold_high(&CAR.body_hit_damage, &DAMAGE.gauge_top);
    }
    if compare_full(DAMAGE.flt_75b3c8.bits(), DAMAGE.gauge_rates[22].bits()) & 0x100 == 0 {
        x87_11(&DAMAGE);
        if DAMAGE.dword_75b3c4.bits() == 0x1 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[24],
                &CAR.light_damage[0],
                &CAR.light_damage[0],
            );
            hold_low(&CAR.light_damage[0], &DAMAGE.gauge_low);
            hold_high(&CAR.light_damage[0], &DAMAGE.gauge_top);
        }
        if DAMAGE.dword_75b3c4.bits() == 0x2 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[24],
                &CAR.light_damage[1],
                &CAR.light_damage[1],
            );
            hold_low(&CAR.light_damage[1], &DAMAGE.gauge_low);
            hold_high(&CAR.light_damage[1], &DAMAGE.gauge_top);
        }
        if DAMAGE.dword_75b3c4.bits() == 0x3 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[24],
                &CAR.light_damage[2],
                &CAR.light_damage[2],
            );
            hold_low(&CAR.light_damage[2], &DAMAGE.gauge_low);
            hold_high(&CAR.light_damage[2], &DAMAGE.gauge_top);
        }
        if DAMAGE.dword_75b3c4.bits() == 0x4 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[24],
                &CAR.light_damage[3],
                &CAR.light_damage[3],
            );
            hold_low(&CAR.light_damage[3], &DAMAGE.gauge_low);
            hold_high(&CAR.light_damage[3], &DAMAGE.gauge_top);
        }
        if DAMAGE.dword_75b3c4.bits() == 0x1 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[25],
                &CAR.light_damage[4],
                &CAR.light_damage[4],
            );
            hold_low(&CAR.light_damage[4], &DAMAGE.gauge_low);
            hold_high(&CAR.light_damage[4], &DAMAGE.gauge_top);
        }
        if DAMAGE.dword_75b3c4.bits() == 0x2 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[25],
                &CAR.light_damage[4],
                &CAR.light_damage[4],
            );
            hold_low(&CAR.light_damage[4], &DAMAGE.gauge_low);
            hold_high(&CAR.light_damage[4], &DAMAGE.gauge_top);
        }
        if DAMAGE.dword_75b3c4.bits() == 0x3 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[25],
                &CAR.light_damage[5],
                &CAR.light_damage[5],
            );
            hold_low(&CAR.light_damage[5], &DAMAGE.gauge_low);
            hold_high(&CAR.light_damage[5], &DAMAGE.gauge_top);
        }
        if DAMAGE.dword_75b3c4.bits() == 0x4 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[25],
                &CAR.light_damage[5],
                &CAR.light_damage[5],
            );
            hold_low(&CAR.light_damage[5], &DAMAGE.gauge_low);
            hold_high(&CAR.light_damage[5], &DAMAGE.gauge_top);
        }
        x87_1(
            &DAMAGE.flt_75b3c8,
            &DAMAGE.figures[23],
            &CAR.steering_damage,
            &CAR.steering_damage,
        );
        hold_low(&CAR.steering_damage, &DAMAGE.gauge_low);
        hold_high(&CAR.steering_damage, &DAMAGE.gauge_top);
        x87_1(
            &DAMAGE.flt_75b3c8,
            &DAMAGE.figures[22],
            &CAR.electrics_damage,
            &CAR.electrics_damage,
        );
        hold_low(&CAR.electrics_damage, &DAMAGE.gauge_low);
        hold_high(&CAR.electrics_damage, &DAMAGE.gauge_top);
        x87_1(
            &DAMAGE.flt_75b3c8,
            &DAMAGE.figures[17],
            &CAR.turbo_damage,
            &CAR.turbo_damage,
        );
        hold_low(&CAR.turbo_damage, &DAMAGE.gauge_low);
        hold_high(&CAR.turbo_damage, &DAMAGE.gauge_top);
        x87_1(
            &DAMAGE.flt_75b3c8,
            &DAMAGE.figures[18],
            &CAR.clutch_damage,
            &CAR.clutch_damage,
        );
        hold_low(&CAR.clutch_damage, &DAMAGE.gauge_low);
        hold_high(&CAR.clutch_damage, &DAMAGE.gauge_top);
        x87_1(
            &DAMAGE.flt_75b3c8,
            &DAMAGE.figures[19],
            &CAR.differential_damage,
            &CAR.differential_damage,
        );
        hold_low(&CAR.differential_damage, &DAMAGE.gauge_low);
        hold_high(&CAR.differential_damage, &DAMAGE.gauge_top);
        x87_1(
            &DAMAGE.flt_75b3c8,
            &DAMAGE.figures[20],
            &CAR.gear_wear[0],
            &CAR.gear_wear[0],
        );
        hold_low(&CAR.gear_wear[0], &DAMAGE.gauge_low);
        hold_high(&CAR.gear_wear[0], &DAMAGE.gauge_top);
        x87_1(
            &DAMAGE.flt_75b3c8,
            &DAMAGE.figures[20],
            &CAR.gear_wear[1],
            &CAR.gear_wear[1],
        );
        hold_low(&CAR.gear_wear[1], &DAMAGE.gauge_low);
        hold_high(&CAR.gear_wear[1], &DAMAGE.gauge_top);
        x87_1(
            &DAMAGE.flt_75b3c8,
            &DAMAGE.figures[20],
            &CAR.gear_wear[2],
            &CAR.gear_wear[2],
        );
        hold_low(&CAR.gear_wear[2], &DAMAGE.gauge_low);
        hold_high(&CAR.gear_wear[2], &DAMAGE.gauge_top);
        x87_1(
            &DAMAGE.flt_75b3c8,
            &DAMAGE.figures[20],
            &CAR.gear_wear[3],
            &CAR.gear_wear[3],
        );
        hold_low(&CAR.gear_wear[3], &DAMAGE.gauge_low);
        hold_high(&CAR.gear_wear[3], &DAMAGE.gauge_top);
        x87_1(
            &DAMAGE.flt_75b3c8,
            &DAMAGE.figures[20],
            &CAR.gear_wear[4],
            &CAR.gear_wear[4],
        );
        hold_low(&CAR.gear_wear[4], &DAMAGE.gauge_low);
        hold_high(&CAR.gear_wear[4], &DAMAGE.gauge_top);
        x87_1(
            &DAMAGE.flt_75b3c8,
            &DAMAGE.figures[20],
            &CAR.gear_wear[5],
            &CAR.gear_wear[5],
        );
        hold_low(&CAR.gear_wear[5], &DAMAGE.gauge_low);
        hold_high(&CAR.gear_wear[5], &DAMAGE.gauge_top);
        x87_1(
            &DAMAGE.flt_75b3c8,
            &DAMAGE.figures[20],
            &CAR.gear_wear[6],
            &CAR.gear_wear[6],
        );
        hold_low(&CAR.gear_wear[6], &DAMAGE.gauge_low);
        hold_high(&CAR.gear_wear[6], &DAMAGE.gauge_top);
        x87_1(
            &DAMAGE.flt_75b3c8,
            &DAMAGE.figures[20],
            &CAR.gear_wear[7],
            &CAR.gear_wear[7],
        );
        hold_low(&CAR.gear_wear[7], &DAMAGE.gauge_low);
        hold_high(&CAR.gear_wear[7], &DAMAGE.gauge_top);
        x87_1(
            &DAMAGE.flt_75b3c8,
            &DAMAGE.figures[20],
            &CAR.gear_wear[8],
            &CAR.gear_wear[8],
        );
        hold_low(&CAR.gear_wear[8], &DAMAGE.gauge_low);
        hold_high(&CAR.gear_wear[8], &DAMAGE.gauge_top);
        let gear = DAMAGE.dword_75b3bc.bits();
        let gauge = gear_gauge(gear);
        x87_12(&DAMAGE, gauge as usize);
        hold_at(gauge);
        if DAMAGE.dword_75b3c4.bits() == 0x1 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[28],
                &CAR.wheel_hit_damage[0],
                &CAR.wheel_hit_damage[0],
            );
            hold_low(&CAR.wheel_hit_damage[0], &DAMAGE.gauge_low);
            hold_high(&CAR.wheel_hit_damage[0], &DAMAGE.gauge_top);
        }
        if DAMAGE.dword_75b3c4.bits() == 0x2 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[28],
                &CAR.wheel_hit_damage[1],
                &CAR.wheel_hit_damage[1],
            );
            hold_low(&CAR.wheel_hit_damage[1], &DAMAGE.gauge_low);
            hold_high(&CAR.wheel_hit_damage[1], &DAMAGE.gauge_top);
        }
        if DAMAGE.dword_75b3c4.bits() == 0x3 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[28],
                &CAR.wheel_hit_damage[2],
                &CAR.wheel_hit_damage[2],
            );
            hold_low(&CAR.wheel_hit_damage[2], &DAMAGE.gauge_low);
            hold_high(&CAR.wheel_hit_damage[2], &DAMAGE.gauge_top);
        }
        if DAMAGE.dword_75b3c4.bits() == 0x4 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[28],
                &CAR.wheel_hit_damage[3],
                &CAR.wheel_hit_damage[3],
            );
            hold_low(&CAR.wheel_hit_damage[3], &DAMAGE.gauge_low);
            hold_high(&CAR.wheel_hit_damage[3], &DAMAGE.gauge_top);
        }
        if DAMAGE.dword_75b3c4.bits() == 0x1 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[30],
                &CAR.suspension_damage[0],
                &CAR.suspension_damage[0],
            );
            hold_low(&CAR.suspension_damage[0], &DAMAGE.gauge_low);
            hold_high(&CAR.suspension_damage[0], &DAMAGE.gauge_top);
        }
        if DAMAGE.dword_75b3c4.bits() == 0x2 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[30],
                &CAR.suspension_damage[1],
                &CAR.suspension_damage[1],
            );
            hold_low(&CAR.suspension_damage[1], &DAMAGE.gauge_low);
            hold_high(&CAR.suspension_damage[1], &DAMAGE.gauge_top);
        }
        if DAMAGE.dword_75b3c4.bits() == 0x3 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[30],
                &CAR.suspension_damage[2],
                &CAR.suspension_damage[2],
            );
            hold_low(&CAR.suspension_damage[2], &DAMAGE.gauge_low);
            hold_high(&CAR.suspension_damage[2], &DAMAGE.gauge_top);
        }
        if DAMAGE.dword_75b3c4.bits() == 0x4 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[30],
                &CAR.suspension_damage[3],
                &CAR.suspension_damage[3],
            );
            hold_low(&CAR.suspension_damage[3], &DAMAGE.gauge_low);
            hold_high(&CAR.suspension_damage[3], &DAMAGE.gauge_top);
        }
        if DAMAGE.dword_75b3c4.bits() == 0x1 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[27],
                &CAR.cooling_damage,
                &CAR.cooling_damage,
            );
            hold_low(&CAR.cooling_damage, &DAMAGE.gauge_low);
            hold_high(&CAR.cooling_damage, &DAMAGE.gauge_top);
        }
        if DAMAGE.dword_75b3c4.bits() == 0x2 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[27],
                &CAR.cooling_damage,
                &CAR.cooling_damage,
            );
            hold_low(&CAR.cooling_damage, &DAMAGE.gauge_low);
            hold_high(&CAR.cooling_damage, &DAMAGE.gauge_top);
        }
        if DAMAGE.dword_75b3c4.bits() == 0x1 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[26],
                &CAR.engine_damage,
                &CAR.engine_damage,
            );
            hold_low(&CAR.engine_damage, &DAMAGE.gauge_low);
            hold_high(&CAR.engine_damage, &DAMAGE.gauge_top);
        }
        if DAMAGE.dword_75b3c4.bits() == 0x2 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[26],
                &CAR.engine_damage,
                &CAR.engine_damage,
            );
            hold_low(&CAR.engine_damage, &DAMAGE.gauge_low);
            hold_high(&CAR.engine_damage, &DAMAGE.gauge_top);
        }
        if DAMAGE.dword_75b3c4.bits() == 0x1 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[29],
                &CAR.brake_wear_a,
                &CAR.brake_wear_a,
            );
            hold_low(&CAR.brake_wear_a, &DAMAGE.gauge_low);
            hold_high(&CAR.brake_wear_a, &DAMAGE.gauge_top);
        }
        if DAMAGE.dword_75b3c4.bits() == 0x2 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[29],
                &CAR.brake_wear_a,
                &CAR.brake_wear_a,
            );
            hold_low(&CAR.brake_wear_a, &DAMAGE.gauge_low);
            hold_high(&CAR.brake_wear_a, &DAMAGE.gauge_top);
        }
        if DAMAGE.dword_75b3c4.bits() == 0x3 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[29],
                &CAR.brake_wear_b,
                &CAR.brake_wear_b,
            );
            hold_low(&CAR.brake_wear_b, &DAMAGE.gauge_low);
            hold_high(&CAR.brake_wear_b, &DAMAGE.gauge_top);
        }
        if DAMAGE.dword_75b3c4.bits() == 0x4 {
            x87_1(
                &DAMAGE.flt_75b3c8,
                &DAMAGE.figures[29],
                &CAR.brake_wear_b,
                &CAR.brake_wear_b,
            );
            hold_low(&CAR.brake_wear_b, &DAMAGE.gauge_low);
            hold_high(&CAR.brake_wear_b, &DAMAGE.gauge_top);
        }
    }
    x87_5(
        &DAMAGE.wheel_loads[0],
        &DAMAGE.figures[14],
        &DAMAGE.flt_75b370,
        &CAR.suspension_damage[0],
        &CAR.suspension_damage[0],
    );
    hold_low(&CAR.suspension_damage[0], &DAMAGE.gauge_low);
    hold_high(&CAR.suspension_damage[0], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.wheel_loads[1],
        &DAMAGE.figures[14],
        &DAMAGE.flt_75b370,
        &CAR.suspension_damage[1],
        &CAR.suspension_damage[1],
    );
    hold_low(&CAR.suspension_damage[1], &DAMAGE.gauge_low);
    hold_high(&CAR.suspension_damage[1], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.wheel_loads[2],
        &DAMAGE.figures[14],
        &DAMAGE.flt_75b370,
        &CAR.suspension_damage[2],
        &CAR.suspension_damage[2],
    );
    hold_low(&CAR.suspension_damage[2], &DAMAGE.gauge_low);
    hold_high(&CAR.suspension_damage[2], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.wheel_loads[3],
        &DAMAGE.figures[14],
        &DAMAGE.flt_75b370,
        &CAR.suspension_damage[3],
        &CAR.suspension_damage[3],
    );
    hold_low(&CAR.suspension_damage[3], &DAMAGE.gauge_low);
    hold_high(&CAR.suspension_damage[3], &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b3dc,
        &DAMAGE.figures[15],
        &DAMAGE.flt_75b370,
        &CAR.exhaust_damage,
        &CAR.exhaust_damage,
    );
    hold_low(&CAR.exhaust_damage, &DAMAGE.gauge_low);
    hold_high(&CAR.exhaust_damage, &DAMAGE.gauge_top);
    x87_5(
        &DAMAGE.flt_75b3dc,
        &DAMAGE.figures[16],
        &DAMAGE.flt_75b370,
        &CAR.steering_damage,
        &CAR.steering_damage,
    );
    hold_low(&CAR.steering_damage, &DAMAGE.gauge_low);
    hold_high(&CAR.steering_damage, &DAMAGE.gauge_top);
    x87_13(&DAMAGE);
    if DAMAGE.flt_75b3e4.bits() == 0x0 {
        CAR.turbo_damage.set_bits(0x0);
    }
    // retn
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o3] = ([m + o0] * [m + o1]) + [rc + o2]`
fn x87_1(
    o0: &impl Cell32,
    o1: &impl Cell32,
    o2: &impl crate::session::Cell32,
    o3: &impl crate::session::Cell32,
) {
    let t0 = ld(o0.bits());
    let t1 = ld(o1.bits());
    let t2 = t0 * t1;
    let t3 = ld(o2.bits());
    let t4 = t2 + t3;
    o3.set_bits(st(t4));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o5] = ((([m + o0] * [m + o1]) * [m + o2]) * [m + 0x35B38C]) + [rc + o4]`
fn x87_2(
    m: &DamageModel,
    o0: &impl Cell32,
    o1: &impl Cell32,
    o2: &impl Cell32,
    o4: &impl crate::session::Cell32,
    o5: &impl crate::session::Cell32,
) {
    let t0 = ld(o0.bits());
    let t1 = ld(o1.bits());
    let t2 = t0 * t1;
    let t3 = ld(o2.bits());
    let t4 = t2 * t3;
    let t5 = ld(m.flt_75b38c.bits());
    let t6 = t4 * t5;
    let t7 = ld(o4.bits());
    let t8 = t6 + t7;
    o5.set_bits(st(t8));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[v + 0] = ((([m + 0x35B398] * [m + 0x35B5A8]) * [m + 0x35B370]) * [t + 0]) + [v + 0]`
fn x87_3(m: &DamageModel, t: usize, v: usize) {
    let t0 = ld(m.flt_75b398.bits());
    let t1 = ld(m.wear_rates[12].bits());
    let t2 = t0 * t1;
    let t3 = ld(m.flt_75b370.bits());
    let t4 = t2 * t3;
    let t5 = fld(t, 0);
    let t6 = t4 * t5;
    let t7 = fld(v, 0);
    let t8 = t6 + t7;
    fstp(v, 0, t8);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B43C] = 1 - [m + 0x35B400]`
/// `[rc + 0x31A320] = ((([m + 0x35B5F4] * [m + 0x35B400]) * [m + 0x35B3B0]) * [m + 0x35B38C]) + [rc + 0x31A320]`
fn x87_4(m: &DamageModel) {
    let t0 = X(1.0);
    let t1 = ld(m.flt_75b400.bits());
    let t2 = t0 - t1;
    m.flt_75b43c.set_bits(st(t2));
    let t3 = ld(m.gauge_rates[9].bits());
    let t4 = ld(m.flt_75b400.bits());
    let t5 = t3 * t4;
    let t6 = ld(m.flt_75b3b0.bits());
    let t7 = t5 * t6;
    let t8 = ld(m.flt_75b38c.bits());
    let t9 = t7 * t8;
    let t10 = ld(CAR.brake_wear_a.bits());
    let t11 = t9 + t10;
    CAR.brake_wear_a.set_bits(st(t11));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o4] = (([m + o0] * [m + o1]) * [m + o2]) + [rc + o3]`
fn x87_5(
    o0: &impl Cell32,
    o1: &impl Cell32,
    o2: &impl Cell32,
    o3: &impl crate::session::Cell32,
    o4: &impl crate::session::Cell32,
) {
    let t0 = ld(o0.bits());
    let t1 = ld(o1.bits());
    let t2 = t0 * t1;
    let t3 = ld(o2.bits());
    let t4 = t2 * t3;
    let t5 = ld(o3.bits());
    let t6 = t4 + t5;
    o4.set_bits(st(t6));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B488] = (([m + 0x35B39C] / ([m + 0x35B484] * [m + 0x35B380])) * [m + 0x35B34C]) * [m + 0x35B3F4]`
/// `[m + 0x35B48C] = (([m + 0x35B39C] / ([m + 0x35B484] * [m + 0x35B380])) * [m + 0x35B34C]) * [m + 0x35B3F4]`
fn x87_6(m: &DamageModel) {
    let t0 = ld(m.flt_75b39c.bits());
    let t1 = ld(m.flt_75b484.bits());
    let t2 = ld(m.flt_75b380.bits());
    let t3 = t1 * t2;
    let t4 = t0 / t3;
    let t5 = ld(m.gauge_top.bits());
    let t6 = t4 * t5;
    let t7 = ld(m.flt_75b3f4.bits());
    let t8 = t6 * t7;
    m.flt_75b488.set_bits(st(t8));
    m.flt_75b48c.set_bits(st(t8));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B488] = [m + 0x35B488] * [t + 0]`
/// `[m + 0x35B48C] = [m + 0x35B48C] * [u + 0]`
fn x87_7(m: &DamageModel, t: usize, u: usize) {
    let t0 = ld(m.flt_75b488.bits());
    let t1 = fld(t, 0);
    let t2 = t0 * t1;
    m.flt_75b488.set_bits(st(t2));
    let t3 = ld(m.flt_75b48c.bits());
    let t4 = fld(u, 0);
    let t5 = t3 * t4;
    m.flt_75b48c.set_bits(st(t5));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A290] = (([rc + 0x31A28C] * [m + 0x35B608]) * [m + 0x35B370]) + [rc + 0x31A290]`
fn x87_8(m: &DamageModel) {
    let t0 = ld(CAR.engine_heat.bits());
    let t1 = ld(m.gauge_rates[14].bits());
    let t2 = t0 * t1;
    let t3 = ld(m.flt_75b370.bits());
    let t4 = t2 * t3;
    let t5 = ld(CAR.engine_damage.bits());
    let t6 = t4 + t5;
    CAR.engine_damage.set_bits(st(t6));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A2A0] = [rc + 0x31A2A0] + [m + o1]`
fn x87_9(o1: &impl Cell32) {
    let t0 = ld(CAR.clutch_damage.bits());
    let t1 = ld(o1.bits());
    let t2 = t0 + t1;
    CAR.clutch_damage.set_bits(st(t2));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[v + 0] = [v + 0] + [m + o1]`
fn x87_10(v: usize, o1: &impl Cell32) {
    let t0 = fld(v, 0);
    let t1 = ld(o1.bits());
    let t2 = t0 + t1;
    fstp(v, 0, t2);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3C8] = [m + 0x35B3C8] * [rc + 0x31C708]`
fn x87_11(m: &DamageModel) {
    let t0 = ld(m.flt_75b3c8.bits());
    let t1 = ld(RACE_WORK.patch_factors[1].bits());
    let t2 = t0 * t1;
    m.flt_75b3c8.set_bits(st(t2));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[v + 0] = ([m + 0x35B3C8] * [m + 0x35B6CC]) + [v + 0]`
fn x87_12(m: &DamageModel, v: usize) {
    let t0 = ld(m.flt_75b3c8.bits());
    let t1 = ld(m.figures[21].bits());
    let t2 = t0 * t1;
    let t3 = fld(v, 0);
    let t4 = t2 + t3;
    fstp(v, 0, t4);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A304] = ((((([rc + 0x31A2EC] + [rc + 0x31A2F0]) + [rc + 0x31A2F4]) + [rc + 0x31A2F8]) + [rc + 0x31A300]) + [rc + 0x31A2FC]) / [m + 0x35B344]`
/// `[rc + 0x31A328] = ([rc + 0x31A320] + [rc + 0x31A324]) * [m + 0x35B360]`
/// `[m + 0x35B440] = ([rc + 0x31A32C] + [rc + 0x31A330]) * [m + 0x35B360]`
/// `[m + 0x35B444] = ([rc + 0x31A334] + [rc + 0x31A338]) * [m + 0x35B360]`
/// `[rc + 0x31A33C] = ([m + 0x35B440] + [m + 0x35B444]) * [m + 0x35B360]`
/// (a long formula: see the instructions in the tests' `asm_reference`)
/// (a long formula: see the instructions in the tests' `asm_reference`)
/// `[rc + 0x31A31C] = ((([rc + 0x31A30C] + [rc + 0x31A310]) + [rc + 0x31A314]) + [rc + 0x31A318]) * [m + 0x35B364]`
/// `[rc + 0x31A430] = ((([rc + 0x31A420] + [rc + 0x31A424]) + [rc + 0x31A428]) + [rc + 0x31A42C]) * [m + 0x35B364]`
fn x87_13(m: &DamageModel) {
    let t0 = ld(CAR.light_damage[0].bits());
    let t1 = ld(CAR.light_damage[1].bits());
    let t2 = t0 + t1;
    let t3 = ld(CAR.light_damage[2].bits());
    let t4 = t2 + t3;
    let t5 = ld(CAR.light_damage[3].bits());
    let t6 = t4 + t5;
    let t7 = ld(CAR.light_damage[5].bits());
    let t8 = t6 + t7;
    let t9 = ld(CAR.light_damage[4].bits());
    let t10 = t8 + t9;
    let t11 = ld(m.flt_75b344.bits());
    let t12 = t10 / t11;
    CAR.lights_damage.set_bits(st(t12));
    let t13 = ld(CAR.brake_wear_a.bits());
    let t14 = ld(CAR.brake_wear_b.bits());
    let t15 = t13 + t14;
    let t16 = ld(m.flt_75b360.bits());
    let t17 = t15 * t16;
    CAR.brake_damage.set_bits(st(t17));
    let t18 = ld(CAR.part_hit_damage[0].bits());
    let t19 = ld(CAR.part_hit_damage[1].bits());
    let t20 = t18 + t19;
    let t21 = ld(m.flt_75b360.bits());
    let t22 = t20 * t21;
    m.flt_75b440.set_bits(st(t22));
    let t23 = ld(CAR.part_hit_damage[2].bits());
    let t24 = ld(CAR.part_hit_damage[3].bits());
    let t25 = t23 + t24;
    let t26 = ld(m.flt_75b360.bits());
    let t27 = t25 * t26;
    m.flt_75b444.set_bits(st(t27));
    let t28 = ld(m.flt_75b440.bits());
    let t29 = ld(m.flt_75b444.bits());
    let t30 = t28 + t29;
    let t31 = ld(m.flt_75b360.bits());
    let t32 = t30 * t31;
    CAR.tyre_wear.set_bits(st(t32));
    let t33 = ld(CAR.gear_wear[0].bits());
    let t34 = ld(CAR.gear_wear[2].bits());
    let t35 = t33 + t34;
    let t36 = ld(CAR.gear_wear[3].bits());
    let t37 = t35 + t36;
    let t38 = ld(CAR.gear_wear[4].bits());
    let t39 = t37 + t38;
    let t40 = ld(CAR.gear_wear[5].bits());
    let t41 = t39 + t40;
    let t42 = ld(CAR.gear_wear[6].bits());
    let t43 = t41 + t42;
    let t44 = ld(CAR.gear_wear[7].bits());
    let t45 = t43 + t44;
    let t46 = ld(CAR.gear_wear[8].bits());
    let t47 = t45 + t46;
    let t48 = ld(m.flt_75b360.bits());
    let t49 = t47 * t48;
    let t50 = ld(m.flt_75b360.bits());
    let t51 = t49 * t50;
    let t52 = ld(m.flt_75b360.bits());
    let t53 = t51 * t52;
    CAR.gearbox_damage.set_bits(st(t53));
    let t54 = ld(CAR.wheel_hit_damage[0].bits());
    let t55 = ld(CAR.wheel_hit_damage[1].bits());
    let t56 = t54 + t55;
    let t57 = ld(CAR.wheel_hit_damage[2].bits());
    let t58 = t56 + t57;
    let t59 = ld(CAR.wheel_hit_damage[3].bits());
    let t60 = t58 + t59;
    let t61 = ld(CAR.body_hit_damage.bits());
    let t62 = t60 + t61;
    let t63 = ld(CAR.body_hit_damage.bits());
    let t64 = t62 + t63;
    let t65 = ld(CAR.body_hit_damage.bits());
    let t66 = t64 + t65;
    let t67 = ld(CAR.body_hit_damage.bits());
    let t68 = t66 + t67;
    let t69 = ld(CAR.body_hit_damage.bits());
    let t70 = t68 + t69;
    let t71 = ld(CAR.body_hit_damage.bits());
    let t72 = t70 + t71;
    let t73 = ld(m.flt_75b368.bits());
    let t74 = t72 * t73;
    CAR.body_damage.set_bits(st(t74));
    let t75 = ld(CAR.suspension_damage[0].bits());
    let t76 = ld(CAR.suspension_damage[1].bits());
    let t77 = t75 + t76;
    let t78 = ld(CAR.suspension_damage[2].bits());
    let t79 = t77 + t78;
    let t80 = ld(CAR.suspension_damage[3].bits());
    let t81 = t79 + t80;
    let t82 = ld(m.flt_75b364.bits());
    let t83 = t81 * t82;
    CAR.overall_suspension_damage.set_bits(st(t83));
    let t84 = ld(CAR.wheel_stiffness[0].bits());
    let t85 = ld(CAR.wheel_stiffness[1].bits());
    let t86 = t84 + t85;
    let t87 = ld(CAR.wheel_stiffness[2].bits());
    let t88 = t86 + t87;
    let t89 = ld(CAR.wheel_stiffness[3].bits());
    let t90 = t88 + t89;
    let t91 = ld(m.flt_75b364.bits());
    let t92 = t90 * t91;
    CAR.stiffness_mean.set_bits(st(t92));
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

    // The original's instructions, one function per run.
    reference!(ref_0(m, rc) "fld dword ptr [{m} + 0x35B5D0]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A290]", "fstp dword ptr [{rc} + 0x31A290]");
    reference!(ref_1(m, rc) "fld dword ptr [{m} + 0x35B5D4]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A294]", "fstp dword ptr [{rc} + 0x31A294]");
    reference!(ref_2(m, rc) "fld dword ptr [{m} + 0x35B5D8]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A29C]", "fstp dword ptr [{rc} + 0x31A29C]");
    reference!(ref_3(m, rc) "fld dword ptr [{m} + 0x35B5DC]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A2A0]", "fstp dword ptr [{rc} + 0x31A2A0]");
    reference!(ref_4(m, rc) "fld dword ptr [{m} + 0x35B5E0]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A2A4]", "fstp dword ptr [{rc} + 0x31A2A4]");
    reference!(ref_5(m, rc) "fld dword ptr [{m} + 0x35B5E4]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A2E8]", "fstp dword ptr [{rc} + 0x31A2E8]");
    reference!(ref_6(m, rc) "fld dword ptr [{m} + 0x35B5E8]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A2EC]", "fstp dword ptr [{rc} + 0x31A2EC]");
    reference!(ref_7(m, rc) "fld dword ptr [{m} + 0x35B5E8]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A2F0]", "fstp dword ptr [{rc} + 0x31A2F0]");
    reference!(ref_8(m, rc) "fld dword ptr [{m} + 0x35B5E8]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A2F4]", "fstp dword ptr [{rc} + 0x31A2F4]");
    reference!(ref_9(m, rc) "fld dword ptr [{m} + 0x35B5E8]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A2F8]", "fstp dword ptr [{rc} + 0x31A2F8]");
    reference!(ref_10(m, rc) "fld dword ptr [{m} + 0x35B5E8]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A2FC]", "fstp dword ptr [{rc} + 0x31A2FC]");
    reference!(ref_11(m, rc) "fld dword ptr [{m} + 0x35B5E8]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A300]", "fstp dword ptr [{rc} + 0x31A300]");
    reference!(ref_12(m, rc) "fld dword ptr [{m} + 0x35B5EC]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A308]", "fstp dword ptr [{rc} + 0x31A308]");
    reference!(ref_13(m, rc) "fld dword ptr [{m} + 0x35B5F0]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A30C]", "fstp dword ptr [{rc} + 0x31A30C]");
    reference!(ref_14(m, rc) "fld dword ptr [{m} + 0x35B5F0]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A310]", "fstp dword ptr [{rc} + 0x31A310]");
    reference!(ref_15(m, rc) "fld dword ptr [{m} + 0x35B5F0]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A314]", "fstp dword ptr [{rc} + 0x31A314]");
    reference!(ref_16(m, rc) "fld dword ptr [{m} + 0x35B5F0]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A318]", "fstp dword ptr [{rc} + 0x31A318]");
    reference!(ref_17(m, rc) "fld dword ptr [{m} + 0x35B5A4]", "fmul dword ptr [{m} + 0x35B3C0]", "fmul dword ptr [{m} + 0x35B398]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A298]", "fstp dword ptr [{rc} + 0x31A298]");
    reference!(ref_18(m, t, v) "fld dword ptr [{m} + 0x35B398]", "fmul dword ptr [{m} + 0x35B5A8]", "fmul dword ptr [{m} + 0x35B370]", "fmul dword ptr [{t}]", "fadd dword ptr [{v}]", "fstp dword ptr [{v}]");
    reference!(ref_19(m, rc) ".byte 0xD9, 0xE8", "fsub dword ptr [{m} + 0x35B400]", "fstp dword ptr [{m} + 0x35B43C]", "fld dword ptr [{m} + 0x35B5F4]", "fmul dword ptr [{m} + 0x35B400]", "fmul dword ptr [{m} + 0x35B3B0]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A320]", "fstp dword ptr [{rc} + 0x31A320]");
    reference!(ref_20(m, rc) "fld dword ptr [{m} + 0x35B5F4]", "fmul dword ptr [{m} + 0x35B43C]", "fmul dword ptr [{m} + 0x35B3B0]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A324]", "fstp dword ptr [{rc} + 0x31A324]");
    reference!(ref_21(m, rc) "fld dword ptr [{m} + 0x35B5F8]", "fmul dword ptr [{m} + 0x35B3B4]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A2E8]", "fstp dword ptr [{rc} + 0x31A2E8]");
    reference!(ref_22(m, rc) "fld dword ptr [{m} + 0x35B5FC]", "fmul dword ptr [{m} + 0x35B3B4]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A2EC]", "fstp dword ptr [{rc} + 0x31A2EC]");
    reference!(ref_23(m, rc) "fld dword ptr [{m} + 0x35B5FC]", "fmul dword ptr [{m} + 0x35B3B4]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A2F0]", "fstp dword ptr [{rc} + 0x31A2F0]");
    reference!(ref_24(m, rc) "fld dword ptr [{m} + 0x35B5FC]", "fmul dword ptr [{m} + 0x35B3B4]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A2F4]", "fstp dword ptr [{rc} + 0x31A2F4]");
    reference!(ref_25(m, rc) "fld dword ptr [{m} + 0x35B5FC]", "fmul dword ptr [{m} + 0x35B3B4]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A2F8]", "fstp dword ptr [{rc} + 0x31A2F8]");
    reference!(ref_26(m, rc) "fld dword ptr [{m} + 0x35B5FC]", "fmul dword ptr [{m} + 0x35B3B4]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A2FC]", "fstp dword ptr [{rc} + 0x31A2FC]");
    reference!(ref_27(m, rc) "fld dword ptr [{m} + 0x35B5FC]", "fmul dword ptr [{m} + 0x35B3B4]", "fmul dword ptr [{m} + 0x35B38C]", "fadd dword ptr [{rc} + 0x31A300]", "fstp dword ptr [{rc} + 0x31A300]");
    reference!(ref_28(m) "fld dword ptr [{m} + 0x35B39C]", "fld dword ptr [{m} + 0x35B484]", "fmul dword ptr [{m} + 0x35B380]", ".byte 0xDE, 0xF9", "fmul dword ptr [{m} + 0x35B34C]", "fmul dword ptr [{m} + 0x35B3F4]", "fst dword ptr [{m} + 0x35B488]", "fstp dword ptr [{m} + 0x35B48C]");
    reference!(ref_29(m, t, u) "fld dword ptr [{m} + 0x35B488]", "fmul dword ptr [{t}]", "fstp dword ptr [{m} + 0x35B488]", "fld dword ptr [{m} + 0x35B48C]", "fmul dword ptr [{u}]", "fstp dword ptr [{m} + 0x35B48C]");
    reference!(ref_30(m, rc) "fld dword ptr [{m} + 0x35B46C]", "fmul dword ptr [{m} + 0x35B488]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A32C]", "fstp dword ptr [{rc} + 0x31A32C]");
    reference!(ref_31(m, rc) "fld dword ptr [{m} + 0x35B46C]", "fmul dword ptr [{m} + 0x35B488]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A330]", "fstp dword ptr [{rc} + 0x31A330]");
    reference!(ref_32(m, rc) "fld dword ptr [{m} + 0x35B470]", "fmul dword ptr [{m} + 0x35B48C]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A334]", "fstp dword ptr [{rc} + 0x31A334]");
    reference!(ref_33(m, rc) "fld dword ptr [{m} + 0x35B470]", "fmul dword ptr [{m} + 0x35B48C]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A338]", "fstp dword ptr [{rc} + 0x31A338]");
    reference!(ref_34(m, rc) "fld dword ptr [{m} + 0x35B45C]", "fmul dword ptr [{m} + 0x35B604]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A290]", "fstp dword ptr [{rc} + 0x31A290]");
    reference!(ref_35(m, rc) "fld dword ptr [{m} + 0x35B45C]", "fmul dword ptr [{m} + 0x35B600]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A29C]", "fstp dword ptr [{rc} + 0x31A29C]");
    reference!(ref_36(rc, m) "fld dword ptr [{rc} + 0x31A28C]", "fmul dword ptr [{m} + 0x35B608]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A290]", "fstp dword ptr [{rc} + 0x31A290]");
    reference!(ref_37(m, rc) "fld dword ptr [{m} + 0x35B464]", "fmul dword ptr [{m} + 0x35B60C]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A29C]", "fstp dword ptr [{rc} + 0x31A29C]");
    reference!(ref_38(rc, m) "fld dword ptr [{rc} + 0x31A2A0]", "fadd dword ptr [{m} + 0x35B610]", "fstp dword ptr [{rc} + 0x31A2A0]");
    reference!(ref_39(rc, m) "fld dword ptr [{rc} + 0x31A2A0]", "fadd dword ptr [{m} + 0x35B614]", "fstp dword ptr [{rc} + 0x31A2A0]");
    reference!(ref_40(v, m) "fld dword ptr [{v}]", "fadd dword ptr [{m} + 0x35B618]", "fstp dword ptr [{v}]");
    reference!(ref_41(v, m) "fld dword ptr [{v}]", "fadd dword ptr [{m} + 0x35B61C]", "fstp dword ptr [{v}]");
    reference!(ref_42(m, rc) "fld dword ptr [{m} + 0x35B450]", "fmul dword ptr [{m} + 0x35B670]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A2D0]", "fstp dword ptr [{rc} + 0x31A2D0]");
    reference!(ref_43(m, rc) "fld dword ptr [{m} + 0x35B450]", "fmul dword ptr [{m} + 0x35B670]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A2D4]", "fstp dword ptr [{rc} + 0x31A2D4]");
    reference!(ref_44(m, rc) "fld dword ptr [{m} + 0x35B450]", "fmul dword ptr [{m} + 0x35B670]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A2D8]", "fstp dword ptr [{rc} + 0x31A2D8]");
    reference!(ref_45(m, rc) "fld dword ptr [{m} + 0x35B450]", "fmul dword ptr [{m} + 0x35B670]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A2DC]", "fstp dword ptr [{rc} + 0x31A2DC]");
    reference!(ref_46(m, rc) "fld dword ptr [{m} + 0x35B458]", "fmul dword ptr [{m} + 0x35B678]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A30C]", "fstp dword ptr [{rc} + 0x31A30C]");
    reference!(ref_47(m, rc) "fld dword ptr [{m} + 0x35B458]", "fmul dword ptr [{m} + 0x35B678]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A310]", "fstp dword ptr [{rc} + 0x31A310]");
    reference!(ref_48(m, rc) "fld dword ptr [{m} + 0x35B458]", "fmul dword ptr [{m} + 0x35B678]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A314]", "fstp dword ptr [{rc} + 0x31A314]");
    reference!(ref_49(m, rc) "fld dword ptr [{m} + 0x35B458]", "fmul dword ptr [{m} + 0x35B678]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A318]", "fstp dword ptr [{rc} + 0x31A318]");
    reference!(ref_50(m, rc) "fld dword ptr [{m} + 0x35B454]", "fmul dword ptr [{m} + 0x35B67C]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A2E0]", "fstp dword ptr [{rc} + 0x31A2E0]");
    reference!(ref_51(m, rc) "fld dword ptr [{m} + 0x35B454]", "fmul dword ptr [{m} + 0x35B680]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A308]", "fstp dword ptr [{rc} + 0x31A308]");
    reference!(ref_52(m, rc) "fld dword ptr [{m} + 0x35B454]", "fmul dword ptr [{m} + 0x35B684]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A290]", "fstp dword ptr [{rc} + 0x31A290]");
    reference!(ref_53(m, rc) "fld dword ptr [{m} + 0x35B454]", "fmul dword ptr [{m} + 0x35B688]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A294]", "fstp dword ptr [{rc} + 0x31A294]");
    reference!(ref_54(m, rc) "fld dword ptr [{m} + 0x35B454]", "fmul dword ptr [{m} + 0x35B68C]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A2A8]", "fstp dword ptr [{rc} + 0x31A2A8]");
    reference!(ref_55(m, rc) "fld dword ptr [{m} + 0x35B454]", "fmul dword ptr [{m} + 0x35B68C]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A2B0]", "fstp dword ptr [{rc} + 0x31A2B0]");
    reference!(ref_56(m, rc) "fld dword ptr [{m} + 0x35B454]", "fmul dword ptr [{m} + 0x35B68C]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A2B4]", "fstp dword ptr [{rc} + 0x31A2B4]");
    reference!(ref_57(m, rc) "fld dword ptr [{m} + 0x35B454]", "fmul dword ptr [{m} + 0x35B68C]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A2B8]", "fstp dword ptr [{rc} + 0x31A2B8]");
    reference!(ref_58(m, rc) "fld dword ptr [{m} + 0x35B454]", "fmul dword ptr [{m} + 0x35B68C]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A2BC]", "fstp dword ptr [{rc} + 0x31A2BC]");
    reference!(ref_59(m, rc) "fld dword ptr [{m} + 0x35B454]", "fmul dword ptr [{m} + 0x35B68C]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A2C0]", "fstp dword ptr [{rc} + 0x31A2C0]");
    reference!(ref_60(m, rc) "fld dword ptr [{m} + 0x35B454]", "fmul dword ptr [{m} + 0x35B68C]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A2C4]", "fstp dword ptr [{rc} + 0x31A2C4]");
    reference!(ref_61(m, rc) "fld dword ptr [{m} + 0x35B454]", "fmul dword ptr [{m} + 0x35B68C]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A2C8]", "fstp dword ptr [{rc} + 0x31A2C8]");
    reference!(ref_62(m, rc) "fld dword ptr [{m} + 0x35B454]", "fmul dword ptr [{m} + 0x35B690]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A2A4]", "fstp dword ptr [{rc} + 0x31A2A4]");
    reference!(ref_63(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6AC]", "fadd dword ptr [{rc} + 0x31A2E0]", "fstp dword ptr [{rc} + 0x31A2E0]");
    reference!(ref_64(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{rc} + 0x31C708]", "fstp dword ptr [{m} + 0x35B3C8]");
    reference!(ref_65(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6D8]", "fadd dword ptr [{rc} + 0x31A2EC]", "fstp dword ptr [{rc} + 0x31A2EC]");
    reference!(ref_66(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6D8]", "fadd dword ptr [{rc} + 0x31A2F0]", "fstp dword ptr [{rc} + 0x31A2F0]");
    reference!(ref_67(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6D8]", "fadd dword ptr [{rc} + 0x31A2F4]", "fstp dword ptr [{rc} + 0x31A2F4]");
    reference!(ref_68(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6D8]", "fadd dword ptr [{rc} + 0x31A2F8]", "fstp dword ptr [{rc} + 0x31A2F8]");
    reference!(ref_69(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6DC]", "fadd dword ptr [{rc} + 0x31A2FC]", "fstp dword ptr [{rc} + 0x31A2FC]");
    reference!(ref_70(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6DC]", "fadd dword ptr [{rc} + 0x31A2FC]", "fstp dword ptr [{rc} + 0x31A2FC]");
    reference!(ref_71(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6DC]", "fadd dword ptr [{rc} + 0x31A300]", "fstp dword ptr [{rc} + 0x31A300]");
    reference!(ref_72(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6DC]", "fadd dword ptr [{rc} + 0x31A300]", "fstp dword ptr [{rc} + 0x31A300]");
    reference!(ref_73(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6D4]", "fadd dword ptr [{rc} + 0x31A308]", "fstp dword ptr [{rc} + 0x31A308]");
    reference!(ref_74(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6D0]", "fadd dword ptr [{rc} + 0x31A2E8]", "fstp dword ptr [{rc} + 0x31A2E8]");
    reference!(ref_75(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6BC]", "fadd dword ptr [{rc} + 0x31A298]", "fstp dword ptr [{rc} + 0x31A298]");
    reference!(ref_76(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6C0]", "fadd dword ptr [{rc} + 0x31A2A0]", "fstp dword ptr [{rc} + 0x31A2A0]");
    reference!(ref_77(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6C4]", "fadd dword ptr [{rc} + 0x31A2A4]", "fstp dword ptr [{rc} + 0x31A2A4]");
    reference!(ref_78(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6C8]", "fadd dword ptr [{rc} + 0x31A2A8]", "fstp dword ptr [{rc} + 0x31A2A8]");
    reference!(ref_79(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6C8]", "fadd dword ptr [{rc} + 0x31A2AC]", "fstp dword ptr [{rc} + 0x31A2AC]");
    reference!(ref_80(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6C8]", "fadd dword ptr [{rc} + 0x31A2B0]", "fstp dword ptr [{rc} + 0x31A2B0]");
    reference!(ref_81(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6C8]", "fadd dword ptr [{rc} + 0x31A2B4]", "fstp dword ptr [{rc} + 0x31A2B4]");
    reference!(ref_82(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6C8]", "fadd dword ptr [{rc} + 0x31A2B8]", "fstp dword ptr [{rc} + 0x31A2B8]");
    reference!(ref_83(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6C8]", "fadd dword ptr [{rc} + 0x31A2BC]", "fstp dword ptr [{rc} + 0x31A2BC]");
    reference!(ref_84(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6C8]", "fadd dword ptr [{rc} + 0x31A2C0]", "fstp dword ptr [{rc} + 0x31A2C0]");
    reference!(ref_85(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6C8]", "fadd dword ptr [{rc} + 0x31A2C4]", "fstp dword ptr [{rc} + 0x31A2C4]");
    reference!(ref_86(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6C8]", "fadd dword ptr [{rc} + 0x31A2C8]", "fstp dword ptr [{rc} + 0x31A2C8]");
    reference!(ref_87(m, v) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6CC]", "fadd dword ptr [{v}]", "fstp dword ptr [{v}]");
    reference!(ref_88(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6E8]", "fadd dword ptr [{rc} + 0x31A2D0]", "fstp dword ptr [{rc} + 0x31A2D0]");
    reference!(ref_89(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6E8]", "fadd dword ptr [{rc} + 0x31A2D4]", "fstp dword ptr [{rc} + 0x31A2D4]");
    reference!(ref_90(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6E8]", "fadd dword ptr [{rc} + 0x31A2D8]", "fstp dword ptr [{rc} + 0x31A2D8]");
    reference!(ref_91(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6E8]", "fadd dword ptr [{rc} + 0x31A2DC]", "fstp dword ptr [{rc} + 0x31A2DC]");
    reference!(ref_92(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6F0]", "fadd dword ptr [{rc} + 0x31A30C]", "fstp dword ptr [{rc} + 0x31A30C]");
    reference!(ref_93(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6F0]", "fadd dword ptr [{rc} + 0x31A310]", "fstp dword ptr [{rc} + 0x31A310]");
    reference!(ref_94(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6F0]", "fadd dword ptr [{rc} + 0x31A314]", "fstp dword ptr [{rc} + 0x31A314]");
    reference!(ref_95(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6F0]", "fadd dword ptr [{rc} + 0x31A318]", "fstp dword ptr [{rc} + 0x31A318]");
    reference!(ref_96(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6E4]", "fadd dword ptr [{rc} + 0x31A29C]", "fstp dword ptr [{rc} + 0x31A29C]");
    reference!(ref_97(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6E4]", "fadd dword ptr [{rc} + 0x31A29C]", "fstp dword ptr [{rc} + 0x31A29C]");
    reference!(ref_98(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6E0]", "fadd dword ptr [{rc} + 0x31A290]", "fstp dword ptr [{rc} + 0x31A290]");
    reference!(ref_99(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6E0]", "fadd dword ptr [{rc} + 0x31A290]", "fstp dword ptr [{rc} + 0x31A290]");
    reference!(ref_100(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6EC]", "fadd dword ptr [{rc} + 0x31A320]", "fstp dword ptr [{rc} + 0x31A320]");
    reference!(ref_101(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6EC]", "fadd dword ptr [{rc} + 0x31A320]", "fstp dword ptr [{rc} + 0x31A320]");
    reference!(ref_102(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6EC]", "fadd dword ptr [{rc} + 0x31A324]", "fstp dword ptr [{rc} + 0x31A324]");
    reference!(ref_103(m, rc) "fld dword ptr [{m} + 0x35B3C8]", "fmul dword ptr [{m} + 0x35B6EC]", "fadd dword ptr [{rc} + 0x31A324]", "fstp dword ptr [{rc} + 0x31A324]");
    reference!(ref_104(m, rc) "fld dword ptr [{m} + 0x35B3CC]", "fmul dword ptr [{m} + 0x35B6B0]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A30C]", "fstp dword ptr [{rc} + 0x31A30C]");
    reference!(ref_105(m, rc) "fld dword ptr [{m} + 0x35B3D0]", "fmul dword ptr [{m} + 0x35B6B0]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A310]", "fstp dword ptr [{rc} + 0x31A310]");
    reference!(ref_106(m, rc) "fld dword ptr [{m} + 0x35B3D4]", "fmul dword ptr [{m} + 0x35B6B0]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A314]", "fstp dword ptr [{rc} + 0x31A314]");
    reference!(ref_107(m, rc) "fld dword ptr [{m} + 0x35B3D8]", "fmul dword ptr [{m} + 0x35B6B0]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A318]", "fstp dword ptr [{rc} + 0x31A318]");
    reference!(ref_108(m, rc) "fld dword ptr [{m} + 0x35B3DC]", "fmul dword ptr [{m} + 0x35B6B4]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A294]", "fstp dword ptr [{rc} + 0x31A294]");
    reference!(ref_109(m, rc) "fld dword ptr [{m} + 0x35B3DC]", "fmul dword ptr [{m} + 0x35B6B8]", "fmul dword ptr [{m} + 0x35B370]", "fadd dword ptr [{rc} + 0x31A308]", "fstp dword ptr [{rc} + 0x31A308]");
    reference!(ref_110(rc, m) "fld dword ptr [{rc} + 0x31A2EC]", "fadd dword ptr [{rc} + 0x31A2F0]", "fadd dword ptr [{rc} + 0x31A2F4]", "fadd dword ptr [{rc} + 0x31A2F8]", "fadd dword ptr [{rc} + 0x31A300]", "fadd dword ptr [{rc} + 0x31A2FC]", "fdiv dword ptr [{m} + 0x35B344]", "fstp dword ptr [{rc} + 0x31A304]", "fld dword ptr [{rc} + 0x31A320]", "fadd dword ptr [{rc} + 0x31A324]", "fmul dword ptr [{m} + 0x35B360]", "fstp dword ptr [{rc} + 0x31A328]", "fld dword ptr [{rc} + 0x31A32C]", "fadd dword ptr [{rc} + 0x31A330]", "fmul dword ptr [{m} + 0x35B360]", "fstp dword ptr [{m} + 0x35B440]", "fld dword ptr [{rc} + 0x31A334]", "fadd dword ptr [{rc} + 0x31A338]", "fmul dword ptr [{m} + 0x35B360]", "fstp dword ptr [{m} + 0x35B444]", "fld dword ptr [{m} + 0x35B440]", "fadd dword ptr [{m} + 0x35B444]", "fmul dword ptr [{m} + 0x35B360]", "fstp dword ptr [{rc} + 0x31A33C]", "fld dword ptr [{rc} + 0x31A2A8]", "fadd dword ptr [{rc} + 0x31A2B0]", "fadd dword ptr [{rc} + 0x31A2B4]", "fadd dword ptr [{rc} + 0x31A2B8]", "fadd dword ptr [{rc} + 0x31A2BC]", "fadd dword ptr [{rc} + 0x31A2C0]", "fadd dword ptr [{rc} + 0x31A2C4]", "fadd dword ptr [{rc} + 0x31A2C8]", "fmul dword ptr [{m} + 0x35B360]", "fmul dword ptr [{m} + 0x35B360]", "fmul dword ptr [{m} + 0x35B360]", "fstp dword ptr [{rc} + 0x31A2CC]", "fld dword ptr [{rc} + 0x31A2D0]", "fadd dword ptr [{rc} + 0x31A2D4]", "fadd dword ptr [{rc} + 0x31A2D8]", "fadd dword ptr [{rc} + 0x31A2DC]", "fadd dword ptr [{rc} + 0x31A2E0]", "fadd dword ptr [{rc} + 0x31A2E0]", "fadd dword ptr [{rc} + 0x31A2E0]", "fadd dword ptr [{rc} + 0x31A2E0]", "fadd dword ptr [{rc} + 0x31A2E0]", "fadd dword ptr [{rc} + 0x31A2E0]", "fmul dword ptr [{m} + 0x35B368]", "fstp dword ptr [{rc} + 0x31A2E4]", "fld dword ptr [{rc} + 0x31A30C]", "fadd dword ptr [{rc} + 0x31A310]", "fadd dword ptr [{rc} + 0x31A314]", "fadd dword ptr [{rc} + 0x31A318]", "fmul dword ptr [{m} + 0x35B364]", "fstp dword ptr [{rc} + 0x31A31C]", "fld dword ptr [{rc} + 0x31A420]", "fadd dword ptr [{rc} + 0x31A424]", "fadd dword ptr [{rc} + 0x31A428]", "fadd dword ptr [{rc} + 0x31A42C]", "fmul dword ptr [{m} + 0x35B364]", "fstp dword ptr [{rc} + 0x31A430]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "damage_gauges line 45",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B5D0, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A290, In::F)],
                        writes: &[(0x31A290, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).gauge_rates[0],
                        &dm(a[0]).flt_75b38c,
                        &CAR.engine_damage,
                        &CAR.engine_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 55",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B5D4, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A294, In::F)],
                        writes: &[(0x31A294, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).gauge_rates[1],
                        &dm(a[0]).flt_75b38c,
                        &CAR.exhaust_damage,
                        &CAR.exhaust_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 65",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B5D8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A29C, In::F)],
                        writes: &[(0x31A29C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).gauge_rates[2],
                        &dm(a[0]).flt_75b38c,
                        &CAR.cooling_damage,
                        &CAR.cooling_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 75",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B5DC, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2A0, In::F)],
                        writes: &[(0x31A2A0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_3(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).gauge_rates[3],
                        &dm(a[0]).flt_75b38c,
                        &CAR.clutch_damage,
                        &CAR.clutch_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 85",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B5E0, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2A4, In::F)],
                        writes: &[(0x31A2A4, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_4(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).gauge_rates[4],
                        &dm(a[0]).flt_75b38c,
                        &CAR.differential_damage,
                        &CAR.differential_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 95",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B5E4, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2E8, In::F)],
                        writes: &[(0x31A2E8, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_5(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).gauge_rates[5],
                        &dm(a[0]).flt_75b38c,
                        &CAR.electrics_damage,
                        &CAR.electrics_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 105",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B5E8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2EC, In::F)],
                        writes: &[(0x31A2EC, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_6(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).gauge_rates[6],
                        &dm(a[0]).flt_75b38c,
                        &CAR.light_damage[0],
                        &CAR.light_damage[0],
                    );
                },
            },
            Block {
                name: "damage_gauges line 115",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B5E8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2F0, In::F)],
                        writes: &[(0x31A2F0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_7(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).gauge_rates[6],
                        &dm(a[0]).flt_75b38c,
                        &CAR.light_damage[1],
                        &CAR.light_damage[1],
                    );
                },
            },
            Block {
                name: "damage_gauges line 125",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B5E8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2F4, In::F)],
                        writes: &[(0x31A2F4, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_8(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).gauge_rates[6],
                        &dm(a[0]).flt_75b38c,
                        &CAR.light_damage[2],
                        &CAR.light_damage[2],
                    );
                },
            },
            Block {
                name: "damage_gauges line 135",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B5E8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2F8, In::F)],
                        writes: &[(0x31A2F8, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_9(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).gauge_rates[6],
                        &dm(a[0]).flt_75b38c,
                        &CAR.light_damage[3],
                        &CAR.light_damage[3],
                    );
                },
            },
            Block {
                name: "damage_gauges line 145",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B5E8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2FC, In::F)],
                        writes: &[(0x31A2FC, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_10(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).gauge_rates[6],
                        &dm(a[0]).flt_75b38c,
                        &CAR.light_damage[4],
                        &CAR.light_damage[4],
                    );
                },
            },
            Block {
                name: "damage_gauges line 155",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B5E8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A300, In::F)],
                        writes: &[(0x31A300, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_11(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).gauge_rates[6],
                        &dm(a[0]).flt_75b38c,
                        &CAR.light_damage[5],
                        &CAR.light_damage[5],
                    );
                },
            },
            Block {
                name: "damage_gauges line 165",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B5EC, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A308, In::F)],
                        writes: &[(0x31A308, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_12(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).gauge_rates[7],
                        &dm(a[0]).flt_75b38c,
                        &CAR.steering_damage,
                        &CAR.steering_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 175",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B5F0, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A30C, In::F)],
                        writes: &[(0x31A30C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_13(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).gauge_rates[8],
                        &dm(a[0]).flt_75b38c,
                        &CAR.suspension_damage[0],
                        &CAR.suspension_damage[0],
                    );
                },
            },
            Block {
                name: "damage_gauges line 185",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B5F0, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A310, In::F)],
                        writes: &[(0x31A310, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_14(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).gauge_rates[8],
                        &dm(a[0]).flt_75b38c,
                        &CAR.suspension_damage[1],
                        &CAR.suspension_damage[1],
                    );
                },
            },
            Block {
                name: "damage_gauges line 195",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B5F0, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A314, In::F)],
                        writes: &[(0x31A314, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_15(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).gauge_rates[8],
                        &dm(a[0]).flt_75b38c,
                        &CAR.suspension_damage[2],
                        &CAR.suspension_damage[2],
                    );
                },
            },
            Block {
                name: "damage_gauges line 205",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B5F0, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A318, In::F)],
                        writes: &[(0x31A318, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_16(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).gauge_rates[8],
                        &dm(a[0]).flt_75b38c,
                        &CAR.suspension_damage[3],
                        &CAR.suspension_damage[3],
                    );
                },
            },
            Block {
                name: "damage_gauges line 215",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (0x35B38C, In::F),
                            (0x35B398, In::F),
                            (0x35B3C0, In::F),
                            (0x35B5A4, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A298, In::F)],
                        writes: &[(0x31A298, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_17(a[0], a[1]) },
                plain: |a| {
                    x87_2(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[11],
                        &dm(a[0]).flt_75b3c0,
                        &dm(a[0]).flt_75b398,
                        &CAR.turbo_damage,
                        &CAR.turbo_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 229",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B398, In::F), (0x35B5A8, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[(0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_18(a[0], a[1], a[2]) },
                plain: |a| x87_3(dm(a[0]), a[1], a[2]),
            },
            Block {
                name: "damage_gauges line 239",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (0x35B38C, In::F),
                            (0x35B3B0, In::F),
                            (0x35B400, In::F),
                            (0x35B5F4, In::F),
                        ],
                        writes: &[(0x35B43C, Out::F)],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A320, In::F)],
                        writes: &[(0x31A320, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_19(a[0], a[1]) },
                plain: |a| x87_4(dm(a[0])),
            },
            Block {
                name: "damage_gauges line 254",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (0x35B38C, In::F),
                            (0x35B3B0, In::F),
                            (0x35B43C, In::F),
                            (0x35B5F4, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A324, In::F)],
                        writes: &[(0x31A324, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_20(a[0], a[1]) },
                plain: |a| {
                    x87_2(
                        dm(a[0]),
                        &dm(a[0]).gauge_rates[9],
                        &dm(a[0]).flt_75b43c,
                        &dm(a[0]).flt_75b3b0,
                        &CAR.brake_wear_b,
                        &CAR.brake_wear_b,
                    );
                },
            },
            Block {
                name: "damage_gauges line 266",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B3B4, In::F), (0x35B5F8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2E8, In::F)],
                        writes: &[(0x31A2E8, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_21(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).gauge_rates[10],
                        &dm(a[0]).flt_75b3b4,
                        &dm(a[0]).flt_75b38c,
                        &CAR.electrics_damage,
                        &CAR.electrics_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 277",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B3B4, In::F), (0x35B5FC, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2EC, In::F)],
                        writes: &[(0x31A2EC, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_22(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).gauge_rates[11],
                        &dm(a[0]).flt_75b3b4,
                        &dm(a[0]).flt_75b38c,
                        &CAR.light_damage[0],
                        &CAR.light_damage[0],
                    );
                },
            },
            Block {
                name: "damage_gauges line 288",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B3B4, In::F), (0x35B5FC, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2F0, In::F)],
                        writes: &[(0x31A2F0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_23(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).gauge_rates[11],
                        &dm(a[0]).flt_75b3b4,
                        &dm(a[0]).flt_75b38c,
                        &CAR.light_damage[1],
                        &CAR.light_damage[1],
                    );
                },
            },
            Block {
                name: "damage_gauges line 299",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B3B4, In::F), (0x35B5FC, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2F4, In::F)],
                        writes: &[(0x31A2F4, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_24(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).gauge_rates[11],
                        &dm(a[0]).flt_75b3b4,
                        &dm(a[0]).flt_75b38c,
                        &CAR.light_damage[2],
                        &CAR.light_damage[2],
                    );
                },
            },
            Block {
                name: "damage_gauges line 310",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B3B4, In::F), (0x35B5FC, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2F8, In::F)],
                        writes: &[(0x31A2F8, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_25(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).gauge_rates[11],
                        &dm(a[0]).flt_75b3b4,
                        &dm(a[0]).flt_75b38c,
                        &CAR.light_damage[3],
                        &CAR.light_damage[3],
                    );
                },
            },
            Block {
                name: "damage_gauges line 321",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B3B4, In::F), (0x35B5FC, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2FC, In::F)],
                        writes: &[(0x31A2FC, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_26(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).gauge_rates[11],
                        &dm(a[0]).flt_75b3b4,
                        &dm(a[0]).flt_75b38c,
                        &CAR.light_damage[4],
                        &CAR.light_damage[4],
                    );
                },
            },
            Block {
                name: "damage_gauges line 332",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B38C, In::F), (0x35B3B4, In::F), (0x35B5FC, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A300, In::F)],
                        writes: &[(0x31A300, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_27(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).gauge_rates[11],
                        &dm(a[0]).flt_75b3b4,
                        &dm(a[0]).flt_75b38c,
                        &CAR.light_damage[5],
                        &CAR.light_damage[5],
                    );
                },
            },
            Block {
                name: "damage_gauges line 343",
                operands: vec![Operand::Buffer {
                    reads: &[
                        (0x35B34C, In::F),
                        (0x35B380, In::F),
                        (0x35B39C, In::F),
                        (0x35B3F4, In::F),
                        (0x35B484, In::F),
                    ],
                    writes: &[(0x35B488, Out::F), (0x35B48C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_28(a[0]) },
                plain: |a| x87_6(dm(a[0])),
            },
            Block {
                name: "damage_gauges line 355",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B488, In::F), (0x35B48C, In::F)],
                        writes: &[(0x35B488, Out::F), (0x35B48C, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_29(a[0], a[1], a[2]) },
                plain: |a| x87_7(dm(a[0]), a[1], a[2]),
            },
            Block {
                name: "damage_gauges line 365",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B46C, In::F), (0x35B488, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A32C, In::F)],
                        writes: &[(0x31A32C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_30(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b46c,
                        &dm(a[0]).flt_75b488,
                        &dm(a[0]).flt_75b370,
                        &CAR.part_hit_damage[0],
                        &CAR.part_hit_damage[0],
                    );
                },
            },
            Block {
                name: "damage_gauges line 376",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B46C, In::F), (0x35B488, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A330, In::F)],
                        writes: &[(0x31A330, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_31(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b46c,
                        &dm(a[0]).flt_75b488,
                        &dm(a[0]).flt_75b370,
                        &CAR.part_hit_damage[1],
                        &CAR.part_hit_damage[1],
                    );
                },
            },
            Block {
                name: "damage_gauges line 387",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B470, In::F), (0x35B48C, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A334, In::F)],
                        writes: &[(0x31A334, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_32(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b470,
                        &dm(a[0]).flt_75b48c,
                        &dm(a[0]).flt_75b370,
                        &CAR.part_hit_damage[2],
                        &CAR.part_hit_damage[2],
                    );
                },
            },
            Block {
                name: "damage_gauges line 398",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B470, In::F), (0x35B48C, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A338, In::F)],
                        writes: &[(0x31A338, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_33(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b470,
                        &dm(a[0]).flt_75b48c,
                        &dm(a[0]).flt_75b370,
                        &CAR.part_hit_damage[3],
                        &CAR.part_hit_damage[3],
                    );
                },
            },
            Block {
                name: "damage_gauges line 409",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B45C, In::F), (0x35B604, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A290, In::F)],
                        writes: &[(0x31A290, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_34(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b45c,
                        &dm(a[0]).gauge_rates[13],
                        &dm(a[0]).flt_75b370,
                        &CAR.engine_damage,
                        &CAR.engine_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 420",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B45C, In::F), (0x35B600, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A29C, In::F)],
                        writes: &[(0x31A29C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_35(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b45c,
                        &dm(a[0]).gauge_rates[12],
                        &dm(a[0]).flt_75b370,
                        &CAR.cooling_damage,
                        &CAR.cooling_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 432",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[(0x31A28C, In::F), (0x31A290, In::F)],
                        writes: &[(0x31A290, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B608, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_36(a[0], a[1]) },
                plain: |a| x87_8(dm(a[1])),
            },
            Block {
                name: "damage_gauges line 443",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B464, In::F), (0x35B60C, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A29C, In::F)],
                        writes: &[(0x31A29C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_37(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b464,
                        &dm(a[0]).gauge_rates[15],
                        &dm(a[0]).flt_75b370,
                        &CAR.cooling_damage,
                        &CAR.cooling_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 456",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[(0x31A2A0, In::F)],
                        writes: &[(0x31A2A0, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0x35B610, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_38(a[0], a[1]) },
                plain: |a| x87_9(&dm(a[1]).gauge_rates[16]),
            },
            Block {
                name: "damage_gauges line 465",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[(0x31A2A0, In::F)],
                        writes: &[(0x31A2A0, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0x35B614, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_39(a[0], a[1]) },
                plain: |a| x87_9(&dm(a[1]).gauge_rates[17]),
            },
            Block {
                name: "damage_gauges line 476",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[(0, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0x35B618, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_40(a[0], a[1]) },
                plain: |a| x87_10(a[0], &dm(a[1]).gauge_rates[18]),
            },
            Block {
                name: "damage_gauges line 486",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[(0, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0x35B61C, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_41(a[0], a[1]) },
                plain: |a| x87_10(a[0], &dm(a[1]).gauge_rates[19]),
            },
            Block {
                name: "damage_gauges line 493",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B450, In::F), (0x35B670, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2D0, In::F)],
                        writes: &[(0x31A2D0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_42(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b450,
                        &dm(a[0]).flt_75b670,
                        &dm(a[0]).flt_75b370,
                        &CAR.wheel_hit_damage[0],
                        &CAR.wheel_hit_damage[0],
                    );
                },
            },
            Block {
                name: "damage_gauges line 504",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B450, In::F), (0x35B670, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2D4, In::F)],
                        writes: &[(0x31A2D4, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_43(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b450,
                        &dm(a[0]).flt_75b670,
                        &dm(a[0]).flt_75b370,
                        &CAR.wheel_hit_damage[1],
                        &CAR.wheel_hit_damage[1],
                    );
                },
            },
            Block {
                name: "damage_gauges line 515",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B450, In::F), (0x35B670, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2D8, In::F)],
                        writes: &[(0x31A2D8, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_44(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b450,
                        &dm(a[0]).flt_75b670,
                        &dm(a[0]).flt_75b370,
                        &CAR.wheel_hit_damage[2],
                        &CAR.wheel_hit_damage[2],
                    );
                },
            },
            Block {
                name: "damage_gauges line 526",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B450, In::F), (0x35B670, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2DC, In::F)],
                        writes: &[(0x31A2DC, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_45(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b450,
                        &dm(a[0]).flt_75b670,
                        &dm(a[0]).flt_75b370,
                        &CAR.wheel_hit_damage[3],
                        &CAR.wheel_hit_damage[3],
                    );
                },
            },
            Block {
                name: "damage_gauges line 537",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B458, In::F), (0x35B678, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A30C, In::F)],
                        writes: &[(0x31A30C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_46(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b458,
                        &dm(a[0]).figures[0],
                        &dm(a[0]).flt_75b370,
                        &CAR.suspension_damage[0],
                        &CAR.suspension_damage[0],
                    );
                },
            },
            Block {
                name: "damage_gauges line 548",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B458, In::F), (0x35B678, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A310, In::F)],
                        writes: &[(0x31A310, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_47(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b458,
                        &dm(a[0]).figures[0],
                        &dm(a[0]).flt_75b370,
                        &CAR.suspension_damage[1],
                        &CAR.suspension_damage[1],
                    );
                },
            },
            Block {
                name: "damage_gauges line 559",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B458, In::F), (0x35B678, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A314, In::F)],
                        writes: &[(0x31A314, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_48(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b458,
                        &dm(a[0]).figures[0],
                        &dm(a[0]).flt_75b370,
                        &CAR.suspension_damage[2],
                        &CAR.suspension_damage[2],
                    );
                },
            },
            Block {
                name: "damage_gauges line 570",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B458, In::F), (0x35B678, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A318, In::F)],
                        writes: &[(0x31A318, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_49(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b458,
                        &dm(a[0]).figures[0],
                        &dm(a[0]).flt_75b370,
                        &CAR.suspension_damage[3],
                        &CAR.suspension_damage[3],
                    );
                },
            },
            Block {
                name: "damage_gauges line 581",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B454, In::F), (0x35B67C, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2E0, In::F)],
                        writes: &[(0x31A2E0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_50(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b454,
                        &dm(a[0]).figures[1],
                        &dm(a[0]).flt_75b370,
                        &CAR.body_hit_damage,
                        &CAR.body_hit_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 592",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B454, In::F), (0x35B680, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A308, In::F)],
                        writes: &[(0x31A308, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_51(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b454,
                        &dm(a[0]).figures[2],
                        &dm(a[0]).flt_75b370,
                        &CAR.steering_damage,
                        &CAR.steering_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 603",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B454, In::F), (0x35B684, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A290, In::F)],
                        writes: &[(0x31A290, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_52(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b454,
                        &dm(a[0]).figures[3],
                        &dm(a[0]).flt_75b370,
                        &CAR.engine_damage,
                        &CAR.engine_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 614",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B454, In::F), (0x35B688, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A294, In::F)],
                        writes: &[(0x31A294, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_53(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b454,
                        &dm(a[0]).figures[4],
                        &dm(a[0]).flt_75b370,
                        &CAR.exhaust_damage,
                        &CAR.exhaust_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 625",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B454, In::F), (0x35B68C, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2A8, In::F)],
                        writes: &[(0x31A2A8, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_54(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b454,
                        &dm(a[0]).figures[5],
                        &dm(a[0]).flt_75b370,
                        &CAR.gear_wear[0],
                        &CAR.gear_wear[0],
                    );
                },
            },
            Block {
                name: "damage_gauges line 636",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B454, In::F), (0x35B68C, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2B0, In::F)],
                        writes: &[(0x31A2B0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_55(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b454,
                        &dm(a[0]).figures[5],
                        &dm(a[0]).flt_75b370,
                        &CAR.gear_wear[2],
                        &CAR.gear_wear[2],
                    );
                },
            },
            Block {
                name: "damage_gauges line 647",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B454, In::F), (0x35B68C, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2B4, In::F)],
                        writes: &[(0x31A2B4, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_56(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b454,
                        &dm(a[0]).figures[5],
                        &dm(a[0]).flt_75b370,
                        &CAR.gear_wear[3],
                        &CAR.gear_wear[3],
                    );
                },
            },
            Block {
                name: "damage_gauges line 658",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B454, In::F), (0x35B68C, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2B8, In::F)],
                        writes: &[(0x31A2B8, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_57(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b454,
                        &dm(a[0]).figures[5],
                        &dm(a[0]).flt_75b370,
                        &CAR.gear_wear[4],
                        &CAR.gear_wear[4],
                    );
                },
            },
            Block {
                name: "damage_gauges line 669",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B454, In::F), (0x35B68C, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2BC, In::F)],
                        writes: &[(0x31A2BC, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_58(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b454,
                        &dm(a[0]).figures[5],
                        &dm(a[0]).flt_75b370,
                        &CAR.gear_wear[5],
                        &CAR.gear_wear[5],
                    );
                },
            },
            Block {
                name: "damage_gauges line 680",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B454, In::F), (0x35B68C, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2C0, In::F)],
                        writes: &[(0x31A2C0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_59(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b454,
                        &dm(a[0]).figures[5],
                        &dm(a[0]).flt_75b370,
                        &CAR.gear_wear[6],
                        &CAR.gear_wear[6],
                    );
                },
            },
            Block {
                name: "damage_gauges line 691",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B454, In::F), (0x35B68C, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2C4, In::F)],
                        writes: &[(0x31A2C4, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_60(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b454,
                        &dm(a[0]).figures[5],
                        &dm(a[0]).flt_75b370,
                        &CAR.gear_wear[7],
                        &CAR.gear_wear[7],
                    );
                },
            },
            Block {
                name: "damage_gauges line 702",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B454, In::F), (0x35B68C, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2C8, In::F)],
                        writes: &[(0x31A2C8, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_61(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b454,
                        &dm(a[0]).figures[5],
                        &dm(a[0]).flt_75b370,
                        &CAR.gear_wear[8],
                        &CAR.gear_wear[8],
                    );
                },
            },
            Block {
                name: "damage_gauges line 713",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B454, In::F), (0x35B690, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2A4, In::F)],
                        writes: &[(0x31A2A4, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_62(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b454,
                        &dm(a[0]).figures[6],
                        &dm(a[0]).flt_75b370,
                        &CAR.differential_damage,
                        &CAR.differential_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 725",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6AC, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2E0, In::F)],
                        writes: &[(0x31A2E0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_63(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[13],
                        &CAR.body_hit_damage,
                        &CAR.body_hit_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 737",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F)],
                        writes: &[(0x35B3C8, Out::F)],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31C708, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_64(a[0], a[1]) },
                plain: |a| x87_11(dm(a[0])),
            },
            Block {
                name: "damage_gauges line 745",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6D8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2EC, In::F)],
                        writes: &[(0x31A2EC, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_65(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[24],
                        &CAR.light_damage[0],
                        &CAR.light_damage[0],
                    );
                },
            },
            Block {
                name: "damage_gauges line 757",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6D8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2F0, In::F)],
                        writes: &[(0x31A2F0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_66(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[24],
                        &CAR.light_damage[1],
                        &CAR.light_damage[1],
                    );
                },
            },
            Block {
                name: "damage_gauges line 769",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6D8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2F4, In::F)],
                        writes: &[(0x31A2F4, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_67(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[24],
                        &CAR.light_damage[2],
                        &CAR.light_damage[2],
                    );
                },
            },
            Block {
                name: "damage_gauges line 781",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6D8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2F8, In::F)],
                        writes: &[(0x31A2F8, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_68(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[24],
                        &CAR.light_damage[3],
                        &CAR.light_damage[3],
                    );
                },
            },
            Block {
                name: "damage_gauges line 793",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6DC, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2FC, In::F)],
                        writes: &[(0x31A2FC, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_69(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[25],
                        &CAR.light_damage[4],
                        &CAR.light_damage[4],
                    );
                },
            },
            Block {
                name: "damage_gauges line 805",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6DC, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2FC, In::F)],
                        writes: &[(0x31A2FC, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_70(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[25],
                        &CAR.light_damage[4],
                        &CAR.light_damage[4],
                    );
                },
            },
            Block {
                name: "damage_gauges line 817",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6DC, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A300, In::F)],
                        writes: &[(0x31A300, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_71(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[25],
                        &CAR.light_damage[5],
                        &CAR.light_damage[5],
                    );
                },
            },
            Block {
                name: "damage_gauges line 829",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6DC, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A300, In::F)],
                        writes: &[(0x31A300, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_72(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[25],
                        &CAR.light_damage[5],
                        &CAR.light_damage[5],
                    );
                },
            },
            Block {
                name: "damage_gauges line 840",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6D4, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A308, In::F)],
                        writes: &[(0x31A308, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_73(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[23],
                        &CAR.steering_damage,
                        &CAR.steering_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 850",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6D0, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2E8, In::F)],
                        writes: &[(0x31A2E8, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_74(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[22],
                        &CAR.electrics_damage,
                        &CAR.electrics_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 860",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6BC, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A298, In::F)],
                        writes: &[(0x31A298, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_75(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[17],
                        &CAR.turbo_damage,
                        &CAR.turbo_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 870",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6C0, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2A0, In::F)],
                        writes: &[(0x31A2A0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_76(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[18],
                        &CAR.clutch_damage,
                        &CAR.clutch_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 880",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6C4, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2A4, In::F)],
                        writes: &[(0x31A2A4, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_77(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[19],
                        &CAR.differential_damage,
                        &CAR.differential_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 890",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6C8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2A8, In::F)],
                        writes: &[(0x31A2A8, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_78(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[20],
                        &CAR.gear_wear[0],
                        &CAR.gear_wear[0],
                    );
                },
            },
            Block {
                name: "damage_gauges line 900",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6C8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2AC, In::F)],
                        writes: &[(0x31A2AC, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_79(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[20],
                        &CAR.gear_wear[1],
                        &CAR.gear_wear[1],
                    );
                },
            },
            Block {
                name: "damage_gauges line 910",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6C8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2B0, In::F)],
                        writes: &[(0x31A2B0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_80(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[20],
                        &CAR.gear_wear[2],
                        &CAR.gear_wear[2],
                    );
                },
            },
            Block {
                name: "damage_gauges line 920",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6C8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2B4, In::F)],
                        writes: &[(0x31A2B4, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_81(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[20],
                        &CAR.gear_wear[3],
                        &CAR.gear_wear[3],
                    );
                },
            },
            Block {
                name: "damage_gauges line 930",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6C8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2B8, In::F)],
                        writes: &[(0x31A2B8, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_82(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[20],
                        &CAR.gear_wear[4],
                        &CAR.gear_wear[4],
                    );
                },
            },
            Block {
                name: "damage_gauges line 940",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6C8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2BC, In::F)],
                        writes: &[(0x31A2BC, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_83(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[20],
                        &CAR.gear_wear[5],
                        &CAR.gear_wear[5],
                    );
                },
            },
            Block {
                name: "damage_gauges line 950",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6C8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2C0, In::F)],
                        writes: &[(0x31A2C0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_84(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[20],
                        &CAR.gear_wear[6],
                        &CAR.gear_wear[6],
                    );
                },
            },
            Block {
                name: "damage_gauges line 960",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6C8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2C4, In::F)],
                        writes: &[(0x31A2C4, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_85(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[20],
                        &CAR.gear_wear[7],
                        &CAR.gear_wear[7],
                    );
                },
            },
            Block {
                name: "damage_gauges line 970",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6C8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2C8, In::F)],
                        writes: &[(0x31A2C8, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_86(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[20],
                        &CAR.gear_wear[8],
                        &CAR.gear_wear[8],
                    );
                },
            },
            Block {
                name: "damage_gauges line 982",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6CC, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[(0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_87(a[0], a[1]) },
                plain: |a| x87_12(dm(a[0]), a[1]),
            },
            Block {
                name: "damage_gauges line 991",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6E8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2D0, In::F)],
                        writes: &[(0x31A2D0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_88(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[28],
                        &CAR.wheel_hit_damage[0],
                        &CAR.wheel_hit_damage[0],
                    );
                },
            },
            Block {
                name: "damage_gauges line 1003",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6E8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2D4, In::F)],
                        writes: &[(0x31A2D4, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_89(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[28],
                        &CAR.wheel_hit_damage[1],
                        &CAR.wheel_hit_damage[1],
                    );
                },
            },
            Block {
                name: "damage_gauges line 1015",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6E8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2D8, In::F)],
                        writes: &[(0x31A2D8, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_90(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[28],
                        &CAR.wheel_hit_damage[2],
                        &CAR.wheel_hit_damage[2],
                    );
                },
            },
            Block {
                name: "damage_gauges line 1027",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6E8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A2DC, In::F)],
                        writes: &[(0x31A2DC, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_91(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[28],
                        &CAR.wheel_hit_damage[3],
                        &CAR.wheel_hit_damage[3],
                    );
                },
            },
            Block {
                name: "damage_gauges line 1039",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6F0, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A30C, In::F)],
                        writes: &[(0x31A30C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_92(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[30],
                        &CAR.suspension_damage[0],
                        &CAR.suspension_damage[0],
                    );
                },
            },
            Block {
                name: "damage_gauges line 1051",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6F0, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A310, In::F)],
                        writes: &[(0x31A310, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_93(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[30],
                        &CAR.suspension_damage[1],
                        &CAR.suspension_damage[1],
                    );
                },
            },
            Block {
                name: "damage_gauges line 1063",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6F0, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A314, In::F)],
                        writes: &[(0x31A314, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_94(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[30],
                        &CAR.suspension_damage[2],
                        &CAR.suspension_damage[2],
                    );
                },
            },
            Block {
                name: "damage_gauges line 1075",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6F0, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A318, In::F)],
                        writes: &[(0x31A318, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_95(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[30],
                        &CAR.suspension_damage[3],
                        &CAR.suspension_damage[3],
                    );
                },
            },
            Block {
                name: "damage_gauges line 1087",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6E4, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A29C, In::F)],
                        writes: &[(0x31A29C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_96(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[27],
                        &CAR.cooling_damage,
                        &CAR.cooling_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 1099",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6E4, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A29C, In::F)],
                        writes: &[(0x31A29C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_97(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[27],
                        &CAR.cooling_damage,
                        &CAR.cooling_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 1111",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6E0, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A290, In::F)],
                        writes: &[(0x31A290, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_98(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[26],
                        &CAR.engine_damage,
                        &CAR.engine_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 1123",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6E0, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A290, In::F)],
                        writes: &[(0x31A290, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_99(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[26],
                        &CAR.engine_damage,
                        &CAR.engine_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 1135",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6EC, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A320, In::F)],
                        writes: &[(0x31A320, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_100(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[29],
                        &CAR.brake_wear_a,
                        &CAR.brake_wear_a,
                    );
                },
            },
            Block {
                name: "damage_gauges line 1147",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6EC, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A320, In::F)],
                        writes: &[(0x31A320, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_101(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[29],
                        &CAR.brake_wear_a,
                        &CAR.brake_wear_a,
                    );
                },
            },
            Block {
                name: "damage_gauges line 1159",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6EC, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A324, In::F)],
                        writes: &[(0x31A324, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_102(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[29],
                        &CAR.brake_wear_b,
                        &CAR.brake_wear_b,
                    );
                },
            },
            Block {
                name: "damage_gauges line 1171",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B3C8, In::F), (0x35B6EC, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A324, In::F)],
                        writes: &[(0x31A324, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_103(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        &dm(a[0]).flt_75b3c8,
                        &dm(a[0]).figures[29],
                        &CAR.brake_wear_b,
                        &CAR.brake_wear_b,
                    );
                },
            },
            Block {
                name: "damage_gauges line 1183",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B3CC, In::F), (0x35B6B0, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A30C, In::F)],
                        writes: &[(0x31A30C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_104(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).wheel_loads[0],
                        &dm(a[0]).figures[14],
                        &dm(a[0]).flt_75b370,
                        &CAR.suspension_damage[0],
                        &CAR.suspension_damage[0],
                    );
                },
            },
            Block {
                name: "damage_gauges line 1194",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B3D0, In::F), (0x35B6B0, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A310, In::F)],
                        writes: &[(0x31A310, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_105(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).wheel_loads[1],
                        &dm(a[0]).figures[14],
                        &dm(a[0]).flt_75b370,
                        &CAR.suspension_damage[1],
                        &CAR.suspension_damage[1],
                    );
                },
            },
            Block {
                name: "damage_gauges line 1205",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B3D4, In::F), (0x35B6B0, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A314, In::F)],
                        writes: &[(0x31A314, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_106(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).wheel_loads[2],
                        &dm(a[0]).figures[14],
                        &dm(a[0]).flt_75b370,
                        &CAR.suspension_damage[2],
                        &CAR.suspension_damage[2],
                    );
                },
            },
            Block {
                name: "damage_gauges line 1216",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B3D8, In::F), (0x35B6B0, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A318, In::F)],
                        writes: &[(0x31A318, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_107(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).wheel_loads[3],
                        &dm(a[0]).figures[14],
                        &dm(a[0]).flt_75b370,
                        &CAR.suspension_damage[3],
                        &CAR.suspension_damage[3],
                    );
                },
            },
            Block {
                name: "damage_gauges line 1227",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B3DC, In::F), (0x35B6B4, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A294, In::F)],
                        writes: &[(0x31A294, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_108(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b3dc,
                        &dm(a[0]).figures[15],
                        &dm(a[0]).flt_75b370,
                        &CAR.exhaust_damage,
                        &CAR.exhaust_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 1238",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B370, In::F), (0x35B3DC, In::F), (0x35B6B8, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A308, In::F)],
                        writes: &[(0x31A308, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_109(a[0], a[1]) },
                plain: |a| {
                    x87_5(
                        &dm(a[0]).flt_75b3dc,
                        &dm(a[0]).figures[16],
                        &dm(a[0]).flt_75b370,
                        &CAR.steering_damage,
                        &CAR.steering_damage,
                    );
                },
            },
            Block {
                name: "damage_gauges line 1249",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[
                            (0x31A2A8, In::F),
                            (0x31A2B0, In::F),
                            (0x31A2B4, In::F),
                            (0x31A2B8, In::F),
                            (0x31A2BC, In::F),
                            (0x31A2C0, In::F),
                            (0x31A2C4, In::F),
                            (0x31A2C8, In::F),
                            (0x31A2D0, In::F),
                            (0x31A2D4, In::F),
                            (0x31A2D8, In::F),
                            (0x31A2DC, In::F),
                            (0x31A2E0, In::F),
                            (0x31A2EC, In::F),
                            (0x31A2F0, In::F),
                            (0x31A2F4, In::F),
                            (0x31A2F8, In::F),
                            (0x31A2FC, In::F),
                            (0x31A300, In::F),
                            (0x31A30C, In::F),
                            (0x31A310, In::F),
                            (0x31A314, In::F),
                            (0x31A318, In::F),
                            (0x31A320, In::F),
                            (0x31A324, In::F),
                            (0x31A32C, In::F),
                            (0x31A330, In::F),
                            (0x31A334, In::F),
                            (0x31A338, In::F),
                            (0x31A420, In::F),
                            (0x31A424, In::F),
                            (0x31A428, In::F),
                            (0x31A42C, In::F),
                        ],
                        writes: &[
                            (0x31A2CC, Out::F),
                            (0x31A2E4, Out::F),
                            (0x31A304, Out::F),
                            (0x31A31C, Out::F),
                            (0x31A328, Out::F),
                            (0x31A33C, Out::F),
                            (0x31A430, Out::F),
                        ],
                    },
                    Operand::Buffer {
                        reads: &[
                            (0x35B344, In::F),
                            (0x35B360, In::F),
                            (0x35B364, In::F),
                            (0x35B368, In::F),
                            (0x35B440, In::F),
                            (0x35B444, In::F),
                        ],
                        writes: &[(0x35B440, Out::F), (0x35B444, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_110(a[0], a[1]) },
                plain: |a| x87_13(dm(a[1])),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
