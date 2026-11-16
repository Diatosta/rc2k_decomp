//! The 3D primary and secondary card switches.

use crate::memory::session::PlainDword;

/// Whether a 3D card was found.
#[derive(rc2k_derive::Cells)]
pub struct Cards {
    /// A 3D primary card.
    pub primary: PlainDword,
    /// A 3D secondary card.
    pub secondary: PlainDword,
}

impl Cards {
    /// The switches.
    pub const fn new() -> Self {
        Cards {
            primary: PlainDword::new(0),
            secondary: PlainDword::new(0),
        }
    }
}
