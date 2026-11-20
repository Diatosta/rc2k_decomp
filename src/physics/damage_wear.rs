//! The wear of this step (`physics/damage_model.rs`).
//!
//! The clock goes into `DamageModel.clock`; table lerps of the speed shares give
//! `DamageModel.engine_speed_wear_factor` ..; the heat, damper, brake-fade and engine-heat
//! gauges are raised and lowered by their rates times the shares and `wear_step`, each held 0
//! ..= `gauge_top`; then the hit figures `DamageModel.roughness_wear_base` ..
//! `DamageModel.suspension_roughness_wear`; and `DamageModel.wear_figure` from the table
//! `DamageModel.wear_table[0]` (row `DamageModel.tyre_kind`, two columns through
//! `DamageModel.wear_columns`) blended by `DamageModel.surface_blend`, times
//! `DamageModel.wear_figure_scale`.

use crate::errors::BadInput;
use crate::memory::session::PlainFloat;
use crate::physics::car_store::Car;
use crate::physics::damage_gauges::raise;
use crate::physics::damage_model::{store, value};
use crate::physics::flow::{hold_low, table_lerp};
use crate::physics::store::DamageModel;

/// The wear of this step: the gauges raised or lowered by their rates times the shares,
/// the lerps of the speed shares, and the hit figures.
pub fn steps(car: &Car, damage: &DamageModel) -> Result<(), BadInput> {
    damage.clock.set(car.noise_word.get());
    table_lerp(
        &damage.engine_speed_share,
        &damage.lerp_tables[0],
        &damage.gauge_low,
        &damage.engine_speed_wear_factor,
    );
    for item in car.brake_heat.iter().take(2) {
        raise(damage, item, |g| {
            add_wear(damage, &damage.wear_rates[3], &damage.brake_share, g, g);
        });
    }
    for item in car.brake_heat.iter().take(2) {
        raise(damage, item, |g| {
            subtract_rate(damage, &damage.wear_rates[4], g, g);
        });
    }
    let extrapolated = [
        &damage.drive_a_extrapolated_share,
        &damage.drive_b_extrapolated_share,
    ];
    for (damper, share) in car.damper_wear.iter().zip(extrapolated) {
        raise(damage, damper, |g| {
            add_wear(damage, &damage.wear_rates[5], share, g, g);
        });
    }
    for item in car.damper_wear.iter().take(2) {
        raise(damage, item, |g| {
            subtract_rate(damage, &damage.wear_rates[6], g, g);
        });
    }
    raise(damage, &car.brake_fade, |g| {
        add_wear(damage, &damage.wear_rates[7], &damage.on_surface_5, g, g);
    });
    raise(damage, &car.brake_fade, |g| {
        subtract_rate(damage, &damage.wear_rates[8], g, g);
    });
    raise(damage, &car.brake_fade, |g| {
        subtract_wear(damage, &damage.wear_rates[9], &damage.brake_share, g, g);
    });
    table_lerp(
        &car.cooling_damage,
        &damage.lerp_tables[1],
        &damage.gauge_low,
        &damage.cooling_capacity,
    );
    lerp_gap(damage);
    hold_low(&damage.overheat_excess, &damage.gauge_low);
    raise(damage, &car.engine_heat, |g| {
        add_wear(damage, &damage.wear_rates[0], &damage.overheat_excess, g, g);
    });
    raise(damage, &car.engine_heat, |g| {
        subtract_rate(damage, &damage.wear_rates[1], g, g);
    });
    raise(damage, &car.engine_heat, |g| {
        subtract_wear(damage, &damage.wear_rates[2], &damage.speed_share, g, g);
    });
    table_lerp(
        &car.stiffness_mean,
        &damage.lerp_tables_b[2],
        &damage.gauge_low,
        &damage.stiffness_body_factor,
    );
    square_into(&damage.stiffness_body_factor, &damage.stiffness_body_factor);
    table_lerp(
        &car.stiffness_mean,
        &damage.lerp_tables_b[0],
        &damage.gauge_low,
        &damage.stiffness_suspension_factor,
    );
    square_into(
        &damage.stiffness_suspension_factor,
        &damage.stiffness_suspension_factor,
    );
    table_lerp(
        &car.ride_height_mean,
        &damage.lerp_tables_b[1],
        &damage.gauge_low,
        &damage.ride_height_body_factor,
    );
    square_into(
        &damage.ride_height_body_factor,
        &damage.ride_height_body_factor,
    );
    table_lerp(
        &damage.drive_a_slip_share,
        &damage.lerp_tables[3],
        &damage.gauge_low,
        &damage.front_slip_wear_factor,
    );
    table_lerp(
        &damage.drive_b_slip_share,
        &damage.lerp_tables[3],
        &damage.gauge_low,
        &damage.rear_slip_wear_factor,
    );
    hit_figures(damage);
    // Two figures of the table `DamageModel.wear_table` (row `DamageModel.tyre_kind`), blended by
    // `DamageModel.surface_blend`, x `DamageModel.wear_figure_scale`. Their columns are the
    // surface's pair of `DamageModel.wear_columns`: the first column, then the second. The 1999
    // code read the second at the first column's pair instead of the surface's (fixed: "Tyre
    // wear on snow, ice, water, dust and grass taken from the wrong surface" in
    // wiki/Fixed-bugs.md); its weight, `surface_blend`, is 0 on every shipped stage.
    let tyre_kind = damage.tyre_kind.get();
    let pair = &damage.wear_columns[(damage.surface.get() & 0xF) as usize];
    let first = damage.wear_figure_entry(tyre_kind, pair[0].get())?;
    let second = damage.wear_figure_entry(tyre_kind, pair[1].get())?;
    blend_wear_figures(first as i32, damage, second as i32);
    Ok(())
}

// ---- the arithmetic ----

/// `to = rate * share * wear_step + from`.
fn add_wear(
    m: &DamageModel,
    rate: &PlainFloat,
    share: &PlainFloat,
    from: &PlainFloat,
    to: &PlainFloat,
) {
    store(
        to,
        value(rate) * value(share) * value(&m.wear_step) + value(from),
    );
}

/// `to = from - rate * wear_step`.
fn subtract_rate(m: &DamageModel, rate: &PlainFloat, from: &PlainFloat, to: &PlainFloat) {
    store(to, -(value(rate) * value(&m.wear_step)) + value(from));
}

/// `to = from - rate * share * wear_step`.
fn subtract_wear(
    m: &DamageModel,
    rate: &PlainFloat,
    share: &PlainFloat,
    from: &PlainFloat,
    to: &PlainFloat,
) {
    store(
        to,
        -(value(rate) * value(share) * value(&m.wear_step)) + value(from),
    );
}

/// How far the engine's speed share is past the cooling capacity.
fn lerp_gap(m: &DamageModel) {
    store(
        &m.overheat_excess,
        value(&m.engine_speed_share) - value(&m.cooling_capacity),
    );
}

/// `to = from * from`.
fn square_into(from: &PlainFloat, to: &PlainFloat) {
    let x = value(from);
    store(to, x * x);
}

/// The three hit figures, each the product of the surface wear, the speed share and the
/// wheels-down switch with further factors; the third is the second times the suspension's.
fn hit_figures(m: &DamageModel) {
    let base = value(&m.point_surface_copy) * value(&m.speed_share) * value(&m.figures[32]);
    store(
        &m.body_roughness_wear,
        base * value(&m.stiffness_body_factor)
            * value(&m.ride_height_body_factor)
            * value(&m.wheels_down),
    );
    let roughness = base * value(&m.wheels_down);
    store(&m.roughness_wear_base, roughness);
    store(
        &m.suspension_roughness_wear,
        roughness * value(&m.stiffness_suspension_factor),
    );
}

/// The wear figure: the two table figures blended by `surface_blend`, times the scale.
fn blend_wear_figures(first: i32, m: &DamageModel, second: i32) {
    let blend = value(&m.surface_blend);
    store(
        &m.wear_figure,
        (f64::from(first) * (1.0 - blend) + f64::from(second) * blend)
            * value(&m.wear_figure_scale),
    );
}

#[cfg(test)]
mod reference_vectors_tests;

#[cfg(test)]
mod plain_matches_x87_tests;
