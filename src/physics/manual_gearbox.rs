//! `physics::manual_gearbox` (0x43C2AE): the gearbox by the keys (`dword_6138A0` up,
//! `dword_6138A4` down). +0x238 0. Then, in this order (`g_car` fields):
//! - grade mode (+0x25C) 2: the start-line rules (below);
//! - the gear refused by `physics::gear_bit`, or +0x430 set: neutral;
//! - the hold +0x1CC not 0: it goes down, nothing else;
//! - +0x42C set: nothing; +0x38 (`unk_71A010`) set: the start-line rules;
//! - up: from reverse neutral, from neutral `physics::pick_start_gear`, at the top
//!   nothing; else `physics::wheels_from_revs`, `physics::higher_free_gear` (none:
//!   nothing), set, `physics::revs_from_wheels`, settled;
//! - down: from 1 neutral, from neutral into reverse (unless `gear_bit` refuses -1: the
//!   engine speed x 0.25, `wheels_from_revs`, settled), in reverse nothing; else
//!   `wheels_from_revs`, `physics::lower_free_gear` (none: neutral), set,
//!   `revs_from_wheels`; an engine speed then not below `flt_614104` takes the gear back
//!   up (`revs_from_wheels`; the hold 20, +0x238 -1, +0x23C 1), else settled.
//!
//! The start-line rules: down asked: neutral into reverse, a forward gear to neutral; up
//! asked: neutral `pick_start_gear`, reverse to neutral; then, with +0x254 clear, the
//! automatic's tests (in gear 1 or more: the engine over its top by
//! `physics::revs_from_speed`, the engine speed put back, not below `flt_5CB958`: up; in
//! gear 2 or more: down when the next free lower gear's share is below `flt_5CB964` and
//! the engine over its top below `flt_5CB960` / `flt_5CB95C` by +0x63C).
//!
//! In:  nothing
//! Out: the registers as the steps leave them; no flags read (the callers jump on)
//!
//! Callers: `physics::gearbox_step`, the one-call `physics::manual_gearbox_call`. Refuses
//! what the calls refuse, and a gear outside -1 ..= 7 where a ratio is read; what was
//! written is put back.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::x87;
use crate::math::x87_run::mul_div_status;
use crate::physics::car_store::CAR;
use crate::physics::gear_store::GEAR;
use crate::physics::gearbox::*;
use crate::physics::higher_free_gear::higher_free_gear;
use crate::physics::lower_free_gear::lower_free_gear;
use crate::physics::pick_start_gear::pick_start_gear;
use crate::physics::tables_store::PHYSICS;
use crate::race::owner_course::Course;
use crate::scene::draw_store::DRAW;
use rc2k_core::physics::gear_shifts::{self as shifts, Shift};

/// Up (0x43C340).
fn up(course: &Course) -> Result<Shift, Refused> {
    let gear = CAR.gear.get();
    if let Some(shift) = shifts::manual_up(gear, || PHYSICS.gear_count.get()) {
        return Ok(shift);
    }
    wheels()?;
    let Some(to) = { higher_free_gear(gear) }? else {
        return Ok(Shift::Done);
    };
    CAR.gear.set(to);
    revs(course)?;
    settle();
    Ok(Shift::Done)
}

/// Down (0x43C379).
fn down(course: &Course) -> Result<Shift, Refused> {
    let gear = CAR.gear.get();
    if let Some(shift) = shifts::manual_down(gear) {
        return Ok(shift);
    }
    wheels()?;
    let Some(to) = { lower_free_gear(gear) }? else {
        return Ok(Shift::Neutral);
    };
    CAR.gear.set(to);
    revs(course)?;
    if !below(CAR.engine_speed.bits(), PHYSICS.rev_limit.bits()) {
        CAR.gear.set(CAR.gear.get().wrapping_add(1));
        revs(course)?;
        CAR.gear_change_hold.set(shifts::HOLD);
        CAR.gear_changed.set(u32::MAX);
        CAR.gear_changed_2.set(1);
        return Ok(Shift::Done);
    }
    settle();
    Ok(Shift::Done)
}

/// Into reverse (0x43C433; with `quarter` the engine speed x 0.25 first, as here; the
/// automatic's 0x43C74E has no such step).
pub fn reverse(course: &Course, quarter: bool) -> Result<(), Refused> {
    if bit(u32::MAX)? {
        return Ok(());
    }
    CAR.gear.set(u32::MAX);
    if quarter {
        CAR.engine_speed.set_bits(x87::mul(
            CAR.engine_speed.bits(),
            course.numbers.quarter.bits(),
        ));
    }
    wheels()?;
    settle();
    Ok(())
}

/// The start-line rules (0x43C48F).
fn start_line() -> Result<Shift, Refused> {
    let gear = CAR.gear.get();
    if let Some(shift) = shifts::start_line_keys(gear, PHYSICS.input_gear_down.get() != 0, || {
        PHYSICS.input_gear_up.get() != 0
    }) {
        return Ok(shift);
    }
    if CAR.all_wheels_off.get() != 0 {
        return Ok(Shift::Done);
    }
    if shifts::forward(gear) {
        let kept = CAR.engine_speed.bits();
        revs_speed()?;
        DRAW.scratch.set(over_top());
        CAR.engine_speed.set_bits(kept);
        if !below(DRAW.scratch.get(), GEAR.shift_up_point.bits()) {
            return Ok(Shift::Up);
        }
    }
    if !shifts::above_first(gear) {
        return Ok(Shift::Done);
    }
    DRAW.scratch.set(over_top());
    let Some(lower) = { lower_free_gear(gear) }? else {
        return Ok(Shift::Done);
    };
    let (rb, rc) = (ratio(gear)?, ratio(lower)?);
    if mul_div_status(DRAW.scratch.get(), rb, rc, GEAR.shift_ratio_figure.bits()) & 0x100 == 0 {
        return Ok(Shift::Done);
    }
    let point = if shifts::down_point_clutch(CAR.brake.get()) {
        GEAR.shift_down_clutch_point.bits()
    } else {
        GEAR.shift_down_point.bits()
    };
    if !below(DRAW.scratch.get(), point) {
        return Ok(Shift::Done);
    }
    Ok(Shift::Down)
}

/// The steps on `r`.
// Converted: `rc2k_core::physics::gear_shifts` (the engine's sums and compares stay x87).
pub fn manual_gearbox(course: &Course) -> Result<(), Refused> {
    CAR.gear_changed.set(0);
    let mut to = 'first: {
        if CAR.driver_kind.get() == 2 {
            break 'first start_line()?;
        }
        let gear = CAR.gear.get();
        if bit(gear)? || CAR.forced_neutral.get() != 0 {
            break 'first Shift::Neutral;
        }
        if CAR.gear_change_hold.get() != 0 {
            CAR.gear_change_hold
                .set(CAR.gear_change_hold.get().wrapping_sub(1));
            return Ok(());
        }
        if CAR.gear_change_blocked.get() != 0 {
            return Ok(());
        }
        if CAR.gearbox_kind.get() != 0 {
            break 'first start_line()?;
        }
        shifts::manual_keys(PHYSICS.input_gear_up.get() != 0, || {
            PHYSICS.input_gear_down.get() != 0
        })
    };
    loop {
        to = match to {
            Shift::Up => up(course)?,
            Shift::Down => down(course)?,
            Shift::Neutral => {
                neutral();
                Shift::Done
            }
            Shift::Start => {
                pick_start_gear(course)?;
                Shift::Done
            }
            Shift::Reverse => {
                reverse(course, true)?;
                Shift::Done
            }
            Shift::Done => return Ok(()),
        };
    }
}
