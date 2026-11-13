//! The surface-locked switch.

use crate::session::PlainDword;

/// The surface-locked switch.
#[derive(rc2k_derive::Cells)]
pub struct Locked {
    /// 1 while the screen's surface is locked for drawing.
    pub surface: PlainDword,
}

impl Locked {
    pub const fn new() -> Self {
        Locked {
            surface: PlainDword::new(0),
        }
    }
}
