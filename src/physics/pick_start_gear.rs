//! From neutral, the gear to start in. the engine speed is kept; `physics::revs_from_speed` runs;
//! `start_gear_figures` get 0 and 100.0; then gears 1 .. 6 in turn: one `physics::gear_bit` does
//! not refuse is set, `revs_from_speed` runs, and the engine speed over its limit
//! (`engine_speed_limit`) is stored; the first below `shift_up_point` is taken: set,
//! `revs_from_speed`, the engine speed moved back toward the kept one by `start_gear_share`
//! ((kept - e) x k + e), `physics::wheels_from_revs`, and the change settled (`drive_cut_steps`,
//! the hold 20, `gear_changed` and `gear_change_pending` 1). None taken: the last tried stays.
//!
//! Refuses what the calls refuse; what was written is put back.

use crate::errors::Refused;
use crate::math::x87;
use crate::math::x87::sub_mul_add;
use crate::physics::car_store::Car;
use crate::physics::figures::START_GEAR_SHARE;
use crate::physics::gear_store::GearFigures;
use crate::physics::gearbox::*;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use rc2k_core::physics::gear_shifts as shifts;

/// The steps on `r`.
// Core: `rc2k_core::physics::gear_shifts::pick_start` (the engine's sums and compares stay x87).
pub(super) fn pick_start_gear(
    car: &Car,
    tables: &PhysicsTables,
    numbers: &Numbers,
    gear_figures: &GearFigures,
) -> Result<(), Refused> {
    tables.start_gear_figures[2].set(car.engine_speed.get());
    revs_speed(car, tables)?;
    tables.start_gear_figures[1].set(0.0);
    tables.start_gear_figures[0].set(x87::loaded(&numbers.hundred));
    let taken = shifts::pick_start(|gear| {
        if bit(car, gear)? {
            return Ok::<_, Refused>(false);
        }
        car.gear.set(gear);
        revs_speed(car, tables)?;
        car.engine_speed.set(over_top(car));
        Ok(below(
            car.engine_speed.get(),
            gear_figures.shift_up_point.get(),
        ))
    })?;
    let Some(gear) = taken else {
        return Ok(());
    };
    car.gear.set(gear);
    revs_speed(car, tables)?;
    car.engine_speed.set(sub_mul_add(
        tables.start_gear_figures[2].get(),
        &car.engine_speed,
        START_GEAR_SHARE,
        &car.engine_speed,
    ));
    wheels(car, tables)?;
    settle(car, tables);
    Ok(())
}
