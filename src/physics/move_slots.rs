//! `slot_count` = the number of the eight `race_slots` (0x78 bytes each) in use, each moved by
//! `physics::move_slot`.
//!
//! Refuses (before anything) what `move_slot` refuses for a slot in use.

use crate::errors::Refused;
use crate::physics::move_slot::move_slot;
use crate::physics::step_families::SlotInputs;

/// The eight slots, those in use moved, `slot_count` the number done so far. Each slot is moved
/// as a copy and put back (also when its move is refused).
pub fn move_slots(c: &SlotInputs<'_>) -> Result<(), Refused> {
    let mut n = 0u32;
    let count = c.work.borrow().race_slots.len();
    for number in 0..count {
        let mut slot = c.work.borrow().race_slots[number].clone();
        if slot.part != 0 {
            n += 1;
            c.work.borrow_mut().slot_count = n;
            let moved = move_slot(c, &mut slot, number);
            c.work.borrow_mut().race_slots[number] = slot;
            moved?;
        }
    }
    c.work.borrow_mut().slot_count = n;
    Ok(())
}
