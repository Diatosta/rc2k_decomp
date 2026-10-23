//! The picture buffer: where a picture file is decoded to, as 32-bit pixels of four 8-bit
//! components, before it is converted to the surface's format and drawn. The background of
//! every menu screen goes through it.

use crate::framework::{game_read, game_write, rva};

/// dword_4F49DC: the pointer to the buffer.
pub const PICTURE_RVA: usize = rva(0x4F49DC);
/// dword_4F49D0, dword_4F49D4: the picture's width and height in pixels.
pub const WIDTH_RVA: usize = rva(0x4F49D0);
pub const HEIGHT_RVA: usize = rva(0x4F49D4);
/// dword_4F49D8: the bytes of one row (width * 4, halved when the picture is converted to
/// 16-bit pixels); 0xFFFFFFFD while a picture is being loaded.
pub const ROW_BYTES_RVA: usize = rva(0x4F49D8);
/// dword_4F49CC: flags of the picture; bits 8 .. 11 say what its pixels are (0x200 after a
/// conversion to 16-bit pixels).
pub const FLAGS_RVA: usize = rva(0x4F49CC);
/// dword_4F4954: the bits per pixel of the screen mode.
pub const SCREEN_BITS_RVA: usize = rva(0x4F4954);

/// The buffer holds a whole screen: 800 x 600 pixels of four bytes.
pub const PICTURE_DWORDS: usize = 0x75300;

pub fn picture() -> *mut u8 {
    unsafe { game_read::<u32>(PICTURE_RVA) as *mut u8 }
}

/// The picture's width, height and row length set (`rc2k_core::gfx::picture_size`).
pub unsafe fn set_size(size: rc2k_core::gfx::picture_size::PictureSize) {
    unsafe { game_write(WIDTH_RVA, size.dwords()) };
}
