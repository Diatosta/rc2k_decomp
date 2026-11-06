//! `physics::rate_damage` (0x4AF1EC): the current car's damage figures (the work copy's
//! 0x71A278 .. 0x71A328) made into its working grades and failures (IDA, 2026-10-17;
//! what each figure is, is a reading at best). In the original's order:
//! - `dword_71A3CC` = 1 - `dword_71A290` k k694 - `flt_71A294` k k698 (k = `flt_75B388`;
//!   80 bits), times `flt_75B5A0` when `flt_75B3E4`'s bits are not 0 and `flt_71A298` is
//!   not below `flt_75B34C`; held to `flt_75B350` .. `flt_75B348`;
//! - `flt_71A3D0`, `flt_71A3D4`: 1 less four products each (`flt_71A284` / `flt_71A288`,
//!   `flt_71A320` / `flt_71A324`, `flt_71A278`, `dword_75B3F0`), held the same way;
//! - 23 ramps (`math::x87_run::ramp`, tables of (x0, y0, x1, y1) at 0x75B4E8 .. 0x75B568)
//!   into 0x71A3D8 .. 0x71A3E0 and 0x75B404 .. 0x75B49C, two of them times
//!   `flt_75B43C` k k, one squared;
//! - failures by chance: a figure times `flt_75B340` (`fistp` into `flt_75B36C`) against
//!   the random `dword_75B4A0` turned right by 4 .. 0x19, low word (the sign of the
//!   difference): `dword_71A400` (set once), `dword_71A3E4` .. `dword_71A3FC` (set from
//!   0x75B404 .., cleared again from 0x75B420 ..), `dword_71A404`, `dword_71A408`;
//! - `flt_71A2AC` = 0, and `g_carGearBits` = the gears (9 figures from 0x71A2A8) equal to
//!   `flt_75B34C` (C3: also no number);
//! - `dword_71A274` = 0 when five figures are all below `flt_75B34C`, else 1, and then
//!   `dword_71A3CC` = 0 unless `flt_71A2E8` is below it.
//!
//! In:  nothing
//! Out: eax `g_carGearBits`, ecx 9, esi 0x75B568 (the last table); the rest kept; no
//!      flags read (`sub_4AB5D0` returns, `sub_43BA8E` returns, `sub_43EB5D` goes on with
//!      `fild`)
//!
//! Caller: `physics::rate_current_car` (0x4AB5D0), its only one.

use crate::math::x87;
use crate::math::x87_run::{compare_full, ext_times, mul3, one_less_products, ramp, square};
use crate::race::owner::Race;
use crate::scene::matrix3::rounded;
use crate::session::{Dword, Float};

/// The ramps in order: (destination, figure, table).
fn ramps_a(race: &Race) -> [(&Float, &Float, &[Float; 4]); 2] {
    [
        (
            &race.shared.car.drive_efficiency[0],
            &race.shared.car.damper_wear[0],
            &race.physics.damage.lerp_tables[4],
        ),
        (
            &race.shared.car.drive_efficiency[1],
            &race.shared.car.damper_wear[1],
            &race.physics.damage.lerp_tables[4],
        ),
    ]
}
/// More ramps: (destination in the model, the car's figure, table).
fn ramps_c(race: &Race) -> [(&Float, &Float, &[Float; 4]); 18] {
    [
        (
            &race.physics.damage.flt_75b490,
            &race.shared.car.clutch_damage,
            &race.physics.damage.lerp_tables[7],
        ),
        (
            &race.physics.damage.flt_75b494,
            &race.shared.car.gearbox_damage,
            &race.physics.damage.lerp_tables[8],
        ),
        (
            &race.physics.damage.flt_75b498,
            &race.shared.car.gearbox_damage,
            &race.physics.damage.lerp_tables[9],
        ),
        (
            &race.physics.damage.flt_75b49c,
            &race.shared.car.brake_damage,
            &race.physics.damage.lerp_tables[10],
        ),
        (
            &race.physics.damage.flt_75b404,
            &race.shared.car.electrics_damage,
            &race.physics.damage.lerp_tables[11],
        ),
        (
            &race.physics.damage.flt_75b408,
            &race.shared.car.light_damage[0],
            &race.physics.damage.lerp_tables[11],
        ),
        (
            &race.physics.damage.flt_75b40c,
            &race.shared.car.light_damage[1],
            &race.physics.damage.lerp_tables[11],
        ),
        (
            &race.physics.damage.flt_75b410,
            &race.shared.car.light_damage[2],
            &race.physics.damage.lerp_tables[11],
        ),
        (
            &race.physics.damage.flt_75b414,
            &race.shared.car.light_damage[3],
            &race.physics.damage.lerp_tables[11],
        ),
        (
            &race.physics.damage.flt_75b418,
            &race.shared.car.light_damage[4],
            &race.physics.damage.lerp_tables[11],
        ),
        (
            &race.physics.damage.flt_75b41c,
            &race.shared.car.light_damage[5],
            &race.physics.damage.lerp_tables[11],
        ),
        (
            &race.physics.damage.flt_75b420,
            &race.shared.car.electrics_damage,
            &race.physics.damage.lerp_tables[11],
        ),
        (
            &race.physics.damage.flt_75b424,
            &race.shared.car.light_damage[0],
            &race.physics.damage.lerp_tables[12],
        ),
        (
            &race.physics.damage.flt_75b428,
            &race.shared.car.light_damage[1],
            &race.physics.damage.lerp_tables[12],
        ),
        (
            &race.physics.damage.flt_75b42c,
            &race.shared.car.light_damage[2],
            &race.physics.damage.lerp_tables[12],
        ),
        (
            &race.physics.damage.flt_75b430,
            &race.shared.car.light_damage[3],
            &race.physics.damage.lerp_tables[12],
        ),
        (
            &race.physics.damage.flt_75b434,
            &race.shared.car.light_damage[4],
            &race.physics.damage.lerp_tables[12],
        ),
        (
            &race.physics.damage.flt_75b438,
            &race.shared.car.light_damage[5],
            &race.physics.damage.lerp_tables[12],
        ),
    ]
}
/// The failures that come and go: (flag, figure while set, figure while clear, turn).
fn failures(race: &Race) -> [(&Dword, &Float, &Float, u32); 7] {
    [
        (
            &race.shared.car.electrical_failures[0],
            &race.physics.damage.flt_75b420,
            &race.physics.damage.flt_75b404,
            0x4,
        ),
        (
            &race.shared.car.electrical_failures[1],
            &race.physics.damage.flt_75b424,
            &race.physics.damage.flt_75b408,
            0x5,
        ),
        (
            &race.shared.car.electrical_failures[2],
            &race.physics.damage.flt_75b428,
            &race.physics.damage.flt_75b40c,
            0x6,
        ),
        (
            &race.shared.car.electrical_failures[3],
            &race.physics.damage.flt_75b42c,
            &race.physics.damage.flt_75b410,
            0x7,
        ),
        (
            &race.shared.car.electrical_failures[4],
            &race.physics.damage.flt_75b430,
            &race.physics.damage.flt_75b414,
            0x8,
        ),
        (
            &race.shared.car.electrical_failures[5],
            &race.physics.damage.flt_75b434,
            &race.physics.damage.flt_75b418,
            0x9,
        ),
        (
            &race.shared.car.electrical_failures[6],
            &race.physics.damage.flt_75b438,
            &race.physics.damage.flt_75b41c,
            0xA,
        ),
    ]
}

/// The Rust version (the exact one, for `physics::grade_or_reset` and
/// `physics::rate_current_car`).
pub fn rate_damage(race: &Race) {
    let d = &race.physics.damage;
    let c = &race.shared.car;
    let less = |a: u32, b: u32| compare_full(a, b) & 0x100 != 0;
    let hold = |v: &Float| {
        if less(v.bits(), d.gauge_low.bits()) {
            v.set_bits(x87::loaded(d.gauge_low.bits()));
        }
        if !less(v.bits(), d.grade_high.bits()) {
            v.set_bits(x87::loaded(d.grade_high.bits()));
        }
    };
    let k = d.grade_k.bits();
    let mut e = one_less_products(&[
        [c.engine_damage.bits(), k, d.figures[7].bits()],
        [c.exhaust_damage.bits(), k, d.figures[8].bits()],
    ]);
    if d.flt_75b3e4.bits() != 0 && !less(c.turbo_damage.bits(), d.gauge_top.bits()) {
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
            [d.dword_75b3f0.bits(), k, d.figures[12].bits()],
        ]);
        out.set_bits(rounded(e));
        hold(out);
    }
    let table_of = |t: &[Float; 4]| [t[0].bits(), t[1].bits(), t[2].bits(), t[3].bits()];
    for (out, x, t) in ramps_a(race) {
        out.set_bits(ramp(x.bits(), table_of(t), d.gauge_low.bits()));
    }
    for (out, x) in [
        (&c.drive_efficiency[0], &d.flt_75b440),
        (&c.drive_efficiency[1], &d.flt_75b444),
    ] {
        d.flt_75b43c.set_bits(ramp(
            x.bits(),
            table_of(&d.lerp_tables[5]),
            d.gauge_low.bits(),
        ));
        out.set_bits(mul3(d.flt_75b43c.bits(), k, k, out.bits()));
    }
    c.steering_efficiency.set_bits(ramp(
        c.steering_damage.bits(),
        table_of(&d.lerp_tables[6]),
        d.gauge_low.bits(),
    ));
    c.steering_efficiency
        .set_bits(square(c.steering_efficiency.bits()));
    for (out, x, t) in ramps_c(race) {
        out.set_bits(ramp(x.bits(), table_of(t), d.gauge_low.bits()));
    }
    // Chance: the figure times `flt_75B340` against the random's low word turned.
    let chance = |figure: &Float, turn: u32| {
        let t = x87::mul_to_int(figure.bits(), d.chance_scale.bits()) as u32;
        d.scratch.set_bits(t);
        let r = d.clock.get().rotate_right(turn) & 0xFFFF;
        (r.wrapping_sub(t) as i32) < 0
    };
    if c.handbrake_failed.get() == 0 && chance(&d.flt_75b49c, 0xD) {
        c.handbrake_failed.set(1);
    }
    for (flag, set, clear, turn) in failures(race) {
        if flag.get() != 0 {
            if chance(set, turn) {
                flag.set(0);
            }
        } else if chance(clear, turn) {
            flag.set(1);
        }
    }
    c.gear_change_blocked.set(0);
    if chance(&d.flt_75b490, 0xB) {
        c.gear_change_blocked.set(1);
    }
    if chance(&d.flt_75b494, 0x19) {
        c.gear_change_blocked.set(1);
    }
    c.forced_neutral.set(0);
    if chance(&d.flt_75b498, 0x12) {
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
