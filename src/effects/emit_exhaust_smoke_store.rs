//! The exhaust smoke's set.

use crate::memory::session::{PlainDword, PlainFloat};

/// Kind 0xD's emitter variables (`emit_exhaust_smoke`).
#[derive(rc2k_derive::Cells)]
pub struct ExhaustSet {
    /// The place's lift
    pub lift: PlainFloat,
    /// The strength's factor
    pub strength_k: PlainFloat,
    /// The motion's factors (x, y, z)
    pub motion_k: [PlainFloat; 3],
    /// The drift's random
    pub drift_random: PlainDword,
    /// The spin's random
    pub spin_random: PlainDword,
    /// The size's factor, base and scale
    pub size_k: [PlainFloat; 3],
}

impl ExhaustSet {
    /// The values the game starts with.
    pub const fn new() -> Self {
        ExhaustSet {
            lift: PlainFloat::new(65536.0),
            strength_k: PlainFloat::new(0.0001),
            motion_k: [
                PlainFloat::new(-2048.0),
                PlainFloat::new(-2048.0),
                PlainFloat::new(-2048.0),
            ],
            drift_random: PlainDword::new(0x0),
            spin_random: PlainDword::new(0x0),
            size_k: [
                PlainFloat::new(65536.0),
                PlainFloat::new(2048.0),
                PlainFloat::new(0.2),
            ],
        }
    }
}
