//! The picture loader's variables (the name, flags, size, rows and buffer of the picture, the list
//! its memory is on), out of RAL.EXE's image: IDA 0x4F49C8 .. 0x4F4A24. The texture grades that
//! follow in the image are `gfx::texture_pixels::GRADING`.

use crate::memory::MemoryList;
use crate::session::PlainDword;

/// The picture buffer: where a picture file is decoded to, as 32-bit pixels of four 8-bit
/// components, before it is converted to the surface's format and drawn.
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
        }
    }
}
