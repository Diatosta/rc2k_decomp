//! The gearbox by the keys (`input_gear_up`, `input_gear_down`). `gear_changed` = 0. Then, in this
//! order:
//! - `driver_kind` 2: the start-line rules (below);
//! - the gear refused by `physics::gear_bit`, or `forced_neutral` set: neutral;
//! - `gear_change_hold` not 0: it goes down, nothing else;
//! - `gear_change_blocked` set: nothing; `gearbox_kind` set: the start-line rules;
//! - up: from reverse neutral, from neutral `physics::pick_start_gear`, at the top nothing;
//!   otherwise `physics::wheels_from_revs`, `physics::higher_free_gear` (none: nothing), set,
//!   `physics::revs_from_wheels`, settled;
//! - down: from 1 neutral, from neutral into reverse (unless `gear_bit` refuses -1: the engine
//!   speed x 0.25, `wheels_from_revs`, settled), in reverse nothing; otherwise `wheels_from_revs`,
//!   `physics::lower_free_gear` (none: neutral), set, `revs_from_wheels`; an engine speed then not
//!   below `rev_limit` takes the gear back up (`revs_from_wheels`; `gear_change_hold` 20,
//!   `gear_changed` -1, `gear_change_pending` 1), else settled.
//!
//! The start-line rules: down asked: neutral into reverse, a forward gear to neutral; up asked:
//! neutral `pick_start_gear`, reverse to neutral; then, with `all_wheels_off` clear, the
//! automatic's tests (in gear 1 or more: the engine over its top by `physics::revs_from_speed`, the
//! engine speed put back, not below `shift_up_point`: up; in gear 2 or more: down when the next
//! free lower gear's share is below `shift_ratio_figure` and the engine over its top is below
//! `shift_down_clutch_point` / `shift_down_point`, by `brake`).
//!
//! Refuses what the calls refuse, and a gear outside -1 ..= 7 where a ratio is read; what was
//! written is put back.

use crate::errors::Refused;
use crate::math::x87;
use crate::math::x87::mul_div_status;
use crate::physics::car_store::Car;
use crate::physics::gear_store::GearFigures;
use crate::physics::gearbox::*;
use crate::physics::higher_free_gear::higher_free_gear;
use crate::physics::lower_free_gear::lower_free_gear;
use crate::physics::pick_start_gear::pick_start_gear;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use rc2k_core::physics::gear_shifts::{self as shifts, Shift};

/// Up.
fn up(car: &Car, tables: &PhysicsTables, numbers: &Numbers) -> Result<Shift, Refused> {
    let gear = car.gear.get();
    if let Some(shift) = shifts::manual_up(gear, || tables.gear_count.get()) {
        return Ok(shift);
    }
    wheels(car, tables)?;
    let Some(to) = { higher_free_gear(car, tables, gear) }? else {
        return Ok(Shift::Done);
    };
    car.gear.set(to);
    revs(car, tables, numbers)?;
    settle(car, tables);
    Ok(Shift::Done)
}

/// Down.
fn down(car: &Car, tables: &PhysicsTables, numbers: &Numbers) -> Result<Shift, Refused> {
    let gear = car.gear.get();
    if let Some(shift) = shifts::manual_down(gear) {
        return Ok(shift);
    }
    wheels(car, tables)?;
    let Some(to) = { lower_free_gear(car, gear) }? else {
        return Ok(Shift::Neutral);
    };
    car.gear.set(to);
    revs(car, tables, numbers)?;
    if !below(&car.engine_speed, &tables.rev_limit) {
        car.gear.set(car.gear.get().wrapping_add(1));
        revs(car, tables, numbers)?;
        car.gear_change_hold.set(shifts::HOLD);
        car.gear_changed.set(-1);
        car.gear_change_pending.set(1);
        return Ok(Shift::Done);
    }
    settle(car, tables);
    Ok(Shift::Done)
}

/// Into reverse (with `quarter` the engine speed x 0.25 first; the automatic's reverse has no such
/// step).
pub fn reverse(
    car: &Car,
    tables: &PhysicsTables,
    numbers: &Numbers,
    quarter: bool,
) -> Result<(), Refused> {
    if bit(car, -1)? {
        return Ok(());
    }
    car.gear.set(-1);
    if quarter {
        car.engine_speed
            .set(x87::mul(&car.engine_speed, &numbers.quarter));
    }
    wheels(car, tables)?;
    settle(car, tables);
    Ok(())
}

/// The start-line rules.
fn start_line(
    car: &Car,
    tables: &PhysicsTables,
    gear_figures: &GearFigures,
) -> Result<Shift, Refused> {
    let gear = car.gear.get();
    if let Some(shift) = shifts::start_line_keys(gear, tables.input_gear_down.get() != 0, || {
        tables.input_gear_up.get() != 0
    }) {
        return Ok(shift);
    }
    if car.all_wheels_off.get() != 0 {
        return Ok(Shift::Done);
    }
    if shifts::forward(gear) {
        let kept = car.engine_speed.get();
        revs_speed(car, tables)?;
        let over = over_top(car).to_bits();
        car.engine_speed.set(kept);
        if !below(over, gear_figures.shift_up_point.get()) {
            return Ok(Shift::Up);
        }
    }
    if !shifts::above_first(gear) {
        return Ok(Shift::Done);
    }
    let over = over_top(car).to_bits();
    let Some(lower) = { lower_free_gear(car, gear) }? else {
        return Ok(Shift::Done);
    };
    let (rb, rc) = (ratio(tables, gear)?, ratio(tables, lower)?);
    if mul_div_status(over, rb, rc, &gear_figures.shift_ratio_figure) & 0x100 == 0 {
        return Ok(Shift::Done);
    }
    let point = if shifts::down_point_clutch(car.brake.get()) {
        gear_figures.shift_down_clutch_point.get()
    } else {
        gear_figures.shift_down_point.get()
    };
    if !below(over, point) {
        return Ok(Shift::Done);
    }
    Ok(Shift::Down)
}

/// The steps on `r`.
// Core: `rc2k_core::physics::gear_shifts` (the engine's sums and compares stay x87).
pub(super) fn manual_gearbox(
    car: &Car,
    tables: &PhysicsTables,
    numbers: &Numbers,
    gear_figures: &GearFigures,
) -> Result<(), Refused> {
    car.gear_changed.set(0);
    let mut to = 'first: {
        if car.driver_kind.get() == 2 {
            break 'first start_line(car, tables, gear_figures)?;
        }
        let gear = car.gear.get();
        if bit(car, gear)? || car.forced_neutral.get() != 0 {
            break 'first Shift::Neutral;
        }
        if car.gear_change_hold.get() != 0 {
            car.gear_change_hold
                .set(car.gear_change_hold.get().wrapping_sub(1));
            return Ok(());
        }
        if car.gear_change_blocked.get() != 0 {
            return Ok(());
        }
        if car.gearbox_kind.get() != 0 {
            break 'first start_line(car, tables, gear_figures)?;
        }
        shifts::manual_keys(tables.input_gear_up.get() != 0, || {
            tables.input_gear_down.get() != 0
        })
    };
    loop {
        to = match to {
            Shift::Up => up(car, tables, numbers)?,
            Shift::Down => down(car, tables, numbers)?,
            Shift::Neutral => {
                neutral(car, tables);
                Shift::Done
            }
            Shift::Start => {
                pick_start_gear(car, tables, numbers, gear_figures)?;
                Shift::Done
            }
            Shift::Reverse => {
                reverse(car, tables, numbers, true)?;
                Shift::Done
            }
            Shift::Done => return Ok(()),
        };
    }
}
