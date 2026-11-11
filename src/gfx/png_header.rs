//! The PNG header's fields, out of RAL.EXE's image: IDA 0xA8C190 .. 0xA8C1A4. `decode_png` writes
//! them from the file's header chunk and, without one in the file, reads what the last file left.

use crate::session::{PlainByte, PlainBytes, PlainDword};

/// IDA's address of the block's first byte.
pub const START: usize = 0xA8C190;
/// The block's length.
pub const LEN: usize = 0xA8C1A4 - START;

/// The header of the PNG being decoded.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct PngHeader {
    /// `dword_A8C190`: the picture's width.
    pub width: PlainDword,
    /// `dword_A8C194`: the picture's height.
    pub height: PlainDword,
    /// `0xA8C198`, `0xA8C199`: nothing reads or writes them (a gap).
    pub _gap_8: PlainBytes<2>,
    /// `byte_A8C19A`: the header's byte after the colour type (the compression method).
    pub compression: PlainByte,
    /// `0xA8C19B` .. `0xA8C19F`: nothing reads or writes them (a gap).
    pub _gap_b: PlainBytes<5>,
    /// `dword_A8C1A0`: the colour type.
    pub colour_type: PlainDword,
}

impl PngHeader {
    /// The header; the first values are the image's.
    pub const fn new() -> Self {
        PngHeader {
            width: PlainDword::new(0),
            height: PlainDword::new(0),
            _gap_8: PlainBytes::new([0; 2]),
            compression: PlainByte::new(0),
            _gap_b: PlainBytes::new([0; 5]),
            colour_type: PlainDword::new(0),
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
            &crate::gfx::owner::GFX.unchecked().png.header,
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
            crate::gfx::owner::GFX.unchecked().png.header.width.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0xA8C1A0),
            crate::gfx::owner::GFX
                .unchecked()
                .png
                .header
                .colour_type
                .addr() as usize
        );
    }
}
