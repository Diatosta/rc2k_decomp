//! The mark slots and their tuning.

/// One mark slot: a sprite that lives a number of ticks.
pub struct MarkSlot {
    /// The ticks left (0 or less: free; `effects::add_mark_slot` sets 0x3C).
    pub life: u32,
    /// Which point of the marks' points (12 bytes each) the slot sits at.
    pub point: u32,
    /// The matrix it is turned by, as a handle (`VIEW_PLACE_MATRIX`); 0 for none.
    pub matrix: u32,
    /// 0.
    pub spare: u32,
    /// A float of the slot's own random (`SlotTuning.base` plus the random's low word).
    pub size: f32,
}

/// The mark slots.
pub struct MarkSlots {
    /// 0x20 slots of 0x14 bytes.
    pub slots: [MarkSlot; 32],
    /// The slot's point turned, then its projected place (z, y, x; then x, y).
    pub work: [f32; 3],
}

impl MarkSlots {
    pub const fn new() -> Self {
        Self {
            slots: [const {
                MarkSlot {
                    life: 0,
                    point: 0,
                    matrix: 0,
                    spare: 0,
                    size: 16384.0,
                }
            }; 32],
            work: [0.0; 3],
        }
    }
}

/// The mark slots' constants and randoms.
pub struct SlotTuning {
    /// Half the slot square's size on the screen.
    pub half: f32,
    /// What a slot's point is scaled by: z times the first, y less the second times the third, x
    /// times the fourth.
    pub place_k: [f32; 4],
    /// `effects::draw_mark_slots`' random (times 0xC5 plus 0x5D) choosing a corner picture.
    pub draw_random: u32,
    /// What one over the depth is multiplied by (the vertices' rhw).
    pub rhw_k: f32,
    /// `effects::add_mark_slot` first takes it, then the random (nothing keeps it).
    pub first_float: f32,
    /// `effects::add_mark_slot`' random (times 0xC5 plus 0x5D).
    pub slot_random: u32,
    /// Added to a new slot's random (the slot's size).
    pub base: f32,
}

impl SlotTuning {
    pub const fn new() -> Self {
        Self {
            half: 0.0,
            place_k: [65.536, 700.0, 65.536, 65.536],
            draw_random: 0x0,
            rhw_k: 256.0,
            first_float: 32768.0,
            slot_random: 0x0,
            base: 16384.0,
        }
    }
}
