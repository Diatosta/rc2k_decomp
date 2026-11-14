//! The gearbox's turn. Unless the car's `finish_flag` is set, or (with `run_kind` not 1) the
//! start timer is below 0x320: in grade mode 2 `physics::manual_gearbox`; with a gear asked for
//! (`gear_asked` bits not 0xE) `physics::take_asked_gear` and it ends there; otherwise by
//! `gearbox_kind`: 1 `manual_gearbox`, 2 `physics::auto_gearbox`, other `manual_gearbox`. Then,
//! after a change, the engine speed x `engine_ease` and `physics::wheels_from_revs`.
//!
//! Callers: `physics::drive_force` (under `physics::car_physics_step`),
//! `physics::test_drive_run`. Refuses what the steps refuse; what was written is put back.

use crate::errors::Refused;
use crate::math::x87;
use crate::physics::auto_gearbox::auto_gearbox;
use crate::physics::gearbox::*;
use crate::physics::manual_gearbox::manual_gearbox;
use crate::physics::take_asked_gear::take_asked_gear;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;

/// The steps on `r`.
// Core: `rc2k_core::physics::gearbox::gearbox_turn`.
pub fn gearbox_step(
    numbers: &Numbers,
    shared: &RaceShared,
    gear_figures: &crate::physics::gear_store::GearFigures,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    use rc2k_core::physics::gearbox::{GearboxTurn, gearbox_turn};
    let turn = gearbox_turn(
        shared.car().finish_flag.get(),
        || shared.vars.get().run_kind.get(),
        || shared.car().run_ticks.get(),
        || shared.car().driver_kind.get(),
        || shared.car().input_keys.get(),
        || shared.car().gearbox_kind.get(),
    );
    match turn {
        GearboxTurn::Idle => {}
        GearboxTurn::Manual => manual_gearbox(
            shared.car(),
            shared.physics_tables.get(),
            numbers,
            gear_figures,
            scene,
        )?,
        GearboxTurn::Asked => {
            { take_asked_gear(numbers, shared) }?;
            return Ok(());
        }
        GearboxTurn::Auto => auto_gearbox(
            shared.car(),
            shared.physics_tables.get(),
            numbers,
            gear_figures,
            scene,
        )?,
    }
    if shared.car().gear_changed.get() == 1 {
        shared.car().engine_speed.set(x87::mul(
            &shared.car().engine_speed,
            &shared.physics_tables.get().engine_ease,
        ));
    }
    wheels(shared.car(), shared.physics_tables.get())
}
