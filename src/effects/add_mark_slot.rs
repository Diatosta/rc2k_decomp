//! `effects::add_mark_slot`: the last free slot of the mark slots (from 0x1F down, free
//! when its life is 0 or less) taken: life 0x3C, the caller's two values, 0, and a float of
//! the slot's own random (the slots' random times 0xC5 plus 0x5D, its low word, plus a
//! constant). Nothing when none is free.
//!
//! `what` is the marks' point the slot sits at and `matrix` its matrix handle
//! (`mark_slots::VIEW_PLACE_MATRIX`). The float first gets `SlotTuning.first_float`, then the
//! random at once: only the second is kept.

use crate::math::x87_float as bh;

const TIME: u32 = 0x3C;

pub fn add_mark_slot(what: u32, matrix: u32, effects: &crate::effects::owner::Effects) {
    let blocks = &effects.blocks;
    for slot in blocks.slots.slots.iter().rev() {
        if slot.life.get() as i32 > 0 {
            continue;
        }
        let random = blocks
            .slot_tuning
            .slot_random
            .get()
            .wrapping_mul(0xC5)
            .wrapping_add(0x5D);
        blocks.slot_tuning.slot_random.set(random);
        let float =
            bh::stf(bh::int((random & 0xFFFF) as i32) + bh::ld(blocks.slot_tuning.base.get()));
        slot.life.set(TIME);
        slot.point.set(what);
        slot.matrix.set(matrix);
        slot.spare.set(0);
        slot.size.set(float);
        return;
    }
}
