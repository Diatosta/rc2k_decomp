//! The wheel grit bursts' tuning.

use crate::memory::session::{PlainDword, PlainFloat};

impl BurstTuning {
    /// The half's (`n` 0 or 1) rate, count and entry randoms.
    pub fn randoms(&self, n: usize) -> &[PlainDword; 3] {
        if n == 0 { &self.first } else { &self.second }
    }
}

/// One burst emitter's factors and randoms (`emit_wheel_grit_bursts`' and `emit_bursts_kind_3`'s).
#[derive(rc2k_derive::Cells)]
pub struct BurstTuning {
    /// The strength's factor
    pub strength_k: PlainFloat,
    /// The first half's rate, count and entry randoms
    pub first: [PlainDword; 3],
    /// The blend's two factors
    pub blend_k: [PlainFloat; 2],
    /// The second half's rate, count and entry randoms
    pub second: [PlainDword; 3],
}

impl BurstTuning {
    /// The values the game starts with.
    pub const fn first() -> Self {
        BurstTuning {
            strength_k: PlainFloat::new(-0.25),
            first: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
            ],
            blend_k: [PlainFloat::new(-0.25), PlainFloat::new(-0.05)],
            second: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
            ],
        }
    }
}
