//! The picture buffer: where a picture file is decoded to, as 32-bit pixels of four 8-bit
//! components, before it is converted to the surface's format and drawn. The background of
//! every menu screen goes through it. Its variables are `gfx::picture_store::PICTURE`.

use crate::gfx::picture_store::PICTURE;

/// The buffer holds a whole screen: 800 x 600 pixels of four bytes.
pub const PICTURE_DWORDS: usize = 0x75300;

pub fn picture() -> *mut u8 {
    crate::pointers::host(PICTURE.buffer.get())
}

/// The picture buffer as a game pointer (null: `addr()` is 0).
pub fn picture_ptr() -> crate::pointers::GamePtr {
    crate::pointers::GamePtr::new(PICTURE.buffer.get())
}

/// The picture's width, height and row length set (`rc2k_core::gfx::picture_size`).
pub fn set_size(size: rc2k_core::gfx::picture_size::PictureSize) {
    let [width, height, row_bytes] = size.dwords();
    PICTURE.width.set(width);
    PICTURE.height.set(height);
    PICTURE.row_bytes.set(row_bytes);
}
