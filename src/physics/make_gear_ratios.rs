//! The car's gear ratios (`gear_ratios`, by gear, `physics/gearing.rs`) from its setup: the top
//! gear's (`gear_ratios[gear_count]`) and the first's (`PhysicsTables.gear_ratios[2]`) each lo +
//! (hi - lo) `quarter` `gear_ratio_setting` (the setup's gearing, a whole number) over
//! `two_and_a_quarter` and `engine_speed_limit` (hi / lo are `LightsAndGear.gear.figures[3]` /
//! `figures[1]` and `figures[2]` / `figures`); the gears between in even steps ((top - first) /
//! (count - 1), kept at 80 bits and added on: gears 2 .. 5 always, 6 and 7 one more step each only
//! past five and six gears; the step itself rounded into `scratch`); reverse (`gear_ratios`) is the
//! first's negative; neutral (`gear_ratios[0]`) is `gear_ratio_step`.
//!
//! Refuses (before anything) a gear count above 7 (past `gear_ratios`).

use crate::errors::{BadInput, Refused};
use crate::math::matrix3::{extended, rounded_value};
use crate::math::x87;
use crate::math::x87::{ext_add, gear_step_80, lerp_int_over2};
use crate::physics::car_store::Car;
use crate::physics::figures::GEAR_RATIO_STEP;
use crate::physics::gear_store::GearFigures;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;

/// The exact version, for `physics::car_physics_step` and `physics::acceleration_figures`: the gear count.
pub(super) fn make_gear_ratios(
    car: &Car,
    tables: &mut PhysicsTables,
    numbers: &Numbers,
    gear_figures: &GearFigures,
) -> Result<u32, Refused> {
    let n = tables.gear_count;
    if n > 7 {
        return Err(BadInput("a gear count above 7 (past g_gearRatios)").into());
    }
    // Gear `g`'s ratio (reverse, -1, first).
    let at = |g: i32| (g + 1) as usize;
    let ratios = &mut tables.gear_ratios;
    let figures = &gear_figures.figures;
    let (k, d) = (&numbers.quarter, &numbers.two_and_a_quarter);
    let one = numbers.one.get();
    let i = car.gear_ratio_setting.get() as i32;
    let limit = car.engine_speed_limit.get();
    let top = n as i32;
    ratios[at(top)] = lerp_int_over2(figures[3].get(), figures[1].get(), k, i, d, limit);
    ratios[at(1)] = lerp_int_over2(figures[2].get(), figures[0].get(), k, i, d, limit);
    let step = gear_step_80(ratios[at(top)], ratios[at(1)], top, one);
    let mut acc = extended(ratios[at(1)]);
    for gear in 2..=5 {
        acc = ext_add(acc, step);
        ratios[at(gear)] = rounded_value(acc);
    }
    if n > 5 {
        acc = ext_add(acc, step);
    }
    ratios[at(6)] = rounded_value(acc);
    if n > 6 {
        acc = ext_add(acc, step);
    }
    ratios[at(7)] = rounded_value(acc);
    ratios[at(-1)] = x87::negated(ratios[at(1)]);
    ratios[at(0)] = x87::loaded(GEAR_RATIO_STEP);
    Ok(n)
}
