//! The marks' kind.

use crate::memory::session::PlainDword;

/// The marks' kind (`effects::kind_values`).
#[derive(rc2k_derive::Cells)]
pub struct MarksKind {
    /// The kind of the marks (the surface / weather: 0 .. 0x21), set at `effects::init_marks`.
    pub kind: PlainDword,
}

impl MarksKind {
    pub const fn new() -> Self {
        Self {
            kind: PlainDword::new(0x0),
        }
    }
}
