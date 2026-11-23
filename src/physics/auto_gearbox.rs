//! The automatic gearbox. `gear_changed` is cleared first. Then:
//! - a gear refused by `physics::gear_bit`, or any gear with the engine speed below
//!   `idle_engine_speed`: neutral;
//! - in neutral: nothing unless the engine's limit times `auto_gear_engine_share` is below the
//!   engine speed; then reverse when `input_gear_wanted` is -1, else `physics::pick_start_gear`;
//! - `forced_neutral` set: neutral; `gear_change_hold` not 0: it counts down;
//!   `gear_change_blocked` set: nothing;
//! - otherwise the engine's share of its limit decides: in gear 1 or more with the wheels on the
//!   ground and the share not below `shift_up_point`, up (nothing at the top; from reverse
//!   `pick_start_gear`); in reverse with `brake` set and the speed not below
//!   `auto_gear_speed_down`, `pick_start_gear`; in gear 2 or more with the share under
//!   `shift_ratio_figure` and the engine under the down point (`shift_down_clutch_point` or
//!   `shift_down_point`, by `brake`), down (an engine speed then not below `rev_limit` takes it
//!   back up and ends in neutral); in gear 1 with `brake` set and the speed below
//!   `auto_gear_speed_up`, reverse.
//!   Shifts go through `physics::revs_from_speed`, `wheels_from_revs` and `revs_from_wheels`.
//!
//! Refuses what the calls refuse, and a gear outside -1 ..= 7 where a ratio is read; what was
//! written is put back.

use crate::errors::Refused;
use crate::math::x87::{mul_div_status, mul_status};
use crate::memory::plain_floats::FloatRow;
use crate::physics::car_store::Car;
use crate::physics::figures::{AUTO_GEAR_ENGINE_SHARE, AUTO_GEAR_SPEED_DOWN, AUTO_GEAR_SPEED_UP};
use crate::physics::gear_store::GearFigures;
use crate::physics::gearbox::*;
use crate::physics::manual_gearbox::reverse;
use crate::physics::pick_start_gear::pick_start_gear;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use rc2k_core::physics::gear_shifts::{self as shifts, Shift};
use rc2k_core::physics::gearbox::GearInputs;

/// Up.
fn up(car: &mut Car, tables: &mut PhysicsTables, numbers: &Numbers) -> Result<Shift, Refused> {
    if let Some(shift) = shifts::auto_up(car.gear, tables.gear_count) {
        return Ok(shift);
    }
    wheels(car, tables)?;
    car.gear = car.gear.wrapping_add(1);
    revs(car, tables, numbers)?;
    settle(car);
    Ok(Shift::Done)
}

/// Down.
fn down(car: &mut Car, tables: &mut PhysicsTables, numbers: &Numbers) -> Result<Shift, Refused> {
    wheels(car, tables)?;
    car.gear = car.gear.wrapping_sub(1);
    revs(car, tables, numbers)?;
    if below(car.engine_speed, tables.rev_limit) {
        settle(car);
        return Ok(Shift::Done);
    }
    car.gear = car.gear.wrapping_add(1);
    revs(car, tables, numbers)?;
    Ok(Shift::Neutral)
}

/// The automatic's tests.
fn tests(
    car: &mut Car,
    tables: &mut PhysicsTables,
    gear_figures: &GearFigures,
) -> Result<Shift, Refused> {
    // `revs_from_speed` leaves the gear it reads in the same place.
    let gear = car.gear;
    let kept = car.engine_speed;
    revs_speed(car, tables)?;
    let over = over_top(car).to_bits();
    car.engine_speed = kept;
    shifts::auto_tests(
        gear,
        || car.all_wheels_off != 0,
        || car.brake != 0,
        || !below(over, gear_figures.shift_up_point.get()),
        || !below(car.local_velocity.at(2), AUTO_GEAR_SPEED_DOWN),
        || {
            let over = over_top(car).to_bits();
            if !shifts::above_first(gear) {
                return Ok(false);
            }
            let (rb, rl) = (ratio(tables, gear)?, ratio(tables, gear.wrapping_sub(1))?);
            if mul_div_status(over, rb, rl, &gear_figures.shift_ratio_figure) & 0x100 == 0 {
                return Ok(false);
            }
            let point = if shifts::down_point_clutch(car.brake) {
                gear_figures.shift_down_clutch_point.get()
            } else {
                gear_figures.shift_down_point.get()
            };
            Ok(below(over, point))
        },
        || below(car.local_velocity.at(2), AUTO_GEAR_SPEED_UP),
    )
}

/// The steps on `r`.
// Core: `rc2k_core::physics::gear_shifts` (the engine's sums and compares stay x87).
pub(super) fn auto_gearbox(
    car: &mut Car,
    tables: &mut PhysicsTables,
    inputs: &GearInputs,
    numbers: &Numbers,
    gear_figures: &GearFigures,
) -> Result<(), Refused> {
    car.gear_changed = 0;
    let mut to = 'first: {
        if bit(car, car.gear)? {
            break 'first Shift::Neutral;
        }
        if car.gear != 0 && below(car.engine_speed, tables.idle_engine_speed) {
            break 'first Shift::Neutral;
        }
        if car.gear == 0 {
            if mul_status(
                car.engine_speed_limit,
                AUTO_GEAR_ENGINE_SHARE,
                car.engine_speed,
            ) & 0x100
                == 0
            {
                return Ok(());
            }
            break 'first if inputs.joined == u32::MAX {
                Shift::Reverse
            } else {
                Shift::Start
            };
        }
        if car.forced_neutral != 0 {
            break 'first Shift::Neutral;
        }
        if car.gear_change_hold != 0 {
            car.gear_change_hold = car.gear_change_hold.wrapping_sub(1);
            return Ok(());
        }
        if car.gear_change_blocked != 0 {
            return Ok(());
        }
        tests(car, tables, gear_figures)?
    };
    loop {
        to = match to {
            Shift::Up => up(car, tables, numbers)?,
            Shift::Down => down(car, tables, numbers)?,
            Shift::Neutral => {
                neutral(car);
                Shift::Done
            }
            Shift::Start => {
                pick_start_gear(car, tables, numbers, gear_figures)?;
                Shift::Done
            }
            Shift::Reverse => {
                reverse(car, tables, numbers, false)?;
                Shift::Done
            }
            Shift::Done => return Ok(()),
        };
    }
}
