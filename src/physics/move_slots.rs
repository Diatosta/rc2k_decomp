//! `physics::move_slots` (0x462206): `slot_count` = the number of the eight
//! `g_raceSlots` (0x78 bytes each) in use (+0 not 0), each moved by
//! `physics::move_slot`.
//!
//! Caller: `drive::animate_stage` (0x45E9CF), from race init's loop and `drive::replay_step`.
//! Refuses (before anything) what `move_slot` refuses for a slot in use.

use crate::errors::Refused;
use crate::physics::move_slot::move_slot;
use crate::physics::step_inputs::StepInputs;

/// The eight slots, those in use moved, `slot_count` the number done so far.
pub fn move_slots(c: &StepInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let shared = c.shared;
    let slots = &shared.work.race_slots;
    let mut n = 0u32;
    for slot in slots {
        if slot.part.get() != 0 {
            n += 1;
            shared.work.slot_count.set(n);
            move_slot(c, slot, scene)?;
        }
    }
    shared.work.slot_count.set(n);
    Ok(())
}
