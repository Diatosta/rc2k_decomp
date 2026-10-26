//! The error list loaded from the course's file (IDA 0x637660 .. 0x63967C; phase 7,
//! typed): the file's 0x2000 bytes (`race::level_init::load_error_list`) and the seven
//! dwords after them. `#[repr(C)]` with the image's layout, over the race block's bytes
//! (`ERROR_LIST`). Names are readings unless IDA's.

use crate::session::{Bytes, Dword};

#[repr(C)]
pub struct ErrorList {
    /// +0x0 (0x637660): `unk_637660`: the error list read from the file named at 0x60AE02 (`race::load_error_list`).
    pub list: Bytes<0x2000>,
    /// +0x2000 (0x639660): `dword_639660` .. `dword_639678`: not worked out.
    pub unknown_2000: [Dword; 7],
}

/// The piece: the race block's bytes at IDA 0x637660 seen as `ErrorList`.
pub struct InBlock;

impl core::ops::Deref for InBlock {
    type Target = ErrorList;
    fn deref(&self) -> &ErrorList {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); every field is an atomic or a cell, as the block's bytes were shared.
        unsafe { &*(crate::pointers::host(piece_addr()) as *const ErrorList) }
    }
}

pub static ERROR_LIST: InBlock = InBlock;

/// IDA 0x637660, the first byte of the piece.
pub const START: usize = 0x637660;
/// IDA 0x63967C, the end of the piece.
pub const END: usize = 0x63967C;

const _: () = assert!(END - START == core::mem::size_of::<ErrorList>());

/// Where IDA 0x637660 is now.
pub fn piece_addr() -> u32 {
    crate::moved::place(START) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_block() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            crate::moved::place(END - 1),
            (piece_addr() + (END - START) as u32 - 1) as usize
        );
        assert_eq!(offset_of!(ErrorList, list), 0x637660 - START);
        assert_eq!(offset_of!(ErrorList, unknown_2000), 0x639660 - START);
    }
}
