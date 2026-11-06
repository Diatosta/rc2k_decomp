//! `physics::gearbox_step` (0x43C15C): the gearbox's turn. Unless `g_car` +0xA8 is set,
//! or (with `dword_62389C` not 1) the start timer +0x14 is below 0x320 (by its sign): in
//! grade mode 2 `physics::manual_gearbox`; with a gear asked for (+0x468 bits 8 .. 11
//! not 0xE) `physics::take_asked_gear` and it ends there; else by +0x38 (`unk_71A010`)
//! 1 `manual_gearbox`, 2 `physics::auto_gearbox`, other `manual_gearbox` (by
//! `physics::manual_gearbox_call`). Then, after a change (+0x238 1), the engine speed x
//! `flt_613E9C`; `physics::wheels_from_revs`.
//!
//! In:  nothing
//! Out: the registers as the steps leave them; no flags read (the callers return)
//!
//! Callers: `sub_43CB14` (under `sub_43B635`), `sub_421848`. Refuses what the steps refuse;
//! what was written is put back.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::x87;
use crate::physics::auto_gearbox::auto_gearbox;
use crate::physics::car_store::CAR;
use crate::physics::gearbox::*;
use crate::physics::manual_gearbox::manual_gearbox;
use crate::physics::tables_store::PHYSICS;
use crate::physics::take_asked_gear::take_asked_gear;
use crate::race::owner_course::Course;
use crate::race::vars_store::RACE_VARS;

/// The steps on `r`.
// Converted: `rc2k_core::physics::gearbox::gearbox_turn`.
pub fn gearbox_step(course: &Course) -> Result<(), Refused> {
    use rc2k_core::physics::gearbox::{GearboxTurn, gearbox_turn};
    let turn = gearbox_turn(
        CAR.finish_flag.get(),
        || RACE_VARS.run_kind.get(),
        || CAR.run_ticks.get(),
        || CAR.driver_kind.get(),
        || CAR.input_keys.get(),
        || CAR.gearbox_kind.get(),
    );
    match turn {
        GearboxTurn::Idle => {}
        GearboxTurn::Manual => manual_gearbox(course)?,
        GearboxTurn::Asked => {
            { take_asked_gear(course) }?;
            return Ok(());
        }
        GearboxTurn::Auto => auto_gearbox(course)?,
    }
    if CAR.gear_changed.get() == 1 {
        CAR.engine_speed.set_bits(x87::mul(
            CAR.engine_speed.bits(),
            PHYSICS.engine_ease.bits(),
        ));
    }
    wheels()
}
