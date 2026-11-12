//! `physics::auto_gearbox` (0x43C5C2): the automatic gearbox. `gear_changed` is cleared first. Then:
//! - the gear refused by `physics::gear_bit`, or in gear with the engine speed below
//!   `idle_engine_speed`: neutral;
//! - in neutral: unless the engine's limit (`engine_speed_limit`) times `auto_gear_engine_share`
//!   is below the engine speed, nothing; else, with `input_gear_wanted` -1, into reverse,
//!   else `physics::pick_start_gear`;
//! - `forced_neutral` set: neutral; `gear_change_hold` not 0: it goes down;
//!   `gear_change_blocked` set: nothing;
//! - else the engine over its limit is found by `physics::revs_from_speed` (the engine speed put
//!   back): in gear 1 or more with `all_wheels_off` clear and the share not below
//!   `shift_up_point`: up (at the top nothing, from reverse `pick_start_gear`, else
//!   `physics::wheels_from_revs`, the gear up, `physics::revs_from_wheels`, settled);
//!   in reverse with `brake` set and the speed not below `auto_gear_speed_down`:
//!   `pick_start_gear`; in gear 2 or more, the share to the gear below under
//!   `shift_ratio_figure` and the engine over its limit under `shift_down_clutch_point` /
//!   `shift_down_point` (by `brake`): down (`wheels_from_revs`, the gear down,
//!   `revs_from_wheels`; an engine speed then not below `rev_limit` takes it back up and
//!   ends in neutral, else settled); in gear 1 with `brake` set and the speed below
//!   `auto_gear_speed_up`: into reverse.
//!
//! Caller: `physics::gearbox_step`. Refuses what the calls refuse, and a gear outside -1
//! ..= 7 where a ratio is read; what was written is put back.

use crate::errors::Refused;
use crate::math::x87::{mul_div_status, mul_status};
use crate::physics::car_store::Car;
use crate::physics::gear_store::GearFigures;
use crate::physics::gearbox::*;
use crate::physics::manual_gearbox::reverse;
use crate::physics::pick_start_gear::pick_start_gear;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use rc2k_core::physics::gear_shifts::{self as shifts, Shift};

/// Up (0x43C66D).
fn up(car: &Car, tables: &PhysicsTables, numbers: &Numbers) -> Result<Shift, Refused> {
    if let Some(shift) = shifts::auto_up(car.gear.get(), tables.gear_count.get()) {
        return Ok(shift);
    }
    wheels(car, tables)?;
    car.gear.set(car.gear.get().wrapping_add(1));
    revs(car, tables, numbers)?;
    settle(car, tables);
    Ok(Shift::Done)
}

/// Down (0x43C6B7).
fn down(car: &Car, tables: &PhysicsTables, numbers: &Numbers) -> Result<Shift, Refused> {
    wheels(car, tables)?;
    car.gear.set(car.gear.get().wrapping_sub(1));
    revs(car, tables, numbers)?;
    if below(car.engine_speed.get(), tables.rev_limit.get()) {
        settle(car, tables);
        return Ok(Shift::Done);
    }
    car.gear.set(car.gear.get().wrapping_add(1));
    revs(car, tables, numbers)?;
    Ok(Shift::Neutral)
}

/// The automatic's tests (0x43C798).
fn tests(
    car: &Car,
    tables: &PhysicsTables,
    gear_figures: &GearFigures,
    scene: &crate::scene::owner::Scene,
) -> Result<Shift, Refused> {
    let drawing = &scene.drawing;
    // `revs_from_speed` leaves the gear it reads in the same place.
    let gear = car.gear.get();
    let kept = car.engine_speed.get();
    revs_speed(car, tables)?;
    drawing.scene.scratch.set(over_top(car));
    car.engine_speed.set(kept);
    shifts::auto_tests(
        gear,
        || car.all_wheels_off.get() != 0,
        || car.brake.get() != 0,
        || {
            !below(
                drawing.scene.scratch.get(),
                gear_figures.shift_up_point.get(),
            )
        },
        || {
            !below(
                car.local_velocity[2].get(),
                tables.auto_gear_speed_down.get(),
            )
        },
        || {
            drawing.scene.scratch.set(over_top(car));
            if !shifts::above_first(gear) {
                return Ok(false);
            }
            let (rb, rl) = (ratio(tables, gear)?, ratio(tables, gear.wrapping_sub(1))?);
            if mul_div_status(
                drawing.scene.scratch.get(),
                rb,
                rl,
                &gear_figures.shift_ratio_figure,
            ) & 0x100
                == 0
            {
                return Ok(false);
            }
            let point = if shifts::down_point_clutch(car.brake.get()) {
                gear_figures.shift_down_clutch_point.get()
            } else {
                gear_figures.shift_down_point.get()
            };
            Ok(below(drawing.scene.scratch.get(), point))
        },
        || below(car.local_velocity[2].get(), tables.auto_gear_speed_up.get()),
    )
}

/// The steps on `r`.
// Converted: `rc2k_core::physics::gear_shifts` (the engine's sums and compares stay x87).
pub fn auto_gearbox(
    car: &Car,
    tables: &PhysicsTables,
    numbers: &Numbers,
    gear_figures: &GearFigures,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    car.gear_changed.set(0);
    let mut to = 'first: {
        if bit(car, car.gear.get())? {
            break 'first Shift::Neutral;
        }
        if car.gear.get() != 0 && below(car.engine_speed.get(), tables.idle_engine_speed.get()) {
            break 'first Shift::Neutral;
        }
        if car.gear.get() == 0 {
            if mul_status(
                &car.engine_speed_limit,
                &tables.auto_gear_engine_share,
                &car.engine_speed,
            ) & 0x100
                == 0
            {
                return Ok(());
            }
            break 'first if tables.input_gear_wanted.get() == u32::MAX {
                Shift::Reverse
            } else {
                Shift::Start
            };
        }
        if car.forced_neutral.get() != 0 {
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
        tests(car, tables, gear_figures, scene)?
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
                reverse(car, tables, numbers, false)?;
                Shift::Done
            }
            Shift::Done => return Ok(()),
        };
    }
}
