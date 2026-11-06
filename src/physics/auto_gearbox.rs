//! `physics::auto_gearbox` (0x43C5C2): the automatic gearbox. +0x238 0. Then:
//! - the gear refused by `physics::gear_bit`, or in gear with the engine speed below
//!   `flt_6140F8`: neutral;
//! - in neutral: unless the engine's top (+0x1DC) x `flt_613F18` is below the engine
//!   speed, nothing; else with `dword_6138A8` -1 into reverse, else
//!   `physics::pick_start_gear`;
//! - +0x430 set: neutral; the hold +0x1CC not 0: it goes down; +0x42C set: nothing;
//! - else (ebx the gear) the engine over its top by `physics::revs_from_speed` (the
//!   engine speed put back): in gear 1 or more with +0x254 clear and it not below
//!   `flt_5CB958`: up (at the top nothing, from reverse `pick_start_gear`, else
//!   `physics::wheels_from_revs`, the gear up, `physics::revs_from_wheels`, settled);
//!   in reverse with +0x63C set and the speed not below `flt_613E98`:
//!   `pick_start_gear`; in gear 2 or more, the share to the gear below under
//!   `flt_5CB964` and the engine over its top under `flt_5CB960` / `flt_5CB95C` (by
//!   +0x63C): down (`wheels_from_revs`, the gear down, `revs_from_wheels`; an engine
//!   speed then not below `flt_614104` takes it back up and ends in neutral, else
//!   settled); in gear 1 with +0x63C set and the speed below `flt_613E94`: into reverse.
//!
//! In:  nothing
//! Out: the registers as the steps leave them; no flags read (the caller jumps on)
//!
//! Caller: `physics::gearbox_step`. Refuses what the calls refuse, and a gear outside -1
//! ..= 7 where a ratio is read; what was written is put back.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::x87_run::{mul_div_status, mul_status};
use crate::physics::gear_store::GEAR;
use crate::physics::gearbox::*;
use crate::physics::manual_gearbox::reverse;
use crate::physics::pick_start_gear::pick_start_gear;
use crate::race::owner::Race;
use crate::scene::owner::scene;
use rc2k_core::physics::gear_shifts::{self as shifts, Shift};

/// Up (0x43C66D).
fn up(race: &Race) -> Result<Shift, Refused> {
    if let Some(shift) = shifts::auto_up(
        race.shared.car.gear.get(),
        race.shared.physics_tables.gear_count.get(),
    ) {
        return Ok(shift);
    }
    wheels(&race.shared)?;
    race.shared
        .car
        .gear
        .set(race.shared.car.gear.get().wrapping_add(1));
    revs(race)?;
    settle(race);
    Ok(Shift::Done)
}

/// Down (0x43C6B7).
fn down(race: &Race) -> Result<Shift, Refused> {
    wheels(&race.shared)?;
    race.shared
        .car
        .gear
        .set(race.shared.car.gear.get().wrapping_sub(1));
    revs(race)?;
    if below(
        race.shared.car.engine_speed.bits(),
        race.shared.physics_tables.rev_limit.bits(),
    ) {
        settle(race);
        return Ok(Shift::Done);
    }
    race.shared
        .car
        .gear
        .set(race.shared.car.gear.get().wrapping_add(1));
    revs(race)?;
    Ok(Shift::Neutral)
}

/// The automatic's tests (0x43C798).
fn tests(race: &Race) -> Result<Shift, Refused> {
    let drawing = &scene().drawing;
    // `revs_from_speed` leaves ebx the gear it reads: the same.
    let gear = race.shared.car.gear.get();
    let kept = race.shared.car.engine_speed.bits();
    revs_speed(&race.shared)?;
    drawing.scene.scratch.set(over_top(race));
    race.shared.car.engine_speed.set_bits(kept);
    shifts::auto_tests(
        gear,
        || race.shared.car.all_wheels_off.get() != 0,
        || race.shared.car.brake.get() != 0,
        || !below(drawing.scene.scratch.get(), GEAR.shift_up_point.bits()),
        || {
            !below(
                race.shared.car.local_velocity[2].bits(),
                race.shared.physics_tables.auto_gear_speed_down.bits(),
            )
        },
        || {
            drawing.scene.scratch.set(over_top(race));
            if !shifts::above_first(gear) {
                return Ok(false);
            }
            let (rb, rl) = (
                ratio(&race.shared, gear)?,
                ratio(&race.shared, gear.wrapping_sub(1))?,
            );
            if mul_div_status(
                drawing.scene.scratch.get(),
                rb,
                rl,
                GEAR.shift_ratio_figure.bits(),
            ) & 0x100
                == 0
            {
                return Ok(false);
            }
            let point = if shifts::down_point_clutch(race.shared.car.brake.get()) {
                GEAR.shift_down_clutch_point.bits()
            } else {
                GEAR.shift_down_point.bits()
            };
            Ok(below(drawing.scene.scratch.get(), point))
        },
        || {
            below(
                race.shared.car.local_velocity[2].bits(),
                race.shared.physics_tables.auto_gear_speed_up.bits(),
            )
        },
    )
}

/// The steps on `r`.
// Converted: `rc2k_core::physics::gear_shifts` (the engine's sums and compares stay x87).
pub fn auto_gearbox(race: &Race) -> Result<(), Refused> {
    race.shared.car.gear_changed.set(0);
    let mut to = 'first: {
        if bit(&race.shared, race.shared.car.gear.get())? {
            break 'first Shift::Neutral;
        }
        if race.shared.car.gear.get() != 0
            && below(
                race.shared.car.engine_speed.bits(),
                race.shared.physics_tables.idle_engine_speed.bits(),
            )
        {
            break 'first Shift::Neutral;
        }
        if race.shared.car.gear.get() == 0 {
            if mul_status(
                race.shared.car.engine_speed_limit.bits(),
                race.shared.physics_tables.auto_gear_engine_share.bits(),
                race.shared.car.engine_speed.bits(),
            ) & 0x100
                == 0
            {
                return Ok(());
            }
            break 'first if race.shared.physics_tables.input_gear_wanted.get() == u32::MAX {
                Shift::Reverse
            } else {
                Shift::Start
            };
        }
        if race.shared.car.forced_neutral.get() != 0 {
            break 'first Shift::Neutral;
        }
        if race.shared.car.gear_change_hold.get() != 0 {
            race.shared
                .car
                .gear_change_hold
                .set(race.shared.car.gear_change_hold.get().wrapping_sub(1));
            return Ok(());
        }
        if race.shared.car.gear_change_blocked.get() != 0 {
            return Ok(());
        }
        tests(race)?
    };
    loop {
        to = match to {
            Shift::Up => up(race)?,
            Shift::Down => down(race)?,
            Shift::Neutral => {
                neutral(race);
                Shift::Done
            }
            Shift::Start => {
                pick_start_gear(race)?;
                Shift::Done
            }
            Shift::Reverse => {
                reverse(race, false)?;
                Shift::Done
            }
            Shift::Done => return Ok(()),
        };
    }
}
