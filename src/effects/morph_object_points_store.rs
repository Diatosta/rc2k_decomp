//! The object morph's work.

use crate::memory::session::{PlainDword, PlainFloat};

/// The object morph's work.
#[derive(rc2k_derive::Cells)]
pub struct MorphWork {
    /// The block last taken for a settings block's points (`lines::set_up_blocks`)
    pub taken: PlainDword,
    /// The factors kept
    pub kept: [PlainFloat; 8],
}

impl MorphWork {
    /// The values the game starts with.
    pub const fn new() -> Self {
        MorphWork {
            taken: PlainDword::new(0x0),
            kept: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
        }
    }
}
