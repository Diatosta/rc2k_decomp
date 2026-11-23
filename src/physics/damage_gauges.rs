//! The damage gauges' step (`physics/damage_model.rs`).
//!
//! Each gauge of the car block (engine, exhaust, cooling, clutch, differential, electrics,
//! lights, steering, suspension, turbo, brakes, the gearbox's gears, the body and part hits) is
//! raised by its rate (`gauge_rates`, `wear_rates`) times this step's shares and hit figures,
//! then held 0 ..= `gauge_top`. The hit's side (`DamageModel.hit_side`, 1 .. 4) chooses the
//! gauges a hit wears, its size (`DamageModel.hit_size`) is scaled past `gauge_rates[22]`, and
//! the direction `DamageModel.gear_changed` (1 or -1) adds fixed steps.

use crate::errors::BadInput;
use crate::physics::car_store::Car;
use crate::physics::damage_model::{figure, store, update, value};
use crate::physics::flow::{below_or_unordered, c0};
use crate::physics::store::DamageModel;
use crate::race::work_store::RaceWork;

/// `Car.gear_wear[1][gear]` (the gear is -1 .. 7; `gear_wear` is indexed by gear + 1).
fn gear_gauge(car: &mut Car, gear: i32) -> &mut f32 {
    let at = gear.wrapping_add(1) as usize;
    match car.gear_wear.get_mut(at) {
        Some(gauge) => gauge,
        None => {
            crate::fatal::raise::end("a gear's wear gauge", BadInput("the gear is not -1 .. 7"))
        }
    }
}

/// The gauge at `at` held 0 ..= `DamageModel.gauge_top`.
fn hold_at(damage: &DamageModel, at: &mut f32) {
    if c0(*at, damage.gauge_low) {
        store(at, figure(damage.gauge_low));
    }
    if !c0(*at, damage.gauge_top) {
        store(at, figure(damage.gauge_top));
    }
}

/// The damage gauges' step: each gauge raised by its rate times this step's shares and hit
/// figures, then held 0 ..= `gauge_top`.
pub fn steps(car: &mut Car, damage: &mut DamageModel, work: &RaceWork) {
    raise(damage, &mut car.engine_damage, |g| {
        add_product(damage.gauge_rates[0], damage.gauge_step, g)
    });
    raise(damage, &mut car.exhaust_damage, |g| {
        add_product(damage.gauge_rates[1], damage.gauge_step, g)
    });
    raise(damage, &mut car.cooling_damage, |g| {
        add_product(damage.gauge_rates[2], damage.gauge_step, g)
    });
    raise(damage, &mut car.clutch_damage, |g| {
        add_product(damage.gauge_rates[3], damage.gauge_step, g)
    });
    raise(damage, &mut car.differential_damage, |g| {
        add_product(damage.gauge_rates[4], damage.gauge_step, g)
    });
    raise(damage, &mut car.electrics_damage, |g| {
        add_product(damage.gauge_rates[5], damage.gauge_step, g)
    });
    for item in car.light_damage.iter_mut().take(6) {
        raise(damage, item, |g| {
            add_product(damage.gauge_rates[6], damage.gauge_step, g)
        });
    }
    raise(damage, &mut car.steering_damage, |g| {
        add_product(damage.gauge_rates[7], damage.gauge_step, g)
    });
    for item in car.suspension_damage.iter_mut().take(4) {
        raise(damage, item, |g| {
            add_product(damage.gauge_rates[8], damage.gauge_step, g)
        });
    }
    raise(damage, &mut car.turbo_damage, |g| {
        add_scaled_triple(
            damage,
            damage.wear_rates[11],
            damage.engine_over_limit,
            damage.engine_speed_share,
            g,
        )
    });
    let gear = damage.gear;
    let gauge = gear_gauge(car, gear);
    let factor = *damage
        .gear_factors
        .get(gear.wrapping_add(1) as usize)
        .unwrap_or_else(|| {
            crate::fatal::raise::end("a gear's factor", BadInput("the gear is not -1 .. 7"))
        });
    gear_wear_from_speed(damage, factor, gauge);
    hold_at(damage, gauge);
    brake_wear_step(car, damage);
    hold_at(damage, &mut car.brake_wear_a);
    raise(damage, &mut car.brake_wear_b, |g| {
        add_scaled_triple(
            damage,
            damage.gauge_rates[9],
            damage.brake_balance_complement,
            damage.brake_share,
            g,
        )
    });
    raise(damage, &mut car.electrics_damage, |g| {
        add_triple_product(
            damage.gauge_rates[10],
            damage.on_surface_5,
            damage.gauge_step,
            g,
        )
    });
    for item in car.light_damage.iter_mut().take(6) {
        raise(damage, item, |g| {
            add_triple_product(
                damage.gauge_rates[11],
                damage.on_surface_5,
                damage.gauge_step,
                g,
            )
        });
    }
    part_hit_figures(damage);
    let part = damage.drive_kind;
    fn kind(table: &[f32; 2], part: u32) -> f32 {
        *table.get(part as usize).unwrap_or_else(|| {
            crate::fatal::raise::end("a kind table", BadInput("the kind is not 0 or 1"))
        })
    }
    let (front, rear) = (
        kind(&damage.kind_table_a, part),
        kind(&damage.kind_table_b, part),
    );
    scale_part_hit_figures(damage, front, rear);
    for item in car.tyre_wears.iter_mut().take(2) {
        raise(damage, item, |g| {
            add_triple_product(
                damage.front_slip_wear_factor,
                damage.front_tyre_wear_rate,
                damage.hundredth,
                g,
            )
        });
    }
    for item in car.tyre_wears.iter_mut().skip(2).take(4 - 2) {
        raise(damage, item, |g| {
            add_triple_product(
                damage.rear_slip_wear_factor,
                damage.rear_tyre_wear_rate,
                damage.hundredth,
                g,
            )
        });
    }
    raise(damage, &mut car.engine_damage, |g| {
        add_triple_product(
            damage.engine_speed_wear_factor,
            damage.gauge_rates[13],
            damage.hundredth,
            g,
        )
    });
    raise(damage, &mut car.cooling_damage, |g| {
        add_triple_product(
            damage.engine_speed_wear_factor,
            damage.gauge_rates[12],
            damage.hundredth,
            g,
        )
    });
    if !below_or_unordered(damage.overheat_excess, damage.gauge_low) {
        engine_damage_carry(car, damage);
        hold_at(damage, &mut car.engine_damage);
        raise(damage, &mut car.cooling_damage, |g| {
            add_triple_product(
                damage.overheat_excess,
                damage.gauge_rates[15],
                damage.hundredth,
                g,
            )
        });
    }
    if damage.gear_changed == 1 {
        add_rate_to_clutch(car, damage.gauge_rates[16]);
    }
    if damage.gear_changed == -1 {
        add_rate_to_clutch(car, damage.gauge_rates[17]);
    }
    let gear = damage.gear;
    if damage.gear_changed == 1 {
        let gauge = gear_gauge(car, gear);
        add_rate_to_gauge(gauge, damage.gauge_rates[18]);
    }
    let gear = damage.gear;
    if damage.gear_changed == -1 {
        let gauge = gear_gauge(car, gear);
        add_rate_to_gauge(gauge, damage.gauge_rates[19]);
    }
    for item in car.wheel_hit_damage.iter_mut().take(4) {
        raise(damage, item, |g| {
            add_triple_product(
                damage.roughness_wear_base,
                damage.wheel_roughness_rate,
                damage.hundredth,
                g,
            )
        });
    }
    for item in car.suspension_damage.iter_mut().take(4) {
        raise(damage, item, |g| {
            add_triple_product(
                damage.suspension_roughness_wear,
                damage.figures[0],
                damage.hundredth,
                g,
            )
        });
    }
    raise(damage, &mut car.body_hit_damage, |g| {
        add_triple_product(
            damage.body_roughness_wear,
            damage.figures[1],
            damage.hundredth,
            g,
        )
    });
    raise(damage, &mut car.steering_damage, |g| {
        add_triple_product(
            damage.body_roughness_wear,
            damage.figures[2],
            damage.hundredth,
            g,
        )
    });
    raise(damage, &mut car.engine_damage, |g| {
        add_triple_product(
            damage.body_roughness_wear,
            damage.figures[3],
            damage.hundredth,
            g,
        )
    });
    raise(damage, &mut car.exhaust_damage, |g| {
        add_triple_product(
            damage.body_roughness_wear,
            damage.figures[4],
            damage.hundredth,
            g,
        )
    });
    raise(damage, &mut car.gear_wear[0], |g| {
        add_triple_product(
            damage.body_roughness_wear,
            damage.figures[5],
            damage.hundredth,
            g,
        )
    });
    for item in car.gear_wear.iter_mut().skip(2).take(9 - 2) {
        raise(damage, item, |g| {
            add_triple_product(
                damage.body_roughness_wear,
                damage.figures[5],
                damage.hundredth,
                g,
            )
        });
    }
    raise(damage, &mut car.differential_damage, |g| {
        add_triple_product(
            damage.body_roughness_wear,
            damage.figures[6],
            damage.hundredth,
            g,
        )
    });
    if !below_or_unordered(damage.hit_size, damage.gauge_rates[23]) {
        raise(damage, &mut car.body_hit_damage, |g| {
            add_product(damage.hit_size, damage.figures[13], g)
        });
    }
    if !below_or_unordered(damage.hit_size, damage.gauge_rates[22]) {
        side_hit_wear(car, damage, work);
    }
    for k in 0..4 {
        raise(damage, &mut car.suspension_damage[k], |g| {
            add_triple_product(
                damage.wheel_loads[k],
                damage.figures[14],
                damage.hundredth,
                g,
            )
        });
    }
    raise(damage, &mut car.exhaust_damage, |g| {
        add_triple_product(
            damage.wheel_load_mean,
            damage.figures[15],
            damage.hundredth,
            g,
        )
    });
    raise(damage, &mut car.steering_damage, |g| {
        add_triple_product(
            damage.wheel_load_mean,
            damage.figures[16],
            damage.hundredth,
            g,
        )
    });
    gauge_totals(car, damage);
    if damage.drive_kind == 0 {
        car.turbo_damage = 0.0;
    }
}

/// One gauge worn by the hit's size times the figure `at`.
fn wear(damage: &DamageModel, gauge: &mut f32, at: usize) {
    raise(damage, gauge, |g| {
        add_product(damage.hit_size, damage.figures[at], g)
    });
}

/// A hit big enough to count (its size at least `gauge_rates[22]`) wears the gauges of the
/// side it landed on: `side` is 0 .. 3 for the hit figure 1 .. 4, and none for any other.
fn side_hit_wear(car: &mut Car, damage: &mut DamageModel, work: &RaceWork) {
    scale_hit_size(work, damage);
    let hit = damage.hit_side;
    let side = (1..=4).contains(&hit).then(|| hit as usize - 1);
    let damage = &*damage;

    if let Some(side) = side {
        wear(damage, &mut car.light_damage[side], 24);
        wear(damage, &mut car.light_damage[4 + side / 2], 25);
    }
    wear(damage, &mut car.steering_damage, 23);
    wear(damage, &mut car.electrics_damage, 22);
    wear(damage, &mut car.turbo_damage, 17);
    wear(damage, &mut car.clutch_damage, 18);
    wear(damage, &mut car.differential_damage, 19);
    for gauge in &mut car.gear_wear {
        wear(damage, gauge, 20);
    }
    let gauge = gear_gauge(car, damage.gear);
    add_hit_to_gear_gauge(damage, gauge);
    hold_at(damage, gauge);
    if let Some(side) = side {
        wear(damage, &mut car.wheel_hit_damage[side], 28);
        wear(damage, &mut car.suspension_damage[side], 30);
    }
    // The front hits (1 and 2) reach the cooling, the engine and the first brake wear,
    // the rear hits (3 and 4) the second brake wear.
    if (1..=2).contains(&hit) {
        wear(damage, &mut car.cooling_damage, 27);
        wear(damage, &mut car.engine_damage, 26);
        wear(damage, &mut car.brake_wear_a, 29);
    }
    if (3..=4).contains(&hit) {
        wear(damage, &mut car.brake_wear_b, 29);
    }
}

/// Applies `step` to the gauge, then holds it 0 ..= `gauge_top`.
pub(super) fn raise(damage: &DamageModel, gauge: &mut f32, step: impl FnOnce(f32) -> f32) {
    *gauge = step(*gauge);
    hold_at(damage, gauge);
}

// ---- the arithmetic ----

/// `rate * step + from`.
fn add_product(rate: f32, step: f32, from: f32) -> f32 {
    (figure(rate) * figure(step) + figure(from)) as f32
}

/// `rate * a * b * gauge_step + from`.
fn add_scaled_triple(m: &DamageModel, rate: f32, a: f32, b: f32, from: f32) -> f32 {
    (figure(rate) * figure(a) * figure(b) * figure(m.gauge_step) + figure(from)) as f32
}

/// The gear's gauge raised by the engine's speed share times the wear rate, a hundredth
/// and the gear's factor.
fn gear_wear_from_speed(m: &DamageModel, factor: f32, gauge: &mut f32) {
    update(gauge, |v| {
        figure(m.engine_speed_share)
            * figure(m.wear_rates[12])
            * figure(m.hundredth)
            * figure(factor)
            + v
    });
}

/// The brake balance's complement, and the first brake wear raised by the rate times the
/// balance, the brake share and the gauge step.
fn brake_wear_step(car: &mut Car, m: &mut DamageModel) {
    m.brake_balance_complement = (1.0 - figure(m.brake_balance_share)) as f32;
    update(&mut car.brake_wear_a, |v| {
        figure(m.gauge_rates[9])
            * figure(m.brake_balance_share)
            * figure(m.brake_share)
            * figure(m.gauge_step)
            + v
    });
}

/// `rate * a * b + from`.
fn add_triple_product(rate: f32, a: f32, b: f32, from: f32) -> f32 {
    (figure(rate) * figure(a) * figure(b) + figure(from)) as f32
}

/// Both tyre wear rates: the speed share over the wear figure times the length of a mile
/// (in metres), as a share of the gauge, times the wheels-down switch.
fn part_hit_figures(m: &mut DamageModel) {
    let rate = (figure(m.speed_share) / (figure(m.wear_figure) * figure(m.metres_per_mile))
        * figure(m.gauge_top)
        * figure(m.wheels_down)) as f32;
    m.front_tyre_wear_rate = rate;
    m.rear_tyre_wear_rate = rate;
}

/// The two tyre wear rates times the given factors.
fn scale_part_hit_figures(m: &mut DamageModel, front: f32, rear: f32) {
    m.front_tyre_wear_rate = (figure(m.front_tyre_wear_rate) * figure(front)) as f32;
    m.rear_tyre_wear_rate = (figure(m.rear_tyre_wear_rate) * figure(rear)) as f32;
}

/// The engine's damage raised by its heat times the rate and a hundredth.
fn engine_damage_carry(car: &mut Car, m: &DamageModel) {
    let heat = value(&car.engine_heat);
    update(&mut car.engine_damage, |v| {
        heat * figure(m.gauge_rates[14]) * figure(m.hundredth) + v
    });
}

/// The clutch's damage plus a rate.
fn add_rate_to_clutch(car: &mut Car, rate: f32) {
    update(&mut car.clutch_damage, |v| v + figure(rate));
}

/// A gauge plus a rate.
fn add_rate_to_gauge(gauge: &mut f32, rate: f32) {
    update(gauge, |v| v + figure(rate));
}

/// The size of the hit times the damage setting.
fn scale_hit_size(work: &RaceWork, m: &mut DamageModel) {
    m.hit_size = (figure(m.hit_size) * f64::from(work.patch_factors[1])) as f32;
}

/// The gear's gauge plus the size of the hit times the gear's hit figure.
fn add_hit_to_gear_gauge(m: &DamageModel, gauge: &mut f32) {
    update(gauge, |v| figure(m.hit_size) * figure(m.figures[21]) + v);
}

/// The sums of the gauges: the lights' mean, the brakes', the tyres', the gearbox's, the
/// body's, the suspension's and the stiffness mean.
fn gauge_totals(car: &mut Car, m: &mut DamageModel) {
    let [l0, l1, l2, l3, l4, l5] = car.light_damage;
    store(
        &mut car.lights_damage,
        (value(&l0) + value(&l1) + value(&l2) + value(&l3) + value(&l5) + value(&l4))
            / figure(m.light_count_divisor),
    );
    let half = figure(m.half);
    store(
        &mut car.brake_damage,
        (value(&car.brake_wear_a) + value(&car.brake_wear_b)) * half,
    );
    let [t0, t1, t2, t3] = car.tyre_wears;
    m.front_tyre_wear = ((value(&t0) + value(&t1)) * half) as f32;
    m.rear_tyre_wear = ((value(&t2) + value(&t3)) * half) as f32;
    store(
        &mut car.tyre_wear,
        (figure(m.front_tyre_wear) + figure(m.rear_tyre_wear)) * half,
    );
    // Every gear's wear but the neutral's (at 1).
    let [g0, _, g2, g3, g4, g5, g6, g7, g8] = car.gear_wear;
    store(
        &mut car.gearbox_damage,
        (value(&g0)
            + value(&g2)
            + value(&g3)
            + value(&g4)
            + value(&g5)
            + value(&g6)
            + value(&g7)
            + value(&g8))
            * half
            * half
            * half,
    );
    let [w0, w1, w2, w3] = car.wheel_hit_damage;
    let body = value(&car.body_hit_damage);
    store(
        &mut car.body_damage,
        (value(&w0)
            + value(&w1)
            + value(&w2)
            + value(&w3)
            + body
            + body
            + body
            + body
            + body
            + body)
            * figure(m.tenth),
    );
    let [s0, s1, s2, s3] = car.suspension_damage;
    store(
        &mut car.overall_suspension_damage,
        (value(&s0) + value(&s1) + value(&s2) + value(&s3)) * figure(m.quarter),
    );
    let [k0, k1, k2, k3] = car.wheel_stiffness;
    store(
        &mut car.stiffness_mean,
        (value(&k0) + value(&k1) + value(&k2) + value(&k3)) * figure(m.quarter),
    );
}

#[cfg(test)]
mod reference_vectors_tests;

#[cfg(test)]
mod plain_matches_x87_tests;
