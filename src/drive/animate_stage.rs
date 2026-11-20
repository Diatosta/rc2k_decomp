//! `drive::animate_lit_faces`, then `physics::move_slots`.
//!
//! Refuses what the two refuse; what was written before a refusal is put back first.

use crate::drive::animate_lit_faces::animate_lit_faces;
use crate::errors::Refused;
use crate::physics::move_slots::move_slots;
use crate::race::owner::Race;

pub(super) fn animate_stage(
    race: &Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    animate_lit_faces(race, scene)?;
    move_slots(&crate::physics::step_inputs::StepInputs::of(race).slots())?;
    Ok(())
}
