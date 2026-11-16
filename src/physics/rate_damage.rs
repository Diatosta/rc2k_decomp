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

use crate::effects::cell_x87::CellX87;
use crate::math::x87;
use crate::math::x87::{compare_full, ext_times, one_less_products, ramp};
use crate::math::x87_float as bh;
use crate::memory::session::{PlainDword, PlainFloat};
use crate::physics::car_store::Car;
use crate::physics::store::DamageModel;
use crate::scene::matrix3::rounded_value;

/// The ramps in order: (destination, figure, table).
fn ramps_a<'a>(
    car: &'a Car,
    damage: &'a DamageModel,
) -> [(&'a PlainFloat, &'a PlainFloat, &'a [PlainFloat; 4]); 2] {
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
) -> [(&'a PlainFloat, &'a PlainFloat, &'a [PlainFloat; 4]); 18] {
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
) -> [(&'a PlainDword, &'a PlainFloat, &'a PlainFloat, u32); 7] {
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

/// The exact version, for `physics::grade_or_reset`.
pub fn rate_damage(car: &Car, damage: &DamageModel) {
    let d = &damage;
    let c = &car;
    let less = |a: &PlainFloat, b: &PlainFloat| compare_full(a, b) & 0x100 != 0;
    let hold = |v: &PlainFloat| {
        if less(v, &d.gauge_low) {
            v.set(x87::loaded(&d.gauge_low));
        }
        if !less(v, &d.grade_high) {
            v.set(x87::loaded(&d.grade_high));
        }
    };
    let k = d.grade_k.get();
    let mut e = one_less_products(&[
        [c.engine_damage.get(), k, d.figures[7].get()],
        [c.exhaust_damage.get(), k, d.figures[8].get()],
    ]);
    if d.drive_kind.get() != 0 && !less(&c.turbo_damage, &d.gauge_top) {
        e = ext_times(e, d.wear_rates[10].get());
    }
    c.engine_power_factor.set(rounded_value(e));
    hold(&c.engine_power_factor);
    for (out, a, b) in [
        (&c.brake_efficiency[0], &c.brake_heat[0], &c.brake_wear_a),
        (&c.brake_efficiency[1], &c.brake_heat[1], &c.brake_wear_b),
    ] {
        let e = one_less_products(&[
            [a.get(), k, d.figures[10].get()],
            [b.get(), k, d.figures[9].get()],
            [c.brake_fade.get(), k, d.figures[11].get()],
            [d.surface_blend.get(), k, d.figures[12].get()],
        ]);
        out.set(rounded_value(e));
        hold(out);
    }
    let table_of = |t: &[PlainFloat; 4]| [t[0].get(), t[1].get(), t[2].get(), t[3].get()];
    for (out, x, t) in ramps_a(car, damage) {
        out.set(ramp(x.get(), table_of(t), d.gauge_low.get()));
    }
    for (out, x) in [
        (&c.drive_efficiency[0], &d.front_tyre_wear),
        (&c.drive_efficiency[1], &d.rear_tyre_wear),
    ] {
        d.brake_balance_complement
            .set(ramp(x, table_of(&d.lerp_tables[5]), &d.gauge_low));
        out.st(d.brake_balance_complement.ld() * bh::ld(k) * bh::ld(k) * out.ld());
    }
    c.steering_efficiency.set(ramp(
        &c.steering_damage,
        table_of(&d.lerp_tables[6]),
        &d.gauge_low,
    ));
    let efficiency = c.steering_efficiency.get();
    c.steering_efficiency.set(x87::mul(efficiency, efficiency));
    for (out, x, t) in ramps_c(car, damage) {
        out.set(ramp(x, table_of(t), &d.gauge_low));
    }
    // Chance: the figure times `DamageModel.chance_scale` against the random's low word turned.
    let chance = |figure: &PlainFloat, turn: u32| {
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
    c.gear_wear[1].set(0.0);
    let mut bits = 0u32;
    for (n, wear) in c.gear_wear.iter().enumerate() {
        if compare_full(wear, &d.gauge_top) & 0x4000 != 0 {
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
    .all(|v| less(v, &d.gauge_top));
    if all_below {
        c.damage_failed.set(0);
    } else {
        c.damage_failed.set(1);
        if !less(&c.electrics_damage, &d.gauge_top) {
            c.engine_power_factor.set(0.0);
        }
    }
}
