//! The pixel formats' work variables (format numbers, scratch bytes, component format), out
//! of RAL.EXE's image (phase 5; typed in phase 7, `notes/conversion.md`): IDA 0x4F3F78 ..
//! 0x4F3FC4. The row table follows (`gfx::fill_row_table::ROW_TABLE`).

use crate::session::{Bytes, Dword};

/// IDA's address of the block's first byte.
pub const START: usize = 0x4F3F78;
/// The block's length.
pub const LEN: usize = 0x4F3FC4 - START;

/// A pixel layout's number and its masks' summary (`describe_format` picks the number of
/// the layout whose bits the masks match).
#[repr(C)]
pub struct Numbered {
    /// The layout's number.
    pub number: Dword,
    /// The layout (the three masks' top bits as one dword).
    pub layout: Dword,
}

impl Numbered {
    const fn new(number: u32, layout: u32) -> Self {
        Self {
            number: Dword::new(number),
            layout: Dword::new(layout),
        }
    }
}

/// The pixel formats' variables.
#[repr(C)]
pub struct Formats {
    /// `dword_4F3F78`: six of (number, layout).
    pub numbers: [Numbered; 6],
    /// `dword_4F3FA8`: four bytes `components_to_pixel` shifts the components out of.
    pub scratch: Bytes<4>,
    /// `unk_4F3FAC`: the component format, a pixel format record (layout, four masks, number).
    pub component_format: [Dword; 6],
}

/// The pixel formats' variables; the first values are the image's.
pub static FORMATS: Formats = Formats {
    numbers: [
        Numbered::new(1, 0xe0f8fbfd),
        Numbered::new(2, 0xe0f7fafd),
        Numbered::new(3, 0xf0f7fafd),
        Numbered::new(4, 0xf0f4f8fc),
        Numbered::new(5, 0xe0000000),
        Numbered::new(6, 0x80000000),
    ],
    scratch: Bytes::new([0; 4]),
    component_format: [const { Dword::new(0) }; 6],
};

/// Where the block is (`moved::MOVED`'s home for IDA 0x4F3F78).
pub fn home() -> usize {
    crate::pointers::game_address(&FORMATS as *const Formats) as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&FORMATS, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(offset_of!(Formats, scratch), 0x4F3FA8 - START);
        assert_eq!(offset_of!(Formats, component_format), 0x4F3FAC - START);
        assert_eq!(
            crate::moved::place(START),
            FORMATS.numbers[0].number.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x4F3FA8),
            FORMATS.scratch.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x4F3FAC),
            (crate::pointers::game_address(FORMATS.component_format.as_ptr())) as usize
        );
    }
}
