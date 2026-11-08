//! The PNG header's fields, out of RAL.EXE's image (phase 5; typed in phase 7,
//! `notes/conversion.md`): IDA 0xA8C190 .. 0xA8C1A4. `decode_png` writes them from the
//! file's header chunk and, without one in the file, reads what the last file left.

use crate::session::{Byte, Bytes, Dword};

/// IDA's address of the block's first byte.
pub const START: usize = 0xA8C190;
/// The block's length.
pub const LEN: usize = 0xA8C1A4 - START;

/// The header of the PNG being decoded.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct PngHeader {
    /// `dword_A8C190`: the picture's width.
    pub width: Dword,
    /// `dword_A8C194`: the picture's height.
    pub height: Dword,
    /// `0xA8C198`, `0xA8C199`: nothing reads or writes them (a gap).
    pub _gap_8: Bytes<2>,
    /// `byte_A8C19A`: the header's byte after the colour type (the compression method).
    pub compression: Byte,
    /// `0xA8C19B` .. `0xA8C19F`: nothing reads or writes them (a gap).
    pub _gap_b: Bytes<5>,
    /// `dword_A8C1A0`: the colour type.
    pub colour_type: Dword,
}

impl PngHeader {
    /// The header; the first values are the image's.
    pub const fn new() -> Self {
        PngHeader {
            width: Dword::new(0),
            height: Dword::new(0),
            _gap_8: Bytes::new([0; 2]),
            compression: Byte::new(0),
            _gap_b: Bytes::new([0; 5]),
            colour_type: Dword::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(
            &crate::gfx::owner::GFX.png.header,
            START,
            START + LEN,
        );
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(offset_of!(PngHeader, compression), 0xA8C19A - START);
        assert_eq!(offset_of!(PngHeader, colour_type), 0xA8C1A0 - START);
        assert_eq!(
            crate::moved::place(START),
            crate::gfx::owner::GFX.png.header.width.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0xA8C1A0),
            crate::gfx::owner::GFX.png.header.colour_type.addr() as usize
        );
    }
}
