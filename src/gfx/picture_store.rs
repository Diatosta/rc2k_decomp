//! The picture loader's variables (the name, flags, size, rows and buffer of the picture, the list
//! its memory is on), out of RAL.EXE's image: IDA 0x4F49C8 .. 0x4F4A24. The texture grades that
//! follow in the image are `gfx::texture_pixels::GRADING`.

use crate::memory::MemoryList;
use crate::session::PlainDword;

/// IDA's address of the block's first byte.
pub const START: usize = 0x4F49C8;
/// The block's length.
pub const LEN: usize = 0x4F4A24 - START;

/// The picture buffer: where a picture file is decoded to, as 32-bit pixels of four 8-bit
/// components, before it is converted to the surface's format and drawn.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Picture {
    /// `dword_4F49C8`: the name of the file a screenshot is written to, or 0 for the
    /// numbered default. Not used: `gfx::write_screen_pcx` takes the name from its caller.
    pub name: PlainDword,
    /// `g_pictureFlags` (`dword_4F49CC`): flags of the picture; bits 8 .. 11 say what its
    /// pixels are (0x200 after a conversion to 16-bit pixels).
    pub flags: PlainDword,
    /// `dword_4F49D0`: the picture's width in pixels.
    pub width: PlainDword,
    /// `dword_4F49D4`: the picture's height in pixels.
    pub height: PlainDword,
    /// `dword_4F49D8`: the bytes of one row (width * 4, halved when the picture is converted
    /// to 16-bit pixels); 0xFFFFFFFD while a picture is being loaded.
    pub row_bytes: PlainDword,
    /// `g_pictureBuffer` (`dword_4F49DC`): the pointer to the buffer.
    pub buffer: PlainDword,
    /// `dword_4F49E0`: the buffer's size in bytes (0x1D4C00: 800 x 600 dwords).
    pub size: PlainDword,
    /// `Picture.list.head`: the list the buffer's memory is on (16 bytes).
    pub list: crate::memory::ListCells,
    /// `0x4F49F4` .. `0x4F4A24`: nothing reads or writes them (a gap).
    pub _gap_2c: [PlainDword; 12],
}

impl Picture {
    /// The picture's variables; the first values are the image's.
    pub const fn new() -> Self {
        Picture {
            name: PlainDword::new(0),
            flags: PlainDword::new(0),
            width: PlainDword::new(0),
            height: PlainDword::new(0),
            row_bytes: PlainDword::new(0),
            buffer: PlainDword::new(0),
            size: PlainDword::new(0),
            list: crate::memory::ListCells::new(MemoryList {
                head: 0,
                total: 0,
                cursor: 0,
                remaining: 0,
            }),
            _gap_2c: [const { PlainDword::new(0) }; 12],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::owner::gfx;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &gfx().pictures.picture,
        );
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(offset_of!(Picture, flags), 0x4);
        assert_eq!(offset_of!(Picture, buffer), 0x14);
        assert_eq!(offset_of!(Picture, list), 0x1c);
    }
}
