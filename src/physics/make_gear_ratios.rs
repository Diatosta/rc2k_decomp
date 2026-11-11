//! `physics::make_gear_ratios` (0x4219E6): the car's gear ratios (`g_gearRatios`, by gear,
//! `physics/gearing.rs`) from its setup: the top gear's (`g_gearRatios[g_gearCount]`) and
//! the first's (`flt_6140C4`) each lo + (hi - lo) `quarter` `gear_ratio_setting` (the
//! setup's gearing, a whole number) over `two_and_a_quarter` and `engine_speed_limit` (hi / lo
//! `flt_5CB978` / `flt_5CB970` and `flt_5CB974` / `figures`); the gears between in
//! even steps ((top - first) / (count - 1), kept at 80 bits on the stack and added on:
//! gears 2 .. 5 always, 6 and 7 one more step each only past five and six gears; the
//! step itself rounded into `scratch`); reverse (`gear_ratios`) the first's negative;
//! neutral (`g_gearRatios[0]`) `gear_ratio_step`.
//!
//! Returns: `g_gearCount`
//!
//! Callers: `physics::car_physics_step` (0x43B63F) and `physics::test_drive` (the car
//! specification's). Refuses (before anything) a gear count above 7 (past `g_gearRatios`).

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87::{ext_add, gear_step_80, lerp_int_over2};
use crate::physics::car_store::Car;
use crate::physics::gear_store::GEAR;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use crate::scene::matrix3::{extended, rounded};
use crate::session::Cell32;

const RATIO: usize = 8; // index of gear 0 in `game()`'s list

/// On the variables' cells `v`: the gear count.
pub fn ratios_at(numbers: &Numbers, v: &[&dyn Cell32]) -> Result<u32, Refused> {
    let get = |k: usize| v[k].bits();
    let put = |k: usize, x: u32| v[k].set_bits(x);
    let n = get(0);
    if n > 7 {
        return Err(BadInput("a gear count above 7 (past g_gearRatios)").into());
    }
    let (k, d) = (numbers.quarter.bits(), numbers.two_and_a_quarter.bits());
    let one: u32 = numbers.one.bits();
    let i = get(5) as i32;
    put(
        RATIO + n as usize,
        lerp_int_over2(get(1), get(2), k, i, d, get(6)).to_bits(),
    );
    put(
        RATIO + 1,
        lerp_int_over2(get(3), get(4), k, i, d, get(6)).to_bits(),
    );
    let step = gear_step_80(get(RATIO + n as usize), get(RATIO + 1), n as i32, one);
    let mut acc = extended(get(RATIO + 1));
    for gear in 2..=5 {
        acc = ext_add(acc, step);
        put(RATIO + gear, rounded(acc));
    }
    if n > 5 {
        acc = ext_add(acc, step);
    }
    put(RATIO + 6, rounded(acc));
    if n > 6 {
        acc = ext_add(acc, step);
    }
    put(RATIO + 7, rounded(acc));
    put(16, rounded(step));
    put(7, x87::negated(get(RATIO + 1)).to_bits());
    put(RATIO, x87::loaded(get(17)).to_bits());
    Ok(n)
}

fn game<'a>(
    car: &'a Car,
    tables: &'a PhysicsTables,
    scene: &'a crate::scene::owner::Scene,
) -> Vec<&'a dyn Cell32> {
    vec![
        &tables.gear_count as &dyn Cell32,
        &GEAR.figures[3],
        &GEAR.figures[1],
        &GEAR.figures[2],
        &GEAR.figures[0],
        &car.gear_ratio_setting,
        &car.engine_speed_limit,
        &tables.gear_ratios[0],
        &tables.gear_ratios[1],
        &tables.gear_ratios[2],
        &tables.gear_ratios[3],
        &tables.gear_ratios[4],
        &tables.gear_ratios[5],
        &tables.gear_ratios[6],
        &tables.gear_ratios[7],
        &tables.gear_ratios[8],
        &scene.drawing.scene.scratch,
        &tables.gear_ratio_step,
    ]
}

/// The exact version, for `physics::car_physics_step` and `physics::test_drive`: the gear count.
pub fn make_gear_ratios(
    car: &Car,
    tables: &PhysicsTables,
    numbers: &Numbers,
    scene: &crate::scene::owner::Scene,
) -> Result<u32, Refused> {
    ratios_at(numbers, &game(car, tables, scene))
}
