//! The error list loaded from the course's file: the file's 0x2000 bytes
//! (`race::level_init::load_error_list`) and the seven dwords after them. The fields are in the
//! image's order.

use crate::memory::cells::ZeroCells;

use crate::memory::session::{PlainBytes, PlainDword};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ErrorList {
    /// The error list read from the file named in the course (`race::load_error_list`).
    pub list: PlainBytes<0x2000>,
    /// Unused.
    pub unknown_2000: [PlainDword; 7],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl ErrorList {
    pub const fn new() -> Self {
        Self::ZERO
    }
}
