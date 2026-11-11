//! `drive::animate_stage` (0x45E9CA): `drive::animate_lit_faces`, then `physics::move_slots`.
//!
//! Callers: race init's loop (`drive::race_cars_step`, 0x46093E) and `drive::replay_step`. Refuses
//! what the two refuse; what was written before a refusal is put back first.

use crate::drive::animate_lit_faces::animate_lit_faces;
use crate::errors::Refused;
use crate::physics::move_slots::move_slots;
use crate::race::owner::Race;

pub fn animate_stage(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    animate_lit_faces(race, scene)?;
    move_slots(
        &crate::physics::step_inputs::StepInputs::of(race).slots(),
        scene,
    )?;
    Ok(())
}
