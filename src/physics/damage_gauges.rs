//! `physics::damage_gauges` (0x4AC817): the damage gauges' step (`physics/damage_model.rs`).
//!
//! Each gauge of the car block (engine, exhaust, cooling, clutch, differential, electrics,
//! lights, steering, suspension, turbo, brakes, the gearbox's gears, the body and part
//! hits) is raised by its rate (`gauge_rates`, `wear_rates`) times this step's shares and
//! hit figures, then held 0 ..= `gauge_top`. The hit's side (`DamageModel.hit_side`, 1 .. 4)
//! chooses the gauges a hit wears, its size (`DamageModel.hit_size`) is scaled past `gauge_rates[22]`,
//! and the direction `DamageModel.gear_changed` (1 or -1) adds fixed steps.
//!
//! Caller: `physics::damage_step`.

use crate::errors::BadInput;
use crate::physics::car_store::Car;
use crate::physics::damage_model::{store, value};
use crate::physics::flow::{below_or_unordered, c0};
use crate::physics::store::DamageModel;
use crate::race::work_store::RaceWork;
use crate::session::{Cell32, Float};

/// `Car.gear_wear[1][gear]` (the gear -1 .. 7 as the original indexes it, `gear_wear` by gear + 1).
fn gear_gauge(car: &Car, gear: u32) -> &Float {
    car.gear_wear
        .get(gear.wrapping_add(1) as usize)
        .unwrap_or_else(|| {
            crate::fatal::raise::end("a gear's wear gauge", BadInput("the gear is not -1 .. 7"))
        })
}

/// The gauge at `at` held 0 ..= `DamageModel.gauge_top` (the two `fcompp` holds).
fn hold_at(damage: &DamageModel, at: &Float) {
    if c0(at.bits(), damage.gauge_low.bits()) {
        store(at, value(&damage.gauge_low));
    }
    if !c0(at.bits(), damage.gauge_top.bits()) {
        store(at, value(&damage.gauge_top));
    }
}

/// The damage gauges' step: each gauge raised by its rate times this step's shares and hit
/// figures, then held 0 ..= `gauge_top`.
pub fn steps(car: &Car, damage: &DamageModel, work: &RaceWork) {
    raise(damage, &car.engine_damage, |g| {
        add_product(&damage.gauge_rates[0], &damage.gauge_step, g, g);
    });
    raise(damage, &car.exhaust_damage, |g| {
        add_product(&damage.gauge_rates[1], &damage.gauge_step, g, g);
    });
    raise(damage, &car.cooling_damage, |g| {
        add_product(&damage.gauge_rates[2], &damage.gauge_step, g, g);
    });
    raise(damage, &car.clutch_damage, |g| {
        add_product(&damage.gauge_rates[3], &damage.gauge_step, g, g);
    });
    raise(damage, &car.differential_damage, |g| {
        add_product(&damage.gauge_rates[4], &damage.gauge_step, g, g);
    });
    raise(damage, &car.electrics_damage, |g| {
        add_product(&damage.gauge_rates[5], &damage.gauge_step, g, g);
    });
    for k in 0..6 {
        raise(damage, &car.light_damage[k], |g| {
            add_product(&damage.gauge_rates[6], &damage.gauge_step, g, g);
        });
    }
    raise(damage, &car.steering_damage, |g| {
        add_product(&damage.gauge_rates[7], &damage.gauge_step, g, g);
    });
    for k in 0..4 {
        raise(damage, &car.suspension_damage[k], |g| {
            add_product(&damage.gauge_rates[8], &damage.gauge_step, g, g);
        });
    }
    raise(damage, &car.turbo_damage, |g| {
        add_scaled_triple(
            damage,
            &damage.wear_rates[11],
            &damage.engine_over_limit,
            &damage.engine_speed_share,
            g,
            g,
        );
    });
    let gear = damage.gear.bits();
    let gauge = gear_gauge(car, gear);
    let factor = damage
        .gear_factors
        .get(gear.wrapping_add(1) as usize)
        .unwrap_or_else(|| {
            crate::fatal::raise::end("a gear's factor", BadInput("the gear is not -1 .. 7"))
        });
    gear_wear_from_speed(damage, factor, gauge);
    hold_at(damage, gauge);
    brake_wear_step(car, damage);
    hold_at(damage, &car.brake_wear_a);
    raise(damage, &car.brake_wear_b, |g| {
        add_scaled_triple(
            damage,
            &damage.gauge_rates[9],
            &damage.brake_balance_complement,
            &damage.brake_share,
            g,
            g,
        );
    });
    raise(damage, &car.electrics_damage, |g| {
        add_triple_product(
            &damage.gauge_rates[10],
            &damage.on_surface_5,
            &damage.gauge_step,
            g,
            g,
        );
    });
    for k in 0..6 {
        raise(damage, &car.light_damage[k], |g| {
            add_triple_product(
                &damage.gauge_rates[11],
                &damage.on_surface_5,
                &damage.gauge_step,
                g,
                g,
            );
        });
    }
    part_hit_figures(damage);
    let part = damage.drive_kind.bits();
    fn kind(table: &[Float; 2], part: u32) -> &Float {
        table.get(part as usize).unwrap_or_else(|| {
            crate::fatal::raise::end("a kind table", BadInput("the kind is not 0 or 1"))
        })
    }
    scale_part_hit_figures(
        damage,
        kind(&damage.kind_table_a, part),
        kind(&damage.kind_table_b, part),
    );
    for k in 0..2 {
        raise(damage, &car.tyre_wears[k], |g| {
            add_triple_product(
                &damage.front_slip_wear_factor,
                &damage.front_tyre_wear_rate,
                &damage.hundredth,
                g,
                g,
            );
        });
    }
    for k in 2..4 {
        raise(damage, &car.tyre_wears[k], |g| {
            add_triple_product(
                &damage.rear_slip_wear_factor,
                &damage.rear_tyre_wear_rate,
                &damage.hundredth,
                g,
                g,
            );
        });
    }
    raise(damage, &car.engine_damage, |g| {
        add_triple_product(
            &damage.engine_speed_wear_factor,
            &damage.gauge_rates[13],
            &damage.hundredth,
            g,
            g,
        );
    });
    raise(damage, &car.cooling_damage, |g| {
        add_triple_product(
            &damage.engine_speed_wear_factor,
            &damage.gauge_rates[12],
            &damage.hundredth,
            g,
            g,
        );
    });
    if !below_or_unordered(damage.overheat_excess.bits(), damage.gauge_low.bits()) {
        engine_damage_carry(car, damage);
        hold_at(damage, &car.engine_damage);
        raise(damage, &car.cooling_damage, |g| {
            add_triple_product(
                &damage.overheat_excess,
                &damage.gauge_rates[15],
                &damage.hundredth,
                g,
                g,
            );
        });
    }
    if damage.gear_changed.bits() == 0x1 {
        add_rate_to_clutch(car, &damage.gauge_rates[16]);
    }
    if damage.gear_changed.bits() == 0xFFFFFFFF {
        add_rate_to_clutch(car, &damage.gauge_rates[17]);
    }
    let gear = damage.gear.bits();
    if damage.gear_changed.bits() == 0x1 {
        let gauge = gear_gauge(car, gear);
        add_rate_to_gauge(gauge, &damage.gauge_rates[18]);
    }
    let gear = damage.gear.bits();
    if damage.gear_changed.bits() == 0xFFFFFFFF {
        let gauge = gear_gauge(car, gear);
        add_rate_to_gauge(gauge, &damage.gauge_rates[19]);
    }
    for k in 0..4 {
        raise(damage, &car.wheel_hit_damage[k], |g| {
            add_triple_product(
                &damage.roughness_wear_base,
                &damage.wheel_roughness_rate,
                &damage.hundredth,
                g,
                g,
            );
        });
    }
    for k in 0..4 {
        raise(damage, &car.suspension_damage[k], |g| {
            add_triple_product(
                &damage.suspension_roughness_wear,
                &damage.figures[0],
                &damage.hundredth,
                g,
                g,
            );
        });
    }
    raise(damage, &car.body_hit_damage, |g| {
        add_triple_product(
            &damage.body_roughness_wear,
            &damage.figures[1],
            &damage.hundredth,
            g,
            g,
        );
    });
    raise(damage, &car.steering_damage, |g| {
        add_triple_product(
            &damage.body_roughness_wear,
            &damage.figures[2],
            &damage.hundredth,
            g,
            g,
        );
    });
    raise(damage, &car.engine_damage, |g| {
        add_triple_product(
            &damage.body_roughness_wear,
            &damage.figures[3],
            &damage.hundredth,
            g,
            g,
        );
    });
    raise(damage, &car.exhaust_damage, |g| {
        add_triple_product(
            &damage.body_roughness_wear,
            &damage.figures[4],
            &damage.hundredth,
            g,
            g,
        );
    });
    raise(damage, &car.gear_wear[0], |g| {
        add_triple_product(
            &damage.body_roughness_wear,
            &damage.figures[5],
            &damage.hundredth,
            g,
            g,
        );
    });
    for k in 2..9 {
        raise(damage, &car.gear_wear[k], |g| {
            add_triple_product(
                &damage.body_roughness_wear,
                &damage.figures[5],
                &damage.hundredth,
                g,
                g,
            );
        });
    }
    raise(damage, &car.differential_damage, |g| {
        add_triple_product(
            &damage.body_roughness_wear,
            &damage.figures[6],
            &damage.hundredth,
            g,
            g,
        );
    });
    if !below_or_unordered(damage.hit_size.bits(), damage.gauge_rates[23].bits()) {
        raise(damage, &car.body_hit_damage, |g| {
            add_product(&damage.hit_size, &damage.figures[13], g, g);
        });
    }
    if !below_or_unordered(damage.hit_size.bits(), damage.gauge_rates[22].bits()) {
        side_hit_wear(car, damage, work);
    }
    for k in 0..4 {
        raise(damage, &car.suspension_damage[k], |g| {
            add_triple_product(
                &damage.wheel_loads[k],
                &damage.figures[14],
                &damage.hundredth,
                g,
                g,
            );
        });
    }
    raise(damage, &car.exhaust_damage, |g| {
        add_triple_product(
            &damage.wheel_load_mean,
            &damage.figures[15],
            &damage.hundredth,
            g,
            g,
        );
    });
    raise(damage, &car.steering_damage, |g| {
        add_triple_product(
            &damage.wheel_load_mean,
            &damage.figures[16],
            &damage.hundredth,
            g,
            g,
        );
    });
    gauge_totals(car, damage);
    if damage.drive_kind.bits() == 0 {
        car.turbo_damage.set_bits(0x0);
    }
}

/// A hit big enough to count (its size at least `gauge_rates[22]`) wears the gauges of the
/// side it landed on: `side` is 0 .. 3 for the hit figure 1 .. 4, and none for any other.
fn side_hit_wear(car: &Car, damage: &DamageModel, work: &RaceWork) {
    scale_hit_size(work, damage);
    let hit = damage.hit_side.bits();
    let side = (1..=4).contains(&hit).then(|| hit as usize - 1);
    let wear = |gauge: &Float, figure: usize| {
        raise(damage, gauge, |g| {
            add_product(&damage.hit_size, &damage.figures[figure], g, g);
        });
    };
    if let Some(side) = side {
        wear(&car.light_damage[side], 24);
        wear(&car.light_damage[4 + side / 2], 25);
    }
    wear(&car.steering_damage, 23);
    wear(&car.electrics_damage, 22);
    wear(&car.turbo_damage, 17);
    wear(&car.clutch_damage, 18);
    wear(&car.differential_damage, 19);
    for gauge in &car.gear_wear {
        wear(gauge, 20);
    }
    let gauge = gear_gauge(car, damage.gear.bits());
    add_hit_to_gear_gauge(damage, gauge);
    hold_at(damage, gauge);
    if let Some(side) = side {
        wear(&car.wheel_hit_damage[side], 28);
        wear(&car.suspension_damage[side], 30);
    }
    // The front hits (1 and 2) reach the cooling, the engine and the first brake wear,
    // the rear hits (3 and 4) the second brake wear.
    if (1..=2).contains(&hit) {
        wear(&car.cooling_damage, 27);
        wear(&car.engine_damage, 26);
        wear(&car.brake_wear_a, 29);
    }
    if (3..=4).contains(&hit) {
        wear(&car.brake_wear_b, 29);
    }
}

/// Applies `step` to the gauge, then holds it 0 ..= `gauge_top`.
pub(super) fn raise(damage: &DamageModel, gauge: &Float, step: impl FnOnce(&Float)) {
    step(gauge);
    hold_at(damage, gauge);
}

// ---- the arithmetic ----

/// `to = rate * step + from`.
fn add_product(rate: &impl Cell32, step: &impl Cell32, from: &impl Cell32, to: &impl Cell32) {
    store(to, value(rate) * value(step) + value(from));
}

/// `to = rate * a * b * gauge_step + from`.
fn add_scaled_triple(
    m: &DamageModel,
    rate: &impl Cell32,
    a: &impl Cell32,
    b: &impl Cell32,
    from: &impl Cell32,
    to: &impl Cell32,
) {
    store(
        to,
        value(rate) * value(a) * value(b) * value(&m.gauge_step) + value(from),
    );
}

/// The gear's gauge raised by the engine's speed share times the wear rate, a hundredth
/// and the gear's factor.
fn gear_wear_from_speed(m: &DamageModel, factor: &impl Cell32, gauge: &impl Cell32) {
    store(
        gauge,
        value(&m.engine_speed_share)
            * value(&m.wear_rates[12])
            * value(&m.hundredth)
            * value(factor)
            + value(gauge),
    );
}

/// The brake balance's complement, and the first brake wear raised by the rate times the
/// balance, the brake share and the gauge step.
fn brake_wear_step(car: &Car, m: &DamageModel) {
    store(
        &m.brake_balance_complement,
        1.0 - value(&m.brake_balance_share),
    );
    store(
        &car.brake_wear_a,
        value(&m.gauge_rates[9])
            * value(&m.brake_balance_share)
            * value(&m.brake_share)
            * value(&m.gauge_step)
            + value(&car.brake_wear_a),
    );
}

/// `to = rate * a * b + from`.
fn add_triple_product(
    rate: &impl Cell32,
    a: &impl Cell32,
    b: &impl Cell32,
    from: &impl Cell32,
    to: &impl Cell32,
) {
    store(to, value(rate) * value(a) * value(b) + value(from));
}

/// Both tyre wear rates: the speed share over the wear figure times the length of a mile
/// (in metres), as a share of the gauge, times the wheels-down switch.
fn part_hit_figures(m: &DamageModel) {
    let rate = value(&m.speed_share) / (value(&m.wear_figure) * value(&m.metres_per_mile))
        * value(&m.gauge_top)
        * value(&m.wheels_down);
    store(&m.front_tyre_wear_rate, rate);
    store(&m.rear_tyre_wear_rate, rate);
}

/// The two tyre wear rates times the given factors.
fn scale_part_hit_figures(m: &DamageModel, front: &impl Cell32, rear: &impl Cell32) {
    store(
        &m.front_tyre_wear_rate,
        value(&m.front_tyre_wear_rate) * value(front),
    );
    store(
        &m.rear_tyre_wear_rate,
        value(&m.rear_tyre_wear_rate) * value(rear),
    );
}

/// The engine's damage raised by its heat times the rate and a hundredth.
fn engine_damage_carry(car: &Car, m: &DamageModel) {
    store(
        &car.engine_damage,
        value(&car.engine_heat) * value(&m.gauge_rates[14]) * value(&m.hundredth)
            + value(&car.engine_damage),
    );
}

/// The clutch's damage plus a rate.
fn add_rate_to_clutch(car: &Car, rate: &impl Cell32) {
    store(&car.clutch_damage, value(&car.clutch_damage) + value(rate));
}

/// A gauge plus a rate.
fn add_rate_to_gauge(gauge: &impl Cell32, rate: &impl Cell32) {
    store(gauge, value(gauge) + value(rate));
}

/// The size of the hit times the damage setting.
fn scale_hit_size(work: &RaceWork, m: &DamageModel) {
    store(
        &m.hit_size,
        value(&m.hit_size) * value(&work.patch_factors[1]),
    );
}

/// The gear's gauge plus the size of the hit times the gear's hit figure.
fn add_hit_to_gear_gauge(m: &DamageModel, gauge: &impl Cell32) {
    store(
        gauge,
        value(&m.hit_size) * value(&m.figures[21]) + value(gauge),
    );
}

/// The sums of the gauges: the lights' mean, the brakes', the tyres', the gearbox's, the
/// body's, the suspension's and the stiffness mean.
fn gauge_totals(car: &Car, m: &DamageModel) {
    let [l0, l1, l2, l3, l4, l5] = &car.light_damage;
    store(
        &car.lights_damage,
        (value(l0) + value(l1) + value(l2) + value(l3) + value(l5) + value(l4))
            / value(&m.light_count_divisor),
    );
    let half = value(&m.half);
    store(
        &car.brake_damage,
        (value(&car.brake_wear_a) + value(&car.brake_wear_b)) * half,
    );
    let [t0, t1, t2, t3] = &car.tyre_wears;
    store(&m.front_tyre_wear, (value(t0) + value(t1)) * half);
    store(&m.rear_tyre_wear, (value(t2) + value(t3)) * half);
    store(
        &car.tyre_wear,
        (value(&m.front_tyre_wear) + value(&m.rear_tyre_wear)) * half,
    );
    // Every gear's wear but the neutral's (at 1).
    let [g0, _, g2, g3, g4, g5, g6, g7, g8] = &car.gear_wear;
    store(
        &car.gearbox_damage,
        (value(g0)
            + value(g2)
            + value(g3)
            + value(g4)
            + value(g5)
            + value(g6)
            + value(g7)
            + value(g8))
            * half
            * half
            * half,
    );
    let [w0, w1, w2, w3] = &car.wheel_hit_damage;
    let body = value(&car.body_hit_damage);
    store(
        &car.body_damage,
        (value(w0) + value(w1) + value(w2) + value(w3) + body + body + body + body + body + body)
            * value(&m.tenth),
    );
    let [s0, s1, s2, s3] = &car.suspension_damage;
    store(
        &car.overall_suspension_damage,
        (value(s0) + value(s1) + value(s2) + value(s3)) * value(&m.quarter),
    );
    let [k0, k1, k2, k3] = &car.wheel_stiffness;
    store(
        &car.stiffness_mean,
        (value(k0) + value(k1) + value(k2) + value(k3)) * value(&m.quarter),
    );
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
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse_at};

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
                plain: |a| brake_wear_step(&race().shared.car, dm(a[0])),
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
                        &dm(a[0]).brake_balance_complement,
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
                        &dm(a[0]).front_slip_wear_factor,
                        &dm(a[0]).front_tyre_wear_rate,
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.tyre_wears[0],
                        &crate::race::owner::race().shared.car.tyre_wears[0],
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
                        &dm(a[0]).front_slip_wear_factor,
                        &dm(a[0]).front_tyre_wear_rate,
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.tyre_wears[1],
                        &crate::race::owner::race().shared.car.tyre_wears[1],
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
                        &dm(a[0]).rear_slip_wear_factor,
                        &dm(a[0]).rear_tyre_wear_rate,
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.tyre_wears[2],
                        &crate::race::owner::race().shared.car.tyre_wears[2],
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
                        &dm(a[0]).rear_slip_wear_factor,
                        &dm(a[0]).rear_tyre_wear_rate,
                        &dm(a[0]).hundredth,
                        &crate::race::owner::race().shared.car.tyre_wears[3],
                        &crate::race::owner::race().shared.car.tyre_wears[3],
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
                        &dm(a[0]).engine_speed_wear_factor,
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
                        &dm(a[0]).engine_speed_wear_factor,
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
                plain: |a| engine_damage_carry(&race().shared.car, dm(a[1])),
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
                        &dm(a[0]).overheat_excess,
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
                plain: |a| add_rate_to_clutch(&race().shared.car, &dm(a[1]).gauge_rates[16]),
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
                plain: |a| add_rate_to_clutch(&race().shared.car, &dm(a[1]).gauge_rates[17]),
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
                        &dm(a[0]).roughness_wear_base,
                        &dm(a[0]).wheel_roughness_rate,
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
                        &dm(a[0]).roughness_wear_base,
                        &dm(a[0]).wheel_roughness_rate,
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
                        &dm(a[0]).roughness_wear_base,
                        &dm(a[0]).wheel_roughness_rate,
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
                        &dm(a[0]).roughness_wear_base,
                        &dm(a[0]).wheel_roughness_rate,
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
                        &dm(a[0]).suspension_roughness_wear,
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
                        &dm(a[0]).suspension_roughness_wear,
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
                        &dm(a[0]).suspension_roughness_wear,
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
                        &dm(a[0]).suspension_roughness_wear,
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
                        &dm(a[0]).body_roughness_wear,
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
                        &dm(a[0]).body_roughness_wear,
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
                        &dm(a[0]).body_roughness_wear,
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
                        &dm(a[0]).body_roughness_wear,
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
                        &dm(a[0]).body_roughness_wear,
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
                        &dm(a[0]).body_roughness_wear,
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
                        &dm(a[0]).body_roughness_wear,
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
                        &dm(a[0]).body_roughness_wear,
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
                        &dm(a[0]).body_roughness_wear,
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
                        &dm(a[0]).body_roughness_wear,
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
                        &dm(a[0]).body_roughness_wear,
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
                        &dm(a[0]).body_roughness_wear,
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
                        &dm(a[0]).body_roughness_wear,
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
                plain: |a| scale_hit_size(&race().shared.work, dm(a[0])),
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
                plain: |a| gauge_totals(&race().shared.car, dm(a[1])),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        // The plain code rounds to nearest, as the game always runs the damage model.
        sparse_at(&blocks(), 1200, 100, &[0x027F]);
    }
}

#[cfg(test)]
mod plain_matches_x87 {
    use super::*;
    use crate::physics::damage_model::proof::{gauges, prove};

    #[test]
    fn products_and_triples() {
        prove(
            "add_product",
            |s| {
                let (d, g) = (&s.damage, &s.car.engine_damage);
                gauges::add_product(&d.gauge_rates[0], &d.gauge_step, g, g);
            },
            |s| {
                let (d, g) = (&s.damage, &s.car.engine_damage);
                add_product(&d.gauge_rates[0], &d.gauge_step, g, g);
            },
        );
        prove(
            "add_scaled_triple",
            |s| {
                let (d, g) = (&s.damage, &s.car.turbo_damage);
                gauges::add_scaled_triple(
                    d,
                    &d.wear_rates[11],
                    &d.engine_over_limit,
                    &d.engine_speed_share,
                    g,
                    g,
                );
            },
            |s| {
                let (d, g) = (&s.damage, &s.car.turbo_damage);
                add_scaled_triple(
                    d,
                    &d.wear_rates[11],
                    &d.engine_over_limit,
                    &d.engine_speed_share,
                    g,
                    g,
                );
            },
        );
        prove(
            "add_triple_product",
            |s| {
                let (d, g) = (&s.damage, &s.car.electrics_damage);
                gauges::add_triple_product(
                    &d.gauge_rates[10],
                    &d.on_surface_5,
                    &d.gauge_step,
                    g,
                    g,
                );
            },
            |s| {
                let (d, g) = (&s.damage, &s.car.electrics_damage);
                add_triple_product(&d.gauge_rates[10], &d.on_surface_5, &d.gauge_step, g, g);
            },
        );
        prove(
            "gear_wear_from_speed",
            |s| {
                gauges::gear_wear_from_speed(
                    &s.damage,
                    &s.damage.gear_factors[2],
                    &s.car.gear_wear[2],
                );
            },
            |s| gear_wear_from_speed(&s.damage, &s.damage.gear_factors[2], &s.car.gear_wear[2]),
        );
        prove(
            "brake_wear_step",
            |s| gauges::brake_wear_step(&s.car, &s.damage),
            |s| brake_wear_step(&s.car, &s.damage),
        );
        prove(
            "engine_damage_carry",
            |s| gauges::engine_damage_carry(&s.car, &s.damage),
            |s| engine_damage_carry(&s.car, &s.damage),
        );
    }

    #[test]
    fn tyre_and_hit_figures() {
        prove(
            "part_hit_figures",
            |s| gauges::part_hit_figures(&s.damage),
            |s| part_hit_figures(&s.damage),
        );
        prove(
            "scale_part_hit_figures",
            |s| {
                gauges::scale_part_hit_figures(
                    &s.damage,
                    &s.damage.figures[3],
                    &s.damage.figures[4],
                );
            },
            |s| scale_part_hit_figures(&s.damage, &s.damage.figures[3], &s.damage.figures[4]),
        );
        prove(
            "add_rate_to_clutch",
            |s| gauges::add_rate_to_clutch(&s.car, &s.damage.figures[5]),
            |s| add_rate_to_clutch(&s.car, &s.damage.figures[5]),
        );
        prove(
            "add_rate_to_gauge",
            |s| gauges::add_rate_to_gauge(&s.car.tyre_wears[1], &s.damage.figures[6]),
            |s| add_rate_to_gauge(&s.car.tyre_wears[1], &s.damage.figures[6]),
        );
        prove(
            "scale_hit_size",
            |s| gauges::scale_hit_size(&s.work, &s.damage),
            |s| scale_hit_size(&s.work, &s.damage),
        );
        prove(
            "add_hit_to_gear_gauge",
            |s| gauges::add_hit_to_gear_gauge(&s.damage, &s.car.gear_wear[4]),
            |s| add_hit_to_gear_gauge(&s.damage, &s.car.gear_wear[4]),
        );
    }

    #[test]
    fn totals() {
        prove(
            "gauge_totals",
            |s| gauges::gauge_totals(&s.car, &s.damage),
            |s| gauge_totals(&s.car, &s.damage),
        );
    }
}
