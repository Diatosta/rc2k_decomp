//! The error list loaded from the course's file (IDA 0x637660 .. 0x63967C; phase 7,
//! typed): the file's 0x2000 bytes (`race::level_init::load_error_list`) and the seven
//! dwords after them. `#[repr(C)]` with the image's layout, over the race block's bytes
//! (`ERROR_LIST`). Names are readings unless IDA's.

use crate::session::{Bytes, Dword};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ErrorList {
    /// +0x0 (0x637660): `unk_637660`: the error list read from the file named at 0x60AE02 (`race::load_error_list`).
    pub list: Bytes<0x2000>,
    /// +0x2000 (0x639660): `dword_639660` .. `dword_639678`: not worked out.
    pub unknown_2000: [Dword; 7],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
pub static ERROR_LIST: ErrorList =
    crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START));

/// IDA 0x637660, the first byte of the piece.
pub const START: usize = 0x637660;
/// IDA 0x63967C, the end of the piece.
pub const END: usize = 0x63967C;

const _: () = assert!(END - START == core::mem::size_of::<ErrorList>());

/// Where IDA 0x637660 is now.
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(&ERROR_LIST))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&ERROR_LIST, START, END);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            crate::moved::place(END - 1),
            (piece_addr() + (END - START) as u32 - 1) as usize
        );
        assert_eq!(offset_of!(ErrorList, list), 0x637660 - START);
        assert_eq!(offset_of!(ErrorList, unknown_2000), 0x639660 - START);
    }
}
