//! `physics::rate_damage` (0x4AF1EC): the current car's damage figures (the work copy's
//! 0x71A278 .. 0x71A328) made into its working grades and failures (what each figure
//! is, is in `physics/store.rs`). In the original's order:
//! - `engine_power_factor` = 1 - `engine_damage` k k694 - `exhaust_damage` k k698 (k = `flt_75B388`;
//!   80 bits), times `flt_75B5A0` when `flt_75B3E4`'s bits are not 0 and `turbo_damage` is
//!   not below `flt_75B34C`; held to `flt_75B350` .. `flt_75B348`;
//! - `brake_efficiency`, `flt_71A3D4`: 1 less four products each (`brake_heat` / `flt_71A288`,
//!   `brake_wear_a` / `brake_wear_b`, `brake_fade`, `dword_75B3F0`), held the same way;
//! - 23 ramps (`math::x87::ramp`, tables of (x0, y0, x1, y1) at 0x75B4E8 .. 0x75B568)
//!   into 0x71A3D8 .. 0x71A3E0 and the model's failure chances (0x75B404 .. 0x75B438, 0x75B490 .. 0x75B49C), two of them times
//!   `flt_75B43C` k k, one squared;
//! - failures by chance: a figure times `flt_75B340` (`fistp` into `flt_75B36C`) against
//!   the random `dword_75B4A0` turned right by 4 .. 0x19, low word (the sign of the
//!   difference): `handbrake_failed` (set once), `electrical_failures` .. `dword_71A3FC` (set from
//!   0x75B404 .., cleared again from 0x75B420 ..), `gear_change_blocked`, `forced_neutral`;
//! - `flt_71A2AC` = 0, and `g_carGearBits` = the gears (9 figures from 0x71A2A8) equal to
//!   `flt_75B34C` (C3: also no number);
//! - `damage_failed` = 0 when five figures are all below `flt_75B34C`, else 1, and then
//!   `engine_power_factor` = 0 unless `electrics_damage` is below it.
//!
//! Caller: `physics::rate_current_car` (0x4AB5D0), its only one.

use crate::math::x87;
use crate::math::x87::{compare_full, ext_times, one_less_products, ramp};
use crate::math::x87_float as bh;
use crate::physics::car_store::Car;
use crate::physics::store::DamageModel;
use crate::scene::matrix3::rounded;
use crate::session::{Dword, Float};

/// The ramps in order: (destination, figure, table).
fn ramps_a<'a>(
    car: &'a Car,
    damage: &'a DamageModel,
) -> [(&'a Float, &'a Float, &'a [Float; 4]); 2] {
    [
        (
            &car.drive_efficiency[0],
            &car.damper_wear[0],
            &damage.lerp_tables[4],
        ),
        (
            &car.drive_efficiency[1],
            &car.damper_wear[1],
            &damage.lerp_tables[4],
        ),
    ]
}
/// More ramps: (destination in the model, the car's figure, table).
fn ramps_c<'a>(
    car: &'a Car,
    damage: &'a DamageModel,
) -> [(&'a Float, &'a Float, &'a [Float; 4]); 18] {
    [
        (
            &damage.clutch_block_chance,
            &car.clutch_damage,
            &damage.lerp_tables[7],
        ),
        (
            &damage.gearbox_block_chance,
            &car.gearbox_damage,
            &damage.lerp_tables[8],
        ),
        (
            &damage.gearbox_neutral_chance,
            &car.gearbox_damage,
            &damage.lerp_tables[9],
        ),
        (
            &damage.handbrake_fail_chance,
            &car.brake_damage,
            &damage.lerp_tables[10],
        ),
        (
            &damage.electrics_fail_chance,
            &car.electrics_damage,
            &damage.lerp_tables[11],
        ),
        (
            &damage.light_fail_chance_0,
            &car.light_damage[0],
            &damage.lerp_tables[11],
        ),
        (
            &damage.light_fail_chance_1,
            &car.light_damage[1],
            &damage.lerp_tables[11],
        ),
        (
            &damage.light_fail_chance_2,
            &car.light_damage[2],
            &damage.lerp_tables[11],
        ),
        (
            &damage.light_fail_chance_3,
            &car.light_damage[3],
            &damage.lerp_tables[11],
        ),
        (
            &damage.light_fail_chance_4,
            &car.light_damage[4],
            &damage.lerp_tables[11],
        ),
        (
            &damage.light_fail_chance_5,
            &car.light_damage[5],
            &damage.lerp_tables[11],
        ),
        (
            &damage.electrics_recover_chance,
            &car.electrics_damage,
            &damage.lerp_tables[11],
        ),
        (
            &damage.light_recover_chance_0,
            &car.light_damage[0],
            &damage.lerp_tables[12],
        ),
        (
            &damage.light_recover_chance_1,
            &car.light_damage[1],
            &damage.lerp_tables[12],
        ),
        (
            &damage.light_recover_chance_2,
            &car.light_damage[2],
            &damage.lerp_tables[12],
        ),
        (
            &damage.light_recover_chance_3,
            &car.light_damage[3],
            &damage.lerp_tables[12],
        ),
        (
            &damage.light_recover_chance_4,
            &car.light_damage[4],
            &damage.lerp_tables[12],
        ),
        (
            &damage.light_recover_chance_5,
            &car.light_damage[5],
            &damage.lerp_tables[12],
        ),
    ]
}
/// The failures that come and go: (flag, figure while set, figure while clear, turn).
fn failures<'a>(
    car: &'a Car,
    damage: &'a DamageModel,
) -> [(&'a Dword, &'a Float, &'a Float, u32); 7] {
    [
        (
            &car.electrical_failures[0],
            &damage.electrics_recover_chance,
            &damage.electrics_fail_chance,
            0x4,
        ),
        (
            &car.electrical_failures[1],
            &damage.light_recover_chance_0,
            &damage.light_fail_chance_0,
            0x5,
        ),
        (
            &car.electrical_failures[2],
            &damage.light_recover_chance_1,
            &damage.light_fail_chance_1,
            0x6,
        ),
        (
            &car.electrical_failures[3],
            &damage.light_recover_chance_2,
            &damage.light_fail_chance_2,
            0x7,
        ),
        (
            &car.electrical_failures[4],
            &damage.light_recover_chance_3,
            &damage.light_fail_chance_3,
            0x8,
        ),
        (
            &car.electrical_failures[5],
            &damage.light_recover_chance_4,
            &damage.light_fail_chance_4,
            0x9,
        ),
        (
            &car.electrical_failures[6],
            &damage.light_recover_chance_5,
            &damage.light_fail_chance_5,
            0xA,
        ),
    ]
}

/// The exact version, for `physics::grade_or_reset` and `physics::rate_current_car`.
pub fn rate_damage(car: &Car, damage: &DamageModel) {
    let d = &damage;
    let c = &car;
    let less = |a: u32, b: u32| compare_full(a, b) & 0x100 != 0;
    let hold = |v: &Float| {
        if less(v.bits(), d.gauge_low.bits()) {
            v.set_bits(x87::loaded(&d.gauge_low));
        }
        if !less(v.bits(), d.grade_high.bits()) {
            v.set_bits(x87::loaded(&d.grade_high));
        }
    };
    let k = d.grade_k.bits();
    let mut e = one_less_products(&[
        [c.engine_damage.bits(), k, d.figures[7].bits()],
        [c.exhaust_damage.bits(), k, d.figures[8].bits()],
    ]);
    if d.drive_kind.bits() != 0 && !less(c.turbo_damage.bits(), d.gauge_top.bits()) {
        e = ext_times(e, d.wear_rates[10].bits());
    }
    c.engine_power_factor.set_bits(rounded(e));
    hold(&c.engine_power_factor);
    for (out, a, b) in [
        (&c.brake_efficiency[0], &c.brake_heat[0], &c.brake_wear_a),
        (&c.brake_efficiency[1], &c.brake_heat[1], &c.brake_wear_b),
    ] {
        let e = one_less_products(&[
            [a.bits(), k, d.figures[10].bits()],
            [b.bits(), k, d.figures[9].bits()],
            [c.brake_fade.bits(), k, d.figures[11].bits()],
            [d.surface_blend.bits(), k, d.figures[12].bits()],
        ]);
        out.set_bits(rounded(e));
        hold(out);
    }
    let table_of = |t: &[Float; 4]| [t[0].bits(), t[1].bits(), t[2].bits(), t[3].bits()];
    for (out, x, t) in ramps_a(car, damage) {
        out.set_bits(ramp(x.bits(), table_of(t), d.gauge_low.bits()));
    }
    for (out, x) in [
        (&c.drive_efficiency[0], &d.front_tyre_wear),
        (&c.drive_efficiency[1], &d.rear_tyre_wear),
    ] {
        d.brake_balance_complement.set_bits(ramp(
            x.bits(),
            table_of(&d.lerp_tables[5]),
            d.gauge_low.bits(),
        ));
        out.set_bits(bh::st(
            bh::ld(d.brake_balance_complement.bits()) * bh::ld(k) * bh::ld(k) * bh::ld(out.bits()),
        ));
    }
    c.steering_efficiency.set_bits(ramp(
        c.steering_damage.bits(),
        table_of(&d.lerp_tables[6]),
        d.gauge_low.bits(),
    ));
    let efficiency = c.steering_efficiency.bits();
    c.steering_efficiency
        .set_bits(x87::mul(efficiency, efficiency));
    for (out, x, t) in ramps_c(car, damage) {
        out.set_bits(ramp(x.bits(), table_of(t), d.gauge_low.bits()));
    }
    // Chance: the figure times `flt_75B340` against the random's low word turned.
    let chance = |figure: &Float, turn: u32| {
        let t = x87::mul_to_int(figure, &d.chance_scale) as u32;
        d.scratch.set_bits(t);
        let r = d.clock.get().rotate_right(turn) & 0xFFFF;
        (r.wrapping_sub(t) as i32) < 0
    };
    if c.handbrake_failed.get() == 0 && chance(&d.handbrake_fail_chance, 0xD) {
        c.handbrake_failed.set(1);
    }
    for (flag, set, clear, turn) in failures(car, damage) {
        if flag.get() != 0 {
            if chance(set, turn) {
                flag.set(0);
            }
        } else if chance(clear, turn) {
            flag.set(1);
        }
    }
    c.gear_change_blocked.set(0);
    if chance(&d.clutch_block_chance, 0xB) {
        c.gear_change_blocked.set(1);
    }
    if chance(&d.gearbox_block_chance, 0x19) {
        c.gear_change_blocked.set(1);
    }
    c.forced_neutral.set(0);
    if chance(&d.gearbox_neutral_chance, 0x12) {
        c.forced_neutral.set(1);
    }
    c.gear_wear[1].set_bits(0);
    let mut bits = 0u32;
    for (n, wear) in c.gear_wear.iter().enumerate() {
        if compare_full(wear.bits(), d.gauge_top.bits()) & 0x4000 != 0 {
            bits |= 1 << n;
        }
    }
    c.lost_gear_bits.set(bits);
    let all_below = [
        &c.engine_damage,
        &c.clutch_damage,
        &c.differential_damage,
        &c.gearbox_damage,
        &c.electrics_damage,
    ]
    .iter()
    .all(|v| less(v.bits(), d.gauge_top.bits()));
    if all_below {
        c.damage_failed.set(0);
    } else {
        c.damage_failed.set(1);
        if !less(c.electrics_damage.bits(), d.gauge_top.bits()) {
            c.engine_power_factor.set_bits(0);
        }
    }
}
