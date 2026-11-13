//! The PNG header's fields: `decode_png` writes them from the file's header chunk and, without one
//! in the file, reads what the last file left.

use crate::session::{PlainByte, PlainBytes, PlainDword};

/// The header of the PNG being decoded.
#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells)]
pub struct PngHeader {
    /// The picture's width.
    pub width: PlainDword,
    /// The picture's height.
    pub height: PlainDword,
    /// The header's byte after the colour type (the compression method).
    pub compression: PlainByte,
    /// Pads the byte to the next dword (Rust's alignment).
    pub _pad_align: PlainBytes<3>,
    /// The colour type.
    pub colour_type: PlainDword,
}

impl PngHeader {
    /// The header.
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
