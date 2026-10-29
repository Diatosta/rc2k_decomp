//! `effects::add_mark_slot` (0x4C99F4): the last free slot of `g_markSlots` (from 0x1F down,
//! free when its first dword is 0 or less) taken: 0x3C, eax, esi, 0, and a float of the
//! slot's own random (`dword_93540A` times 0xC5 plus 0x5D, its low word, plus
//! `flt_93540E`). Nothing when none is free.
//!
//! In:  eax, esi what the slot holds
//! Out: every register kept
//!
//! One caller, `sub_4C969A` (the marks' frame). The float first gets `flt_935406`, then the
//! random at once: only the second is kept.

use crate::effects::mark_slots::{SLOTS, TUNING};
use crate::math::x87;

const TIME: u32 = 0x3C;

/// The Rust version (see the header).
pub fn add_mark_slot(what: u32, at_esi: u32) {
    for slot in SLOTS.slots.iter().rev() {
        if slot.life.get() as i32 > 0 {
            continue;
        }
        let random = TUNING
            .slot_random
            .get()
            .wrapping_mul(0xC5)
            .wrapping_add(0x5D);
        TUNING.slot_random.set(random);
        let float = x87::int_plus((random & 0xFFFF) as i32, TUNING.base.bits());
        slot.life.set(TIME);
        slot.point.set(what);
        slot.matrix.set(at_esi);
        slot.spare.set(0);
        slot.size.set_bits(float);
        return;
    }
}
