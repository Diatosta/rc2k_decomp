//! The error list loaded from the course's file (IDA 0x637660 .. 0x63967C; phase 7,
//! typed): the file's 0x2000 bytes (`race::level_init::load_error_list`) and the seven
//! dwords after them. `#[repr(C)]` with the image's layout, over the race block's bytes
//! (`ERROR_LIST`). Names are readings unless IDA's.

use crate::cells::ZeroCells;

use crate::session::{PlainBytes, PlainDword};

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ErrorList {
    /// +0x0 (0x637660): `unk_637660`: the error list read from the file named at 0x60AE02 (`race::load_error_list`).
    pub list: PlainBytes<0x2000>,
    /// +0x2000 (0x639660): `dword_639660` .. `dword_639678`: used only by the unported `sub_427DB4` and dead code outside any function (0x43B07E ..); the port never reaches it.
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
