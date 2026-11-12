//! The error list loaded from the course's file (IDA 0x637660 .. 0x63967C; phase 7,
//! typed): the file's 0x2000 bytes (`race::level_init::load_error_list`) and the seven
//! dwords after them. `#[repr(C)]` with the image's layout, over the race block's bytes
//! (`ERROR_LIST`). Names are readings unless IDA's.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::{Bytes, Dword};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ErrorList {
    /// +0x0 (0x637660): `unk_637660`: the error list read from the file named at 0x60AE02 (`race::load_error_list`).
    pub list: Bytes<0x2000>,
    /// +0x2000 (0x639660): `dword_639660` .. `dword_639678`: used only by the unported `sub_427DB4` and dead code outside any function (0x43B07E ..); the port never reaches it.
    pub unknown_2000: [Dword; 7],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl ErrorList {
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x637660, the first byte of the piece.
pub const START: usize = 0x637660;
/// IDA 0x63967C, the end of the piece.
pub const END: usize = 0x63967C;

const _: () = assert!(END - START == core::mem::size_of::<ErrorList>());

/// The piece's host address (the alignment checks).
#[cfg(test)]
pub fn piece_addr() -> usize {
    core::ptr::from_ref(&race().frame.error_list).addr()
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &race().frame.error_list,
        );
    }

    #[test]
    fn fields_are_at_the_games_offsets() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(offset_of!(ErrorList, list), 0x0);
        assert_eq!(offset_of!(ErrorList, unknown_2000), 0x2000);
    }
}
