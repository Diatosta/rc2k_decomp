//! Makes the current car's damage figures (the work copy's gauges; what each figure is, is in
//! `physics/store.rs`) into its working grades and failures:
//! - `engine_power_factor` = 1 - `engine_damage` x k x `figures[7]` - `exhaust_damage` x k x
//!   `figures[8]` (k = `DamageModel.grade_k`; 80 bits), times `DamageModel.wear_rates[10]` when
//!   `DamageModel.drive_kind`'s bits are not 0 and `turbo_damage` is not below
//!   `DamageModel.gauge_top`; held to `DamageModel.gauge_low` .. `DamageModel.grade_high`;
//! - `brake_efficiency`, `Car.brake_efficiency[1]`: 1 less four products each (`brake_heat` /
//!   `Car.brake_heat[1]`, `brake_wear_a` / `brake_wear_b`, `brake_fade`,
//!   `DamageModel.surface_blend`), held the same way;
//! - 23 ramps (`math::x87::ramp`, tables of (x0, y0, x1, y1)) into the car's grade fields and the
//!   model's failure chances, two of them times `DamageModel.brake_balance_complement` x k x k, one
//!   squared;
//! - failures by chance: a figure times `DamageModel.chance_scale` (rounded into
//!   `DamageModel.scratch`) against the random `DamageModel.clock` turned right by 4 .. 0x19, low
//!   word: `handbrake_failed` (set once), `electrical_failures` .. `Car.electrical_failures[6]`
//!   (set from one run of chances, cleared again from another), `gear_change_blocked`,
//!   `forced_neutral`;
//! - `Car.gear_wear[1]` = 0, and `lost_gear_bits` = the gears (nine figures) equal to
//!   `DamageModel.gauge_top` (also when no number);
//! - `damage_failed` = 0 when five figures are all below `DamageModel.gauge_top`, else 1, and then
//!   `engine_power_factor` = 0 unless `electrics_damage` is below it.

use crate::math::cell_x87::CellX87;
use crate::math::matrix3::rounded_value;
use crate::math::x87;
use crate::math::x87::{Flt, compare_full, ext_times, one_less_products, ramp};
use crate::math::x87_float as bh;
use crate::memory::session::{PlainDword, PlainFloat};
use crate::physics::car_store::Car;
use crate::physics::store::DamageModel;

/// The ramps for the model's failure and recovery chances, in the order the model holds them:
/// the clutch, gearbox (two), handbrake, electrics and six lights' failures, then the
/// electrics' and six lights' recoveries.
fn chance_ramps(car: &Car, damage: &DamageModel) -> [f32; 18] {
    let ramp_of = |x: f32, table: usize| ramp(x, damage.lerp_tables[table], damage.gauge_low);
    let light = |n: usize| car.light_damage[n].get();
    [
        ramp_of(car.clutch_damage.get(), 7),
        ramp_of(car.gearbox_damage.get(), 8),
        ramp_of(car.gearbox_damage.get(), 9),
        ramp_of(car.brake_damage.get(), 10),
        ramp_of(car.electrics_damage.get(), 11),
        ramp_of(light(0), 11),
        ramp_of(light(1), 11),
        ramp_of(light(2), 11),
        ramp_of(light(3), 11),
        ramp_of(light(4), 11),
        ramp_of(light(5), 11),
        ramp_of(car.electrics_damage.get(), 11),
        ramp_of(light(0), 12),
        ramp_of(light(1), 12),
        ramp_of(light(2), 12),
        ramp_of(light(3), 12),
        ramp_of(light(4), 12),
        ramp_of(light(5), 12),
    ]
}

/// The failures that come and go: (flag, figure while set, figure while clear, turn).
fn failures<'a>(car: &'a Car, damage: &DamageModel) -> [(&'a PlainDword, f32, f32, u32); 7] {
    [
        (
            &car.electrical_failures[0],
            damage.electrics_recover_chance,
            damage.electrics_fail_chance,
            0x4,
        ),
        (
            &car.electrical_failures[1],
            damage.light_recover_chance_0,
            damage.light_fail_chance_0,
            0x5,
        ),
        (
            &car.electrical_failures[2],
            damage.light_recover_chance_1,
            damage.light_fail_chance_1,
            0x6,
        ),
        (
            &car.electrical_failures[3],
            damage.light_recover_chance_2,
            damage.light_fail_chance_2,
            0x7,
        ),
        (
            &car.electrical_failures[4],
            damage.light_recover_chance_3,
            damage.light_fail_chance_3,
            0x8,
        ),
        (
            &car.electrical_failures[5],
            damage.light_recover_chance_4,
            damage.light_fail_chance_4,
            0x9,
        ),
        (
            &car.electrical_failures[6],
            damage.light_recover_chance_5,
            damage.light_fail_chance_5,
            0xA,
        ),
    ]
}

/// The figure times `chance_scale` (rounded into `scratch`) against the random's low word turned.
fn chance(scratch: &mut f32, clock: u32, chance_scale: f32, figure: f32, turn: u32) -> bool {
    let t = x87::mul_to_int(figure, chance_scale) as u32;
    *scratch = f32::from_bits(t);
    let r = clock.rotate_right(turn) & 0xFFFF;
    (r.wrapping_sub(t) as i32) < 0
}

/// `a` is below `b` or unordered.
fn less(a: impl Flt, b: impl Flt) -> bool {
    compare_full(a, b) & 0x100 != 0
}

/// The exact version, for `physics::grade_or_reset`.
pub fn rate_damage(car: &Car, d: &mut DamageModel) {
    let c = &car;
    let hold = |v: &PlainFloat, d: &DamageModel| {
        if less(v, d.gauge_low) {
            v.set(x87::loaded(d.gauge_low));
        }
        if !less(v, d.grade_high) {
            v.set(x87::loaded(d.grade_high));
        }
    };
    let k = d.grade_k;
    let mut e = one_less_products(&[
        [c.engine_damage.get(), k, d.figures[7]],
        [c.exhaust_damage.get(), k, d.figures[8]],
    ]);
    if d.drive_kind != 0 && !less(&c.turbo_damage, d.gauge_top) {
        e = ext_times(e, d.wear_rates[10]);
    }
    c.engine_power_factor.set(rounded_value(e));
    hold(&c.engine_power_factor, d);
    for (out, a, b) in [
        (&c.brake_efficiency[0], &c.brake_heat[0], &c.brake_wear_a),
        (&c.brake_efficiency[1], &c.brake_heat[1], &c.brake_wear_b),
    ] {
        let e = one_less_products(&[
            [a.get(), k, d.figures[10]],
            [b.get(), k, d.figures[9]],
            [c.brake_fade.get(), k, d.figures[11]],
            [d.surface_blend, k, d.figures[12]],
        ]);
        out.set(rounded_value(e));
        hold(out, d);
    }
    for (out, x) in [
        (&c.drive_efficiency[0], c.damper_wear[0].get()),
        (&c.drive_efficiency[1], c.damper_wear[1].get()),
    ] {
        out.set(ramp(x, d.lerp_tables[4], d.gauge_low));
    }
    for (out, x) in [
        (&c.drive_efficiency[0], d.front_tyre_wear),
        (&c.drive_efficiency[1], d.rear_tyre_wear),
    ] {
        d.brake_balance_complement = ramp(x, d.lerp_tables[5], d.gauge_low);
        out.st(bh::ld(d.brake_balance_complement) * bh::ld(k) * bh::ld(k) * out.ld());
    }
    c.steering_efficiency
        .set(ramp(&c.steering_damage, d.lerp_tables[6], d.gauge_low));
    let efficiency = c.steering_efficiency.get();
    c.steering_efficiency.set(x87::mul(efficiency, efficiency));
    let [
        clutch_block,
        gearbox_block,
        gearbox_neutral,
        handbrake_fail,
        electrics_fail,
        light_fail_0,
        light_fail_1,
        light_fail_2,
        light_fail_3,
        light_fail_4,
        light_fail_5,
        electrics_recover,
        light_recover_0,
        light_recover_1,
        light_recover_2,
        light_recover_3,
        light_recover_4,
        light_recover_5,
    ] = chance_ramps(car, d);
    d.clutch_block_chance = clutch_block;
    d.gearbox_block_chance = gearbox_block;
    d.gearbox_neutral_chance = gearbox_neutral;
    d.handbrake_fail_chance = handbrake_fail;
    d.electrics_fail_chance = electrics_fail;
    d.light_fail_chance_0 = light_fail_0;
    d.light_fail_chance_1 = light_fail_1;
    d.light_fail_chance_2 = light_fail_2;
    d.light_fail_chance_3 = light_fail_3;
    d.light_fail_chance_4 = light_fail_4;
    d.light_fail_chance_5 = light_fail_5;
    d.electrics_recover_chance = electrics_recover;
    d.light_recover_chance_0 = light_recover_0;
    d.light_recover_chance_1 = light_recover_1;
    d.light_recover_chance_2 = light_recover_2;
    d.light_recover_chance_3 = light_recover_3;
    d.light_recover_chance_4 = light_recover_4;
    d.light_recover_chance_5 = light_recover_5;
    // Chance: the figure times `DamageModel.chance_scale` against the random's low word turned.
    if c.handbrake_failed.get() == 0
        && chance(
            &mut d.scratch,
            d.clock,
            d.chance_scale,
            d.handbrake_fail_chance,
            0xD,
        )
    {
        c.handbrake_failed.set(1);
    }
    for (flag, set, clear, turn) in failures(car, d) {
        if flag.get() != 0 {
            if chance(&mut d.scratch, d.clock, d.chance_scale, set, turn) {
                flag.set(0);
            }
        } else if chance(&mut d.scratch, d.clock, d.chance_scale, clear, turn) {
            flag.set(1);
        }
    }
    c.gear_change_blocked.set(0);
    if chance(
        &mut d.scratch,
        d.clock,
        d.chance_scale,
        d.clutch_block_chance,
        0xB,
    ) {
        c.gear_change_blocked.set(1);
    }
    if chance(
        &mut d.scratch,
        d.clock,
        d.chance_scale,
        d.gearbox_block_chance,
        0x19,
    ) {
        c.gear_change_blocked.set(1);
    }
    c.forced_neutral.set(0);
    if chance(
        &mut d.scratch,
        d.clock,
        d.chance_scale,
        d.gearbox_neutral_chance,
        0x12,
    ) {
        c.forced_neutral.set(1);
    }
    c.gear_wear[1].set(0.0);
    let mut bits = 0u32;
    for (n, wear) in c.gear_wear.iter().enumerate() {
        if compare_full(wear, d.gauge_top) & 0x4000 != 0 {
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
    .all(|v| less(*v, d.gauge_top));
    if all_below {
        c.damage_failed.set(0);
    } else {
        c.damage_failed.set(1);
        if !less(&c.electrics_damage, d.gauge_top) {
            c.engine_power_factor.set(0.0);
        }
    }
}
