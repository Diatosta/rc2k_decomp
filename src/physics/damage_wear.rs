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
use crate::physics::car_store::Car;
use crate::physics::damage_gauges::raise;
use crate::physics::damage_model::{figure, held_low, lerp_table};
use crate::physics::store::DamageModel;

/// The wear of this step: the gauges raised or lowered by their rates times the shares,
/// the lerps of the speed shares, and the hit figures.
pub fn steps(car: &mut Car, damage: &mut DamageModel) -> Result<(), BadInput> {
    damage.clock = car.noise_word;
    damage.engine_speed_wear_factor = lerp_table(
        damage.engine_speed_share,
        &damage.lerp_tables[0],
        damage.gauge_low,
    );
    for item in car.brake_heat.iter_mut().take(2) {
        raise(damage, item, |g| {
            add_wear(damage, damage.wear_rates[3], damage.brake_share, g)
        });
    }
    for item in car.brake_heat.iter_mut().take(2) {
        raise(damage, item, |g| {
            subtract_rate(damage, damage.wear_rates[4], g)
        });
    }
    let extrapolated = [
        damage.drive_a_extrapolated_share,
        damage.drive_b_extrapolated_share,
    ];
    for (damper, share) in car.damper_wear.iter_mut().zip(extrapolated) {
        raise(damage, damper, |g| {
            add_wear(damage, damage.wear_rates[5], share, g)
        });
    }
    for item in car.damper_wear.iter_mut().take(2) {
        raise(damage, item, |g| {
            subtract_rate(damage, damage.wear_rates[6], g)
        });
    }
    raise(damage, &mut car.brake_fade, |g| {
        add_wear(damage, damage.wear_rates[7], damage.on_surface_5, g)
    });
    raise(damage, &mut car.brake_fade, |g| {
        subtract_rate(damage, damage.wear_rates[8], g)
    });
    raise(damage, &mut car.brake_fade, |g| {
        subtract_wear(damage, damage.wear_rates[9], damage.brake_share, g)
    });
    damage.cooling_capacity =
        lerp_table(car.cooling_damage, &damage.lerp_tables[1], damage.gauge_low);
    lerp_gap(damage);
    held_low(&mut damage.overheat_excess, damage.gauge_low);
    raise(damage, &mut car.engine_heat, |g| {
        add_wear(damage, damage.wear_rates[0], damage.overheat_excess, g)
    });
    raise(damage, &mut car.engine_heat, |g| {
        subtract_rate(damage, damage.wear_rates[1], g)
    });
    raise(damage, &mut car.engine_heat, |g| {
        subtract_wear(damage, damage.wear_rates[2], damage.speed_share, g)
    });
    damage.stiffness_body_factor = lerp_table(
        car.stiffness_mean,
        &damage.lerp_tables_b[2],
        damage.gauge_low,
    );
    square_into(&mut damage.stiffness_body_factor);
    damage.stiffness_suspension_factor = lerp_table(
        car.stiffness_mean,
        &damage.lerp_tables_b[0],
        damage.gauge_low,
    );
    square_into(&mut damage.stiffness_suspension_factor);
    damage.ride_height_body_factor = lerp_table(
        car.ride_height_mean,
        &damage.lerp_tables_b[1],
        damage.gauge_low,
    );
    square_into(&mut damage.ride_height_body_factor);
    damage.front_slip_wear_factor = lerp_table(
        damage.drive_a_slip_share,
        &damage.lerp_tables[3],
        damage.gauge_low,
    );
    damage.rear_slip_wear_factor = lerp_table(
        damage.drive_b_slip_share,
        &damage.lerp_tables[3],
        damage.gauge_low,
    );
    hit_figures(damage);
    // Two figures of the table `DamageModel.wear_table` (row `DamageModel.tyre_kind`), blended by
    // `DamageModel.surface_blend`, x `DamageModel.wear_figure_scale`. Their columns are the
    // surface's pair of `DamageModel.wear_columns`: the first column, then the second. The 1999
    // code read the second at the first column's pair instead of the surface's (fixed: "Tyre
    // wear on snow, ice, water, dust and grass taken from the wrong surface" in
    // wiki/Fixed-bugs.md); its weight, `surface_blend`, is 0 on every shipped stage.
    let tyre_kind = damage.tyre_kind;
    let pair = damage.wear_columns[(damage.surface & 0xF) as usize];
    let first = damage.wear_figure_entry(tyre_kind, pair[0])?;
    let second = damage.wear_figure_entry(tyre_kind, pair[1])?;
    blend_wear_figures(first as i32, damage, second as i32);
    Ok(())
}

// ---- the arithmetic ----

/// `rate * share * wear_step + from`.
fn add_wear(m: &DamageModel, rate: f32, share: f32, from: f32) -> f32 {
    (figure(rate) * figure(share) * figure(m.wear_step) + figure(from)) as f32
}

/// `from - rate * wear_step`.
fn subtract_rate(m: &DamageModel, rate: f32, from: f32) -> f32 {
    (-(figure(rate) * figure(m.wear_step)) + figure(from)) as f32
}

/// `from - rate * share * wear_step`.
fn subtract_wear(m: &DamageModel, rate: f32, share: f32, from: f32) -> f32 {
    (-(figure(rate) * figure(share) * figure(m.wear_step)) + figure(from)) as f32
}

/// How far the engine's speed share is past the cooling capacity.
fn lerp_gap(m: &mut DamageModel) {
    m.overheat_excess = (figure(m.engine_speed_share) - figure(m.cooling_capacity)) as f32;
}

/// `x * x`.
fn square_into(x: &mut f32) {
    let value = figure(*x);
    *x = (value * value) as f32;
}

/// The three hit figures, each the product of the surface wear, the speed share and the
/// wheels-down switch with further factors; the third is the second times the suspension's.
fn hit_figures(m: &mut DamageModel) {
    let base = figure(m.point_surface_copy) * figure(m.speed_share) * figure(m.figures[32]);
    m.body_roughness_wear = (base
        * figure(m.stiffness_body_factor)
        * figure(m.ride_height_body_factor)
        * figure(m.wheels_down)) as f32;
    let roughness = base * figure(m.wheels_down);
    m.roughness_wear_base = roughness as f32;
    m.suspension_roughness_wear = (roughness * figure(m.stiffness_suspension_factor)) as f32;
}

/// The wear figure: the two table figures blended by `surface_blend`, times the scale.
fn blend_wear_figures(first: i32, m: &mut DamageModel, second: i32) {
    let blend = figure(m.surface_blend);
    m.wear_figure = ((f64::from(first) * (1.0 - blend) + f64::from(second) * blend)
        * figure(m.wear_figure_scale)) as f32;
}

#[cfg(test)]
mod reference_vectors_tests;

#[cfg(test)]
mod plain_matches_x87_tests;
