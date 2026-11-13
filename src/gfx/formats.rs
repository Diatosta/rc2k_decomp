//! The pixel formats' work variables (format numbers, scratch bytes, component format), out
//! of RAL.EXE's image: IDA 0x4F3F78 ..
//! 0x4F3FC4. The row table follows (`ROW_TABLE`).

use crate::session::{PlainBytes, PlainDword};

/// A pixel layout's number and its masks' summary (`describe_format` picks the number of
/// the layout whose bits the masks match).
#[derive(rc2k_derive::Cells)]
pub struct Numbered {
    /// The layout's number.
    pub number: PlainDword,
    /// The layout (the three masks' top bits as one dword).
    pub layout: PlainDword,
}

impl Numbered {
    const fn new(number: u32, layout: u32) -> Self {
        Self {
            number: PlainDword::new(number),
            layout: PlainDword::new(layout),
        }
    }
}

/// The pixel formats' variables.
#[derive(rc2k_derive::Cells)]
pub struct Formats {
    /// `Formats.numbers[0].number`: six of (number, layout).
    pub numbers: [Numbered; 6],
    /// `dword_4F3FA8`: four bytes `components_to_pixel` shifts the components out of.
    pub scratch: PlainBytes<4>,
    /// `unk_4F3FAC`: the component format, a pixel format record (layout, four masks, number).
    pub component_format: [PlainDword; 6],
}

impl Formats {
    /// The pixel formats' variables; the first values are the image's.
    pub const fn new() -> Self {
        Formats {
            numbers: [
                Numbered::new(1, 0xe0f8fbfd),
                Numbered::new(2, 0xe0f7fafd),
                Numbered::new(3, 0xf0f7fafd),
                Numbered::new(4, 0xf0f4f8fc),
                Numbered::new(5, 0xe0000000),
                Numbered::new(6, 0x80000000),
            ],
            scratch: PlainBytes::new([0; 4]),
            component_format: [const { PlainDword::new(0) }; 6],
        }
    }
}
