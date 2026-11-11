//! `physics::move_slots` (0x462206): `slot_count` = the number of the eight
//! `g_raceSlots` (0x78 bytes each) in use (+0 not 0), each moved by
//! `physics::move_slot`.
//!
//! Caller: `drive::animate_stage` (0x45E9CF), from race init's loop and `drive::replay_step`.
//! Refuses (before anything) what `move_slot` refuses for a slot in use.

use crate::errors::Refused;
use crate::physics::move_slot::move_slot;
use crate::physics::step_families::SlotInputs;

/// The eight slots, those in use moved, `slot_count` the number done so far.
pub fn move_slots(c: &SlotInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let work = c.work;
    let slots = &work.race_slots;
    let mut n = 0u32;
    for slot in slots {
        if slot.part.get() != 0 {
            n += 1;
            work.slot_count.set(n);
            move_slot(c, slot, scene)?;
        }
    }
    work.slot_count.set(n);
    Ok(())
}
