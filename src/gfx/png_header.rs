//! The PNG header's fields, out of RAL.EXE's image: IDA 0xA8C190 .. 0xA8C1A4. `decode_png` writes
//! them from the file's header chunk and, without one in the file, reads what the last file left.

use crate::session::{PlainByte, PlainBytes, PlainDword};

/// The header of the PNG being decoded.
#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells)]
pub struct PngHeader {
    /// `dword_A8C190`: the picture's width.
    pub width: PlainDword,
    /// `dword_A8C194`: the picture's height.
    pub height: PlainDword,
    /// `byte_A8C19A`: the header's byte after the colour type (the compression method).
    pub compression: PlainByte,
    /// Pads the byte to the next dword (Rust's alignment).
    pub _pad_align: PlainBytes<3>,
    /// `dword_A8C1A0`: the colour type.
    pub colour_type: PlainDword,
}

impl PngHeader {
    /// The header; the first values are the image's.
    pub const fn new() -> Self {
        PngHeader {
            width: PlainDword::new(0),
            height: PlainDword::new(0),
            compression: PlainByte::new(0),
            _pad_align: PlainBytes::new([0; 3]),
            colour_type: PlainDword::new(0),
        }
    }
}
