//! The mark slots and their tuning.

use crate::memory::session::{PlainDword, PlainFloat};

/// One mark slot: a sprite that lives a number of ticks.
#[derive(rc2k_derive::Cells)]
pub struct MarkSlot {
    /// The ticks left (0 or less: free; `effects::add_mark_slot` sets 0x3C).
    pub life: PlainDword,
    /// Which point of the marks' points (12 bytes each) the slot sits at.
    pub point: PlainDword,
    /// The matrix it is turned by, as a handle (`VIEW_PLACE_MATRIX`); 0 for none.
    pub matrix: PlainDword,
    /// 0.
    pub spare: PlainDword,
    /// A float of the slot's own random (`SlotTuning.base` plus the random's low word).
    pub size: PlainFloat,
}

/// The mark slots.
#[derive(rc2k_derive::Cells)]
pub struct MarkSlots {
    /// 0x20 slots of 0x14 bytes.
    pub slots: [MarkSlot; 32],
    /// The slot's point turned, then its projected place (z, y, x; then x, y).
    pub work: [PlainFloat; 3],
}

impl MarkSlots {
    pub const fn new() -> Self {
        Self {
            slots: [const {
                MarkSlot {
                    life: PlainDword::new(0),
                    point: PlainDword::new(0),
                    matrix: PlainDword::new(0),
                    spare: PlainDword::new(0),
                    size: PlainFloat::new(16384.0),
                }
            }; 32],
            work: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
        }
    }
}

/// The mark slots' constants and randoms.
#[derive(rc2k_derive::Cells)]
pub struct SlotTuning {
    /// Half the slot square's size on the screen.
    pub half: PlainFloat,
    /// What a slot's point is scaled by: z times the first, y less the second times the third, x
    /// times the fourth.
    pub place_k: [PlainFloat; 4],
    /// `effects::draw_mark_slots`' random (times 0xC5 plus 0x5D) choosing a corner picture.
    pub draw_random: PlainDword,
    /// What one over the depth is multiplied by (the vertices' rhw).
    pub rhw_k: PlainFloat,
    /// `effects::add_mark_slot` first takes it, then the random (nothing keeps it).
    pub first_float: PlainFloat,
    /// `effects::add_mark_slot`' random (times 0xC5 plus 0x5D).
    pub slot_random: PlainDword,
    /// Added to a new slot's random (the slot's size).
    pub base: PlainFloat,
}

impl SlotTuning {
    pub const fn new() -> Self {
        Self {
            half: PlainFloat::new(0.0),
            place_k: [
                PlainFloat::new(65.536),
                PlainFloat::new(700.0),
                PlainFloat::new(65.536),
                PlainFloat::new(65.536),
            ],
            draw_random: PlainDword::new(0x0),
            rhw_k: PlainFloat::new(256.0),
            first_float: PlainFloat::new(32768.0),
            slot_random: PlainDword::new(0x0),
            base: PlainFloat::new(16384.0),
        }
    }
}
