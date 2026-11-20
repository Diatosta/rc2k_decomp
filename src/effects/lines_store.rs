//! The block lines' work.

use crate::memory::session::{PlainDword, PlainFloat};

/// The block lines' work (`lines`).
#[derive(rc2k_derive::Cells)]
pub struct LinesWork {
    /// The eight values a block's `factors` start with
    pub block_start: [PlainFloat; 8],
    /// The last line's x, y, z differences
    pub difference: [PlainDword; 3],
    /// The block being measured. Not used: nothing reads it
    pub current: PlainDword,
    /// Measure every block
    pub every: PlainDword,
    /// Make the shares
    pub shares: PlainDword,
    /// A marked block was measured
    pub any: PlainDword,
}

impl LinesWork {
    /// The values the game starts with.
    pub const fn new() -> Self {
        LinesWork {
            block_start: [
                PlainFloat::new(1.0),
                PlainFloat::new(1.0),
                PlainFloat::new(1.0),
                PlainFloat::new(1.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            difference: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
            ],
            current: PlainDword::new(0x0),
            every: PlainDword::new(0x0),
            shares: PlainDword::new(0x0),
            any: PlainDword::new(0x0),
        }
    }
}
