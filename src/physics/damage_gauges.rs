//! `physics::damage_gauges` (0x4AC817): the damage gauges' step (`physics/damage_model.rs`).
//!
//! Each gauge of the car block (engine, exhaust, cooling, clutch, differential, electrics,
//! lights, steering, suspension, turbo, brakes, the gearbox's gears, the body and part
//! hits) is raised by its rate (`gauge_rates`, `wear_rates`) times this step's shares and
//! hit figures, then held 0 ..= `gauge_top`. The hit's side (`dword_75B3C4`, 1 .. 4)
//! chooses the gauges a hit wears, its size (`flt_75B3C8`) is scaled past `gauge_rates[22]`,
//! and the direction `dword_75B3B8` (1 or -1) adds fixed steps.
//!
//! Caller: `physics::damage_step`.

use crate::errors::BadInput;
use crate::math::x87::loaded;
use crate::math::x87_float::X;
use crate::math::x87_float::{ld, st};
use crate::physics::flow::{below_or_unordered, c0};
use crate::physics::store::DamageModel;
use crate::race::owner::Race;
use crate::race::owner_physics::Physics;
use crate::race::owner_shared::Shared;
use crate::session::{Cell32, Float};

/// `unk_71A2AC[gear]` (the gear -1 .. 7 as the original indexes it, `gear_wear` by gear + 1).
fn gear_gauge(shared: &Shared, gear: u32) -> &Float {
    shared
        .car
        .gear_wear
        .get(gear.wrapping_add(1) as usize)
        .unwrap_or_else(|| {
            crate::fatal::raise::end("a gear's wear gauge", BadInput("the gear is not -1 .. 7"))
        })
}

/// The gauge at `at` held 0 ..= `flt_75B34C` (the two `fcompp` holds).
fn hold_at(physics: &Physics, at: &Float) {
    let rd = || at.bits();
    let put = |v: u32| at.set_bits(v);
    if c0(rd(), physics.damage.gauge_low.bits()) {
        put(loaded(physics.damage.gauge_low.bits()));
    }
    if !c0(rd(), physics.damage.gauge_top.bits()) {
        put(loaded(physics.damage.gauge_top.bits()));
    }
}

/// The damage gauges' step: each gauge raised by its rate times this step's shares and hit
/// figures, then held 0 ..= `gauge_top`.
pub fn steps(race: &Race) {
    raise(&race.physics, &race.shared.car.engine_damage, |g| {
        add_product(
            &race.physics.damage.gauge_rates[0],
            &race.physics.damage.gauge_step,
            g,
            g,
        );
    });
    raise(&race.physics, &race.shared.car.exhaust_damage, |g| {
        add_product(
            &race.physics.damage.gauge_rates[1],
            &race.physics.damage.gauge_step,
            g,
            g,
        );
    });
    raise(&race.physics, &race.shared.car.cooling_damage, |g| {
        add_product(
            &race.physics.damage.gauge_rates[2],
            &race.physics.damage.gauge_step,
            g,
            g,
        );
    });
    raise(&race.physics, &race.shared.car.clutch_damage, |g| {
        add_product(
            &race.physics.damage.gauge_rates[3],
            &race.physics.damage.gauge_step,
            g,
            g,
        );
    });
    raise(&race.physics, &race.shared.car.differential_damage, |g| {
        add_product(
            &race.physics.damage.gauge_rates[4],
            &race.physics.damage.gauge_step,
            g,
            g,
        );
    });
    raise(&race.physics, &race.shared.car.electrics_damage, |g| {
        add_product(
            &race.physics.damage.gauge_rates[5],
            &race.physics.damage.gauge_step,
            g,
            g,
        );
    });
    for k in 0..6 {
        raise(&race.physics, &race.shared.car.light_damage[k], |g| {
            add_product(
                &race.physics.damage.gauge_rates[6],
                &race.physics.damage.gauge_step,
                g,
                g,
            );
        });
    }
    raise(&race.physics, &race.shared.car.steering_damage, |g| {
        add_product(
            &race.physics.damage.gauge_rates[7],
            &race.physics.damage.gauge_step,
            g,
            g,
        );
    });
    for k in 0..4 {
        raise(&race.physics, &race.shared.car.suspension_damage[k], |g| {
            add_product(
                &race.physics.damage.gauge_rates[8],
                &race.physics.damage.gauge_step,
                g,
                g,
            );
        });
    }
    raise(&race.physics, &race.shared.car.turbo_damage, |g| {
        add_scaled_triple(
            &race.physics.damage,
            &race.physics.damage.wear_rates[11],
            &race.physics.damage.engine_over_limit,
            &race.physics.damage.engine_speed_share,
            g,
            g,
        );
    });
    let gear = race.physics.damage.gear.bits();
    let gauge = gear_gauge(&race.shared, gear);
    let factor = race
        .physics
        .damage
        .gear_factors
        .get(gear.wrapping_add(1) as usize)
        .unwrap_or_else(|| {
            crate::fatal::raise::end("a gear's factor", BadInput("the gear is not -1 .. 7"))
        });
    gear_wear_from_speed(&race.physics.damage, factor, gauge);
    hold_at(&race.physics, gauge);
    brake_wear_step(&race.shared, &race.physics.damage);
    hold_at(&race.physics, &race.shared.car.brake_wear_a);
    raise(&race.physics, &race.shared.car.brake_wear_b, |g| {
        add_scaled_triple(
            &race.physics.damage,
            &race.physics.damage.gauge_rates[9],
            &race.physics.damage.unknown_fc,
            &race.physics.damage.brake_share,
            g,
            g,
        );
    });
    raise(&race.physics, &race.shared.car.electrics_damage, |g| {
        add_triple_product(
            &race.physics.damage.gauge_rates[10],
            &race.physics.damage.on_surface_5,
            &race.physics.damage.gauge_step,
            g,
            g,
        );
    });
    for k in 0..6 {
        raise(&race.physics, &race.shared.car.light_damage[k], |g| {
            add_triple_product(
                &race.physics.damage.gauge_rates[11],
                &race.physics.damage.on_surface_5,
                &race.physics.damage.gauge_step,
                g,
                g,
            );
        });
    }
    part_hit_figures(&race.physics.damage);
    let part = race.physics.damage.drive_kind.bits();
    fn kind(table: &[Float; 2], part: u32) -> &Float {
        table.get(part as usize).unwrap_or_else(|| {
            crate::fatal::raise::end("a kind table", BadInput("the kind is not 0 or 1"))
        })
    }
    scale_part_hit_figures(
        &race.physics.damage,
        kind(&race.physics.damage.kind_table_a, part),
        kind(&race.physics.damage.kind_table_b, part),
    );
    for k in 0..2 {
        raise(&race.physics, &race.shared.car.part_hit_damage[k], |g| {
            add_triple_product(
                &race.physics.damage.unknown_12c,
                &race.physics.damage.unknown_148,
                &race.physics.damage.hundredth,
                g,
                g,
            );
        });
    }
    for k in 2..4 {
        raise(&race.physics, &race.shared.car.part_hit_damage[k], |g| {
            add_triple_product(
                &race.physics.damage.unknown_130,
                &race.physics.damage.unknown_14c,
                &race.physics.damage.hundredth,
                g,
                g,
            );
        });
    }
    raise(&race.physics, &race.shared.car.engine_damage, |g| {
        add_triple_product(
            &race.physics.damage.unknown_11c,
            &race.physics.damage.gauge_rates[13],
            &race.physics.damage.hundredth,
            g,
            g,
        );
    });
    raise(&race.physics, &race.shared.car.cooling_damage, |g| {
        add_triple_product(
            &race.physics.damage.unknown_11c,
            &race.physics.damage.gauge_rates[12],
            &race.physics.damage.hundredth,
            g,
            g,
        );
    });
    if !below_or_unordered(
        race.physics.damage.unknown_124.bits(),
        race.physics.damage.gauge_low.bits(),
    ) {
        engine_damage_carry(&race.shared, &race.physics.damage);
        hold_at(&race.physics, &race.shared.car.engine_damage);
        raise(&race.physics, &race.shared.car.cooling_damage, |g| {
            add_triple_product(
                &race.physics.damage.unknown_124,
                &race.physics.damage.gauge_rates[15],
                &race.physics.damage.hundredth,
                g,
                g,
            );
        });
    }
    if race.physics.damage.gear_changed.bits() == 0x1 {
        add_rate_to_clutch(&race.shared, &race.physics.damage.gauge_rates[16]);
    }
    if race.physics.damage.gear_changed.bits() == 0xFFFFFFFF {
        add_rate_to_clutch(&race.shared, &race.physics.damage.gauge_rates[17]);
    }
    let gear = race.physics.damage.gear.bits();
    if race.physics.damage.gear_changed.bits() == 0x1 {
        let gauge = gear_gauge(&race.shared, gear);
        add_rate_to_gauge(gauge, &race.physics.damage.gauge_rates[18]);
    }
    let gear = race.physics.damage.gear.bits();
    if race.physics.damage.gear_changed.bits() == 0xFFFFFFFF {
        let gauge = gear_gauge(&race.shared, gear);
        add_rate_to_gauge(gauge, &race.physics.damage.gauge_rates[19]);
    }
    for k in 0..4 {
        raise(&race.physics, &race.shared.car.wheel_hit_damage[k], |g| {
            add_triple_product(
                &race.physics.damage.unknown_110,
                &race.physics.damage.unknown_330,
                &race.physics.damage.hundredth,
                g,
                g,
            );
        });
    }
    for k in 0..4 {
        raise(&race.physics, &race.shared.car.suspension_damage[k], |g| {
            add_triple_product(
                &race.physics.damage.unknown_118,
                &race.physics.damage.figures[0],
                &race.physics.damage.hundredth,
                g,
                g,
            );
        });
    }
    raise(&race.physics, &race.shared.car.body_hit_damage, |g| {
        add_triple_product(
            &race.physics.damage.unknown_114,
            &race.physics.damage.figures[1],
            &race.physics.damage.hundredth,
            g,
            g,
        );
    });
    raise(&race.physics, &race.shared.car.steering_damage, |g| {
        add_triple_product(
            &race.physics.damage.unknown_114,
            &race.physics.damage.figures[2],
            &race.physics.damage.hundredth,
            g,
            g,
        );
    });
    raise(&race.physics, &race.shared.car.engine_damage, |g| {
        add_triple_product(
            &race.physics.damage.unknown_114,
            &race.physics.damage.figures[3],
            &race.physics.damage.hundredth,
            g,
            g,
        );
    });
    raise(&race.physics, &race.shared.car.exhaust_damage, |g| {
        add_triple_product(
            &race.physics.damage.unknown_114,
            &race.physics.damage.figures[4],
            &race.physics.damage.hundredth,
            g,
            g,
        );
    });
    raise(&race.physics, &race.shared.car.gear_wear[0], |g| {
        add_triple_product(
            &race.physics.damage.unknown_114,
            &race.physics.damage.figures[5],
            &race.physics.damage.hundredth,
            g,
            g,
        );
    });
    for k in 2..9 {
        raise(&race.physics, &race.shared.car.gear_wear[k], |g| {
            add_triple_product(
                &race.physics.damage.unknown_114,
                &race.physics.damage.figures[5],
                &race.physics.damage.hundredth,
                g,
                g,
            );
        });
    }
    raise(&race.physics, &race.shared.car.differential_damage, |g| {
        add_triple_product(
            &race.physics.damage.unknown_114,
            &race.physics.damage.figures[6],
            &race.physics.damage.hundredth,
            g,
            g,
        );
    });
    if !below_or_unordered(
        race.physics.damage.hit_size.bits(),
        race.physics.damage.gauge_rates[23].bits(),
    ) {
        raise(&race.physics, &race.shared.car.body_hit_damage, |g| {
            add_product(
                &race.physics.damage.hit_size,
                &race.physics.damage.figures[13],
                g,
                g,
            );
        });
    }
    if !below_or_unordered(
        race.physics.damage.hit_size.bits(),
        race.physics.damage.gauge_rates[22].bits(),
    ) {
        side_hit_wear(race);
    }
    for k in 0..4 {
        raise(&race.physics, &race.shared.car.suspension_damage[k], |g| {
            add_triple_product(
                &race.physics.damage.wheel_loads[k],
                &race.physics.damage.figures[14],
                &race.physics.damage.hundredth,
                g,
                g,
            );
        });
    }
    raise(&race.physics, &race.shared.car.exhaust_damage, |g| {
        add_triple_product(
            &race.physics.damage.wheel_load_mean,
            &race.physics.damage.figures[15],
            &race.physics.damage.hundredth,
            g,
            g,
        );
    });
    raise(&race.physics, &race.shared.car.steering_damage, |g| {
        add_triple_product(
            &race.physics.damage.wheel_load_mean,
            &race.physics.damage.figures[16],
            &race.physics.damage.hundredth,
            g,
            g,
        );
    });
    gauge_totals(&race.shared, &race.physics.damage);
    if race.physics.damage.drive_kind.bits() == 0 {
        race.shared.car.turbo_damage.set_bits(0x0);
    }
}

/// A hit big enough to count (its size at least `gauge_rates[22]`) wears the gauges of the
/// side it landed on: `side` is 0 .. 3 for the hit figure 1 .. 4, and none for any other.
fn side_hit_wear(race: &Race) {
    scale_hit_size(&race.shared, &race.physics.damage);
    let hit = race.physics.damage.hit_side.bits();
    let side = (1..=4).contains(&hit).then(|| hit as usize - 1);
    let wear = |gauge: &Float, figure: usize| {
        raise(&race.physics, gauge, |g| {
            add_product(
                &race.physics.damage.hit_size,
                &race.physics.damage.figures[figure],
                g,
                g,
            );
        });
    };
    if let Some(side) = side {
        wear(&race.shared.car.light_damage[side], 24);
        wear(&race.shared.car.light_damage[4 + side / 2], 25);
    }
    wear(&race.shared.car.steering_damage, 23);
    wear(&race.shared.car.electrics_damage, 22);
    wear(&race.shared.car.turbo_damage, 17);
    wear(&race.shared.car.clutch_damage, 18);
    wear(&race.shared.car.differential_damage, 19);
    for gauge in &race.shared.car.gear_wear {
        wear(gauge, 20);
    }
    let gauge = gear_gauge(&race.shared, race.physics.damage.gear.bits());
    add_hit_to_gear_gauge(&race.physics.damage, gauge);
    hold_at(&race.physics, gauge);
    if let Some(side) = side {
        wear(&race.shared.car.wheel_hit_damage[side], 28);
        wear(&race.shared.car.suspension_damage[side], 30);
    }
    // The front hits (1 and 2) reach the cooling, the engine and the first brake wear,
    // the rear hits (3 and 4) the second brake wear.
    if (1..=2).contains(&hit) {
        wear(&race.shared.car.cooling_damage, 27);
        wear(&race.shared.car.engine_damage, 26);
        wear(&race.shared.car.brake_wear_a, 29);
    }
    if (3..=4).contains(&hit) {
        wear(&race.shared.car.brake_wear_b, 29);
    }
}

/// Applies `step` to the gauge, then holds it 0 ..= `gauge_top`.
pub(super) fn raise(physics: &Physics, gauge: &Float, step: impl FnOnce(&Float)) {
    step(gauge);
    hold_at(physics, gauge);
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o3] = ([m + o0] * [m + o1]) + [rc + o2]`
fn add_product(
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
fn add_scaled_triple(
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
    let t5 = ld(m.gauge_step.bits());
    let t6 = t4 * t5;
    let t7 = ld(o4.bits());
    let t8 = t6 + t7;
    o5.set_bits(st(t8));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[v + 0] = ((([m + 0x35B398] * [m + 0x35B5A8]) * [m + 0x35B370]) * [t + 0]) + [v + 0]`
fn gear_wear_from_speed(m: &DamageModel, t: &impl Cell32, v: &impl Cell32) {
    let t0 = ld(m.engine_speed_share.bits());
    let t1 = ld(m.wear_rates[12].bits());
    let t2 = t0 * t1;
    let t3 = ld(m.hundredth.bits());
    let t4 = t2 * t3;
    let t5 = ld(t.bits());
    let t6 = t4 * t5;
    let t7 = ld(v.bits());
    let t8 = t6 + t7;
    v.set_bits(st(t8));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B43C] = 1 - [m + 0x35B400]`
/// `[rc + 0x31A320] = ((([m + 0x35B5F4] * [m + 0x35B400]) * [m + 0x35B3B0]) * [m + 0x35B38C]) + [rc + 0x31A320]`
fn brake_wear_step(shared: &Shared, m: &DamageModel) {
    let t0 = X(1.0);
    let t1 = ld(m.brake_balance_share.bits());
    let t2 = t0 - t1;
    m.unknown_fc.set_bits(st(t2));
    let t3 = ld(m.gauge_rates[9].bits());
    let t4 = ld(m.brake_balance_share.bits());
    let t5 = t3 * t4;
    let t6 = ld(m.brake_share.bits());
    let t7 = t5 * t6;
    let t8 = ld(m.gauge_step.bits());
    let t9 = t7 * t8;
    let t10 = ld(shared.car.brake_wear_a.bits());
    let t11 = t9 + t10;
    shared.car.brake_wear_a.set_bits(st(t11));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o4] = (([m + o0] * [m + o1]) * [m + o2]) + [rc + o3]`
fn add_triple_product(
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
fn part_hit_figures(m: &DamageModel) {
    let t0 = ld(m.speed_share.bits());
    let t1 = ld(m.wear_figure.bits());
    let t2 = ld(m.metres_per_mile.bits());
    let t3 = t1 * t2;
    let t4 = t0 / t3;
    let t5 = ld(m.gauge_top.bits());
    let t6 = t4 * t5;
    let t7 = ld(m.wheels_down.bits());
    let t8 = t6 * t7;
    m.unknown_148.set_bits(st(t8));
    m.unknown_14c.set_bits(st(t8));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B488] = [m + 0x35B488] * [t + 0]`
/// `[m + 0x35B48C] = [m + 0x35B48C] * [u + 0]`
fn scale_part_hit_figures(m: &DamageModel, t: &impl Cell32, u: &impl Cell32) {
    let t0 = ld(m.unknown_148.bits());
    let t1 = ld(t.bits());
    let t2 = t0 * t1;
    m.unknown_148.set_bits(st(t2));
    let t3 = ld(m.unknown_14c.bits());
    let t4 = ld(u.bits());
    let t5 = t3 * t4;
    m.unknown_14c.set_bits(st(t5));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A290] = (([rc + 0x31A28C] * [m + 0x35B608]) * [m + 0x35B370]) + [rc + 0x31A290]`
fn engine_damage_carry(shared: &Shared, m: &DamageModel) {
    let t0 = ld(shared.car.engine_heat.bits());
    let t1 = ld(m.gauge_rates[14].bits());
    let t2 = t0 * t1;
    let t3 = ld(m.hundredth.bits());
    let t4 = t2 * t3;
    let t5 = ld(shared.car.engine_damage.bits());
    let t6 = t4 + t5;
    shared.car.engine_damage.set_bits(st(t6));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A2A0] = [rc + 0x31A2A0] + [m + o1]`
fn add_rate_to_clutch(shared: &Shared, o1: &impl Cell32) {
    let t0 = ld(shared.car.clutch_damage.bits());
    let t1 = ld(o1.bits());
    let t2 = t0 + t1;
    shared.car.clutch_damage.set_bits(st(t2));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[v + 0] = [v + 0] + [m + o1]`
fn add_rate_to_gauge(v: &impl Cell32, o1: &impl Cell32) {
    let t0 = ld(v.bits());
    let t1 = ld(o1.bits());
    let t2 = t0 + t1;
    v.set_bits(st(t2));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B3C8] = [m + 0x35B3C8] * [rc + 0x31C708]`
fn scale_hit_size(shared: &Shared, m: &DamageModel) {
    let t0 = ld(m.hit_size.bits());
    let t1 = ld(shared.work.patch_factors[1].bits());
    let t2 = t0 * t1;
    m.hit_size.set_bits(st(t2));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[v + 0] = ([m + 0x35B3C8] * [m + 0x35B6CC]) + [v + 0]`
fn add_hit_to_gear_gauge(m: &DamageModel, v: &impl Cell32) {
    let t0 = ld(m.hit_size.bits());
    let t1 = ld(m.figures[21].bits());
    let t2 = t0 * t1;
    let t3 = ld(v.bits());
    let t4 = t2 + t3;
    v.set_bits(st(t4));
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
fn gauge_totals(shared: &Shared, m: &DamageModel) {
    let t0 = ld(shared.car.light_damage[0].bits());
    let t1 = ld(shared.car.light_damage[1].bits());
    let t2 = t0 + t1;
    let t3 = ld(shared.car.light_damage[2].bits());
    let t4 = t2 + t3;
    let t5 = ld(shared.car.light_damage[3].bits());
    let t6 = t4 + t5;
    let t7 = ld(shared.car.light_damage[5].bits());
    let t8 = t6 + t7;
    let t9 = ld(shared.car.light_damage[4].bits());
    let t10 = t8 + t9;
    let t11 = ld(m.light_count_divisor.bits());
    let t12 = t10 / t11;
    shared.car.lights_damage.set_bits(st(t12));
    let t13 = ld(shared.car.brake_wear_a.bits());
    let t14 = ld(shared.car.brake_wear_b.bits());
    let t15 = t13 + t14;
    let t16 = ld(m.half.bits());
    let t17 = t15 * t16;
    shared.car.brake_damage.set_bits(st(t17));
    let t18 = ld(shared.car.part_hit_damage[0].bits());
    let t19 = ld(shared.car.part_hit_damage[1].bits());
    let t20 = t18 + t19;
    let t21 = ld(m.half.bits());
    let t22 = t20 * t21;
    m.unknown_100.set_bits(st(t22));
    let t23 = ld(shared.car.part_hit_damage[2].bits());
    let t24 = ld(shared.car.part_hit_damage[3].bits());
    let t25 = t23 + t24;
    let t26 = ld(m.half.bits());
    let t27 = t25 * t26;
    m.unknown_104.set_bits(st(t27));
    let t28 = ld(m.unknown_100.bits());
    let t29 = ld(m.unknown_104.bits());
    let t30 = t28 + t29;
    let t31 = ld(m.half.bits());
    let t32 = t30 * t31;
    shared.car.tyre_wear.set_bits(st(t32));
    let t33 = ld(shared.car.gear_wear[0].bits());
    let t34 = ld(shared.car.gear_wear[2].bits());
    let t35 = t33 + t34;
    let t36 = ld(shared.car.gear_wear[3].bits());
    let t37 = t35 + t36;
    let t38 = ld(shared.car.gear_wear[4].bits());
    let t39 = t37 + t38;
    let t40 = ld(shared.car.gear_wear[5].bits());
    let t41 = t39 + t40;
    let t42 = ld(shared.car.gear_wear[6].bits());
    let t43 = t41 + t42;
    let t44 = ld(shared.car.gear_wear[7].bits());
    let t45 = t43 + t44;
    let t46 = ld(shared.car.gear_wear[8].bits());
    let t47 = t45 + t46;
    let t48 = ld(m.half.bits());
    let t49 = t47 * t48;
    let t50 = ld(m.half.bits());
    let t51 = t49 * t50;
    let t52 = ld(m.half.bits());
    let t53 = t51 * t52;
    shared.car.gearbox_damage.set_bits(st(t53));
    let t54 = ld(shared.car.wheel_hit_damage[0].bits());
    let t55 = ld(shared.car.wheel_hit_damage[1].bits());
    let t56 = t54 + t55;
    let t57 = ld(shared.car.wheel_hit_damage[2].bits());
    let t58 = t56 + t57;
    let t59 = ld(shared.car.wheel_hit_damage[3].bits());
    let t60 = t58 + t59;
    let t61 = ld(shared.car.body_hit_damage.bits());
    let t62 = t60 + t61;
    let t63 = ld(shared.car.body_hit_damage.bits());
    let t64 = t62 + t63;
    let t65 = ld(shared.car.body_hit_damage.bits());
    let t66 = t64 + t65;
    let t67 = ld(shared.car.body_hit_damage.bits());
    let t68 = t66 + t67;
    let t69 = ld(shared.car.body_hit_damage.bits());
    let t70 = t68 + t69;
    let t71 = ld(shared.car.body_hit_damage.bits());
    let t72 = t70 + t71;
    let t73 = ld(m.tenth.bits());
    let t74 = t72 * t73;
    shared.car.body_damage.set_bits(st(t74));
    let t75 = ld(shared.car.suspension_damage[0].bits());
    let t76 = ld(shared.car.suspension_damage[1].bits());
    let t77 = t75 + t76;
    let t78 = ld(shared.car.suspension_damage[2].bits());
    let t79 = t77 + t78;
    let t80 = ld(shared.car.suspension_damage[3].bits());
    let t81 = t79 + t80;
    let t82 = ld(m.quarter.bits());
    let t83 = t81 * t82;
    shared.car.overall_suspension_damage.set_bits(st(t83));
    let t84 = ld(shared.car.wheel_stiffness[0].bits());
    let t85 = ld(shared.car.wheel_stiffness[1].bits());
    let t86 = t84 + t85;
    let t87 = ld(shared.car.wheel_stiffness[2].bits());
    let t88 = t86 + t87;
    let t89 = ld(shared.car.wheel_stiffness[3].bits());
    let t90 = t88 + t89;
    let t91 = ld(m.quarter.bits());
    let t92 = t90 * t91;
    shared.car.stiffness_mean.set_bits(st(t92));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::race::owner::race;
    use crate::scene::x87_mem::HostCell;

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
                    add_product(
                        &dm(a[0]).gauge_rates[0],
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.engine_damage,
                        &crate::race::owner::race().shared.car.engine_damage,
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
                    add_product(
                        &dm(a[0]).gauge_rates[1],
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.exhaust_damage,
                        &crate::race::owner::race().shared.car.exhaust_damage,
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
                    add_product(
                        &dm(a[0]).gauge_rates[2],
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.cooling_damage,
                        &crate::race::owner::race().shared.car.cooling_damage,
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
                    add_product(
                        &dm(a[0]).gauge_rates[3],
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.clutch_damage,
                        &crate::race::owner::race().shared.car.clutch_damage,
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
                    add_product(
                        &dm(a[0]).gauge_rates[4],
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.differential_damage,
                        &crate::race::owner::race().shared.car.differential_damage,
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
                    add_product(
                        &dm(a[0]).gauge_rates[5],
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.electrics_damage,
                        &crate::race::owner::race().shared.car.electrics_damage,
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
                    add_product(
                        &dm(a[0]).gauge_rates[6],
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.light_damage[0],
                        &crate::race::owner::race().shared.car.light_damage[0],
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
                    add_product(
                        &dm(a[0]).gauge_rates[6],
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.light_damage[1],
                        &crate::race::owner::race().shared.car.light_damage[1],
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
                    add_product(
                        &dm(a[0]).gauge_rates[6],
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.light_damage[2],
                        &crate::race::owner::race().shared.car.light_damage[2],
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
                    add_product(
                        &dm(a[0]).gauge_rates[6],
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.light_damage[3],
                        &crate::race::owner::race().shared.car.light_damage[3],
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
                    add_product(
                        &dm(a[0]).gauge_rates[6],
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.light_damage[4],
                        &crate::race::owner::race().shared.car.light_damage[4],
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
                    add_product(
                        &dm(a[0]).gauge_rates[6],
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.light_damage[5],
                        &crate::race::owner::race().shared.car.light_damage[5],
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
                    add_product(
                        &dm(a[0]).gauge_rates[7],
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.steering_damage,
                        &crate::race::owner::race().shared.car.steering_damage,
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
                    add_product(
                        &dm(a[0]).gauge_rates[8],
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.suspension_damage[0],
                        &crate::race::owner::race().shared.car.suspension_damage[0],
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
                    add_product(
                        &dm(a[0]).gauge_rates[8],
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.suspension_damage[1],
                        &crate::race::owner::race().shared.car.suspension_damage[1],
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
                    add_product(
                        &dm(a[0]).gauge_rates[8],
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.suspension_damage[2],
                        &crate::race::owner::race().shared.car.suspension_damage[2],
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
                    add_product(
                        &dm(a[0]).gauge_rates[8],
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.suspension_damage[3],
                        &crate::race::owner::race().shared.car.suspension_damage[3],
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
                    add_scaled_triple(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[11],
                        &dm(a[0]).engine_over_limit,
                        &dm(a[0]).engine_speed_share,
                        &crate::race::owner::race().shared.car.turbo_damage,
                        &crate::race::owner::race().shared.car.turbo_damage,
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
                plain: |a| gear_wear_from_speed(dm(a[0]), &HostCell(a[1]), &HostCell(a[2])),
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
                plain: |a| brake_wear_step(&race().shared, dm(a[0])),
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
                    add_scaled_triple(
                        dm(a[0]),
                        &dm(a[0]).gauge_rates[9],
                        &dm(a[0]).unknown_fc,
                        &dm(a[0]).brake_share,
                        &crate::race::owner::race().shared.car.brake_wear_b,
                        &crate::race::owner::race().shared.car.brake_wear_b,
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
                    add_triple_product(
                        &dm(a[0]).gauge_rates[10],
                        &dm(a[0]).on_surface_5,
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.electrics_damage,
                        &crate::race::owner::race().shared.car.electrics_damage,
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
                    add_triple_product(
                        &dm(a[0]).gauge_rates[11],
                        &dm(a[0]).on_surface_5,
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.light_damage[0],
                        &crate::race::owner::race().shared.car.light_damage[0],
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
                    add_triple_product(
                        &dm(a[0]).gauge_rates[11],
                        &dm(a[0]).on_surface_5,
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.light_damage[1],
                        &crate::race::owner::race().shared.car.light_damage[1],
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
                    add_triple_product(
                        &dm(a[0]).gauge_rates[11],
                        &dm(a[0]).on_surface_5,
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.light_damage[2],
                        &crate::race::owner::race().shared.car.light_damage[2],
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
                    add_triple_product(
                        &dm(a[0]).gauge_rates[11],
                        &dm(a[0]).on_surface_5,
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.light_damage[3],
                        &crate::race::owner::race().shared.car.light_damage[3],
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
                    add_triple_product(
                        &dm(a[0]).gauge_rates[11],
                        &dm(a[0]).on_surface_5,
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.light_damage[4],
                        &crate::race::owner::race().shared.car.light_damage[4],
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
                    add_triple_product(
                        &dm(a[0]).gauge_rates[11],
                        &dm(a[0]).on_surface_5,
                        &dm(a[0]).gauge_step,
                        &crate::race::owner::race().shared.car.light_damage[5],
                        &crate::race::owner::race().shared.car.light_damage[5],
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
                plain: |a| part_hit_figures(dm(a[0])),
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
                plain: |a| scale_part_hit_figures(dm(a[0]), &HostCell(a[1]), &HostCell(a[2])),
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
                    add_triple_product(
                        &dm(a[0]).unknown_12c,
                        &dm(a[0]).unknown_148,
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.part_hit_damage[0],
                        &crate::race::owner::race().shared.car.part_hit_damage[0],
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
                    add_triple_product(
                        &dm(a[0]).unknown_12c,
                        &dm(a[0]).unknown_148,
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.part_hit_damage[1],
                        &crate::race::owner::race().shared.car.part_hit_damage[1],
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
                    add_triple_product(
                        &dm(a[0]).unknown_130,
                        &dm(a[0]).unknown_14c,
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.part_hit_damage[2],
                        &crate::race::owner::race().shared.car.part_hit_damage[2],
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
                    add_triple_product(
                        &dm(a[0]).unknown_130,
                        &dm(a[0]).unknown_14c,
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.part_hit_damage[3],
                        &crate::race::owner::race().shared.car.part_hit_damage[3],
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
                    add_triple_product(
                        &dm(a[0]).unknown_11c,
                        &dm(a[0]).gauge_rates[13],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.engine_damage,
                        &crate::race::owner::race().shared.car.engine_damage,
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
                    add_triple_product(
                        &dm(a[0]).unknown_11c,
                        &dm(a[0]).gauge_rates[12],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.cooling_damage,
                        &crate::race::owner::race().shared.car.cooling_damage,
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
                plain: |a| engine_damage_carry(&race().shared, dm(a[1])),
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
                    add_triple_product(
                        &dm(a[0]).unknown_124,
                        &dm(a[0]).gauge_rates[15],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.cooling_damage,
                        &crate::race::owner::race().shared.car.cooling_damage,
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
                plain: |a| add_rate_to_clutch(&race().shared, &dm(a[1]).gauge_rates[16]),
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
                plain: |a| add_rate_to_clutch(&race().shared, &dm(a[1]).gauge_rates[17]),
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
                plain: |a| add_rate_to_gauge(&HostCell(a[0]), &dm(a[1]).gauge_rates[18]),
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
                plain: |a| add_rate_to_gauge(&HostCell(a[0]), &dm(a[1]).gauge_rates[19]),
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
                    add_triple_product(
                        &dm(a[0]).unknown_110,
                        &dm(a[0]).unknown_330,
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.wheel_hit_damage[0],
                        &crate::race::owner::race().shared.car.wheel_hit_damage[0],
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
                    add_triple_product(
                        &dm(a[0]).unknown_110,
                        &dm(a[0]).unknown_330,
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.wheel_hit_damage[1],
                        &crate::race::owner::race().shared.car.wheel_hit_damage[1],
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
                    add_triple_product(
                        &dm(a[0]).unknown_110,
                        &dm(a[0]).unknown_330,
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.wheel_hit_damage[2],
                        &crate::race::owner::race().shared.car.wheel_hit_damage[2],
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
                    add_triple_product(
                        &dm(a[0]).unknown_110,
                        &dm(a[0]).unknown_330,
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.wheel_hit_damage[3],
                        &crate::race::owner::race().shared.car.wheel_hit_damage[3],
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
                    add_triple_product(
                        &dm(a[0]).unknown_118,
                        &dm(a[0]).figures[0],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.suspension_damage[0],
                        &crate::race::owner::race().shared.car.suspension_damage[0],
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
                    add_triple_product(
                        &dm(a[0]).unknown_118,
                        &dm(a[0]).figures[0],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.suspension_damage[1],
                        &crate::race::owner::race().shared.car.suspension_damage[1],
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
                    add_triple_product(
                        &dm(a[0]).unknown_118,
                        &dm(a[0]).figures[0],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.suspension_damage[2],
                        &crate::race::owner::race().shared.car.suspension_damage[2],
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
                    add_triple_product(
                        &dm(a[0]).unknown_118,
                        &dm(a[0]).figures[0],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.suspension_damage[3],
                        &crate::race::owner::race().shared.car.suspension_damage[3],
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
                    add_triple_product(
                        &dm(a[0]).unknown_114,
                        &dm(a[0]).figures[1],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.body_hit_damage,
                        &crate::race::owner::race().shared.car.body_hit_damage,
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
                    add_triple_product(
                        &dm(a[0]).unknown_114,
                        &dm(a[0]).figures[2],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.steering_damage,
                        &crate::race::owner::race().shared.car.steering_damage,
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
                    add_triple_product(
                        &dm(a[0]).unknown_114,
                        &dm(a[0]).figures[3],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.engine_damage,
                        &crate::race::owner::race().shared.car.engine_damage,
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
                    add_triple_product(
                        &dm(a[0]).unknown_114,
                        &dm(a[0]).figures[4],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.exhaust_damage,
                        &crate::race::owner::race().shared.car.exhaust_damage,
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
                    add_triple_product(
                        &dm(a[0]).unknown_114,
                        &dm(a[0]).figures[5],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.gear_wear[0],
                        &crate::race::owner::race().shared.car.gear_wear[0],
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
                    add_triple_product(
                        &dm(a[0]).unknown_114,
                        &dm(a[0]).figures[5],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.gear_wear[2],
                        &crate::race::owner::race().shared.car.gear_wear[2],
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
                    add_triple_product(
                        &dm(a[0]).unknown_114,
                        &dm(a[0]).figures[5],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.gear_wear[3],
                        &crate::race::owner::race().shared.car.gear_wear[3],
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
                    add_triple_product(
                        &dm(a[0]).unknown_114,
                        &dm(a[0]).figures[5],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.gear_wear[4],
                        &crate::race::owner::race().shared.car.gear_wear[4],
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
                    add_triple_product(
                        &dm(a[0]).unknown_114,
                        &dm(a[0]).figures[5],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.gear_wear[5],
                        &crate::race::owner::race().shared.car.gear_wear[5],
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
                    add_triple_product(
                        &dm(a[0]).unknown_114,
                        &dm(a[0]).figures[5],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.gear_wear[6],
                        &crate::race::owner::race().shared.car.gear_wear[6],
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
                    add_triple_product(
                        &dm(a[0]).unknown_114,
                        &dm(a[0]).figures[5],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.gear_wear[7],
                        &crate::race::owner::race().shared.car.gear_wear[7],
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
                    add_triple_product(
                        &dm(a[0]).unknown_114,
                        &dm(a[0]).figures[5],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.gear_wear[8],
                        &crate::race::owner::race().shared.car.gear_wear[8],
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
                    add_triple_product(
                        &dm(a[0]).unknown_114,
                        &dm(a[0]).figures[6],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.differential_damage,
                        &crate::race::owner::race().shared.car.differential_damage,
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[13],
                        &crate::race::owner::race().shared.car.body_hit_damage,
                        &crate::race::owner::race().shared.car.body_hit_damage,
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
                plain: |a| scale_hit_size(&race().shared, dm(a[0])),
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[24],
                        &crate::race::owner::race().shared.car.light_damage[0],
                        &crate::race::owner::race().shared.car.light_damage[0],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[24],
                        &crate::race::owner::race().shared.car.light_damage[1],
                        &crate::race::owner::race().shared.car.light_damage[1],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[24],
                        &crate::race::owner::race().shared.car.light_damage[2],
                        &crate::race::owner::race().shared.car.light_damage[2],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[24],
                        &crate::race::owner::race().shared.car.light_damage[3],
                        &crate::race::owner::race().shared.car.light_damage[3],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[25],
                        &crate::race::owner::race().shared.car.light_damage[4],
                        &crate::race::owner::race().shared.car.light_damage[4],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[25],
                        &crate::race::owner::race().shared.car.light_damage[4],
                        &crate::race::owner::race().shared.car.light_damage[4],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[25],
                        &crate::race::owner::race().shared.car.light_damage[5],
                        &crate::race::owner::race().shared.car.light_damage[5],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[25],
                        &crate::race::owner::race().shared.car.light_damage[5],
                        &crate::race::owner::race().shared.car.light_damage[5],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[23],
                        &crate::race::owner::race().shared.car.steering_damage,
                        &crate::race::owner::race().shared.car.steering_damage,
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[22],
                        &crate::race::owner::race().shared.car.electrics_damage,
                        &crate::race::owner::race().shared.car.electrics_damage,
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[17],
                        &crate::race::owner::race().shared.car.turbo_damage,
                        &crate::race::owner::race().shared.car.turbo_damage,
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[18],
                        &crate::race::owner::race().shared.car.clutch_damage,
                        &crate::race::owner::race().shared.car.clutch_damage,
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[19],
                        &crate::race::owner::race().shared.car.differential_damage,
                        &crate::race::owner::race().shared.car.differential_damage,
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[20],
                        &crate::race::owner::race().shared.car.gear_wear[0],
                        &crate::race::owner::race().shared.car.gear_wear[0],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[20],
                        &crate::race::owner::race().shared.car.gear_wear[1],
                        &crate::race::owner::race().shared.car.gear_wear[1],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[20],
                        &crate::race::owner::race().shared.car.gear_wear[2],
                        &crate::race::owner::race().shared.car.gear_wear[2],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[20],
                        &crate::race::owner::race().shared.car.gear_wear[3],
                        &crate::race::owner::race().shared.car.gear_wear[3],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[20],
                        &crate::race::owner::race().shared.car.gear_wear[4],
                        &crate::race::owner::race().shared.car.gear_wear[4],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[20],
                        &crate::race::owner::race().shared.car.gear_wear[5],
                        &crate::race::owner::race().shared.car.gear_wear[5],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[20],
                        &crate::race::owner::race().shared.car.gear_wear[6],
                        &crate::race::owner::race().shared.car.gear_wear[6],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[20],
                        &crate::race::owner::race().shared.car.gear_wear[7],
                        &crate::race::owner::race().shared.car.gear_wear[7],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[20],
                        &crate::race::owner::race().shared.car.gear_wear[8],
                        &crate::race::owner::race().shared.car.gear_wear[8],
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
                plain: |a| add_hit_to_gear_gauge(dm(a[0]), &HostCell(a[1])),
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[28],
                        &crate::race::owner::race().shared.car.wheel_hit_damage[0],
                        &crate::race::owner::race().shared.car.wheel_hit_damage[0],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[28],
                        &crate::race::owner::race().shared.car.wheel_hit_damage[1],
                        &crate::race::owner::race().shared.car.wheel_hit_damage[1],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[28],
                        &crate::race::owner::race().shared.car.wheel_hit_damage[2],
                        &crate::race::owner::race().shared.car.wheel_hit_damage[2],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[28],
                        &crate::race::owner::race().shared.car.wheel_hit_damage[3],
                        &crate::race::owner::race().shared.car.wheel_hit_damage[3],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[30],
                        &crate::race::owner::race().shared.car.suspension_damage[0],
                        &crate::race::owner::race().shared.car.suspension_damage[0],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[30],
                        &crate::race::owner::race().shared.car.suspension_damage[1],
                        &crate::race::owner::race().shared.car.suspension_damage[1],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[30],
                        &crate::race::owner::race().shared.car.suspension_damage[2],
                        &crate::race::owner::race().shared.car.suspension_damage[2],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[30],
                        &crate::race::owner::race().shared.car.suspension_damage[3],
                        &crate::race::owner::race().shared.car.suspension_damage[3],
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[27],
                        &crate::race::owner::race().shared.car.cooling_damage,
                        &crate::race::owner::race().shared.car.cooling_damage,
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[27],
                        &crate::race::owner::race().shared.car.cooling_damage,
                        &crate::race::owner::race().shared.car.cooling_damage,
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[26],
                        &crate::race::owner::race().shared.car.engine_damage,
                        &crate::race::owner::race().shared.car.engine_damage,
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[26],
                        &crate::race::owner::race().shared.car.engine_damage,
                        &crate::race::owner::race().shared.car.engine_damage,
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[29],
                        &crate::race::owner::race().shared.car.brake_wear_a,
                        &crate::race::owner::race().shared.car.brake_wear_a,
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[29],
                        &crate::race::owner::race().shared.car.brake_wear_a,
                        &crate::race::owner::race().shared.car.brake_wear_a,
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[29],
                        &crate::race::owner::race().shared.car.brake_wear_b,
                        &crate::race::owner::race().shared.car.brake_wear_b,
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
                    add_product(
                        &dm(a[0]).hit_size,
                        &dm(a[0]).figures[29],
                        &crate::race::owner::race().shared.car.brake_wear_b,
                        &crate::race::owner::race().shared.car.brake_wear_b,
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
                    add_triple_product(
                        &dm(a[0]).wheel_loads[0],
                        &dm(a[0]).figures[14],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.suspension_damage[0],
                        &crate::race::owner::race().shared.car.suspension_damage[0],
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
                    add_triple_product(
                        &dm(a[0]).wheel_loads[1],
                        &dm(a[0]).figures[14],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.suspension_damage[1],
                        &crate::race::owner::race().shared.car.suspension_damage[1],
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
                    add_triple_product(
                        &dm(a[0]).wheel_loads[2],
                        &dm(a[0]).figures[14],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.suspension_damage[2],
                        &crate::race::owner::race().shared.car.suspension_damage[2],
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
                    add_triple_product(
                        &dm(a[0]).wheel_loads[3],
                        &dm(a[0]).figures[14],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.suspension_damage[3],
                        &crate::race::owner::race().shared.car.suspension_damage[3],
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
                    add_triple_product(
                        &dm(a[0]).wheel_load_mean,
                        &dm(a[0]).figures[15],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.exhaust_damage,
                        &crate::race::owner::race().shared.car.exhaust_damage,
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
                    add_triple_product(
                        &dm(a[0]).wheel_load_mean,
                        &dm(a[0]).figures[16],
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.steering_damage,
                        &crate::race::owner::race().shared.car.steering_damage,
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
                plain: |a| gauge_totals(&race().shared, dm(a[1])),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
