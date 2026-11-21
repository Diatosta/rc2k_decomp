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

pub(super) fn add_mark_slot(what: u32, matrix: u32, effects: &crate::effects::owner::Effects) {
    let blocks = &effects.blocks;
    let mut slots = blocks.slots.borrow_mut();
    let Some(slot) = slots
        .slots
        .iter_mut()
        .rev()
        .find(|slot| slot.life as i32 <= 0)
    else {
        return;
    };
    let mut tuning = blocks.slot_tuning.borrow_mut();
    let random = tuning.slot_random.wrapping_mul(0xC5).wrapping_add(0x5D);
    tuning.slot_random = random;
    let float = bh::stf(bh::int((random & 0xFFFF) as i32) + bh::ld(tuning.base));
    slot.life = TIME;
    slot.point = what;
    slot.matrix = matrix;
    slot.spare = 0;
    slot.size = float;
}
