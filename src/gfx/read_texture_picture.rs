//! `gfx::read_texture_picture` (0x405356): a texture's picture read into the picture
//! buffer.
//!
//! In:  eax = the path (may be written to, as `gfx::read_picture_file` may), or 0
//! Out: read: ebx = the decoded pixels (`g_decodedPixels`), ecx, edx = their width
//!      and height (`g_decodedWidth`, `g_decodedHeight`), carry clear; not read: ebx =
//!      the picture buffer, ecx, edx = the picture's size, carry set, or clear for no
//!      path; everything else preserved
//!
//! The picture's size 256 x 256 with rows that follow one another (row length 0xFFFFFFFD), its
//! first 0x10000 dwords filled with -1, then `read_picture_file` if there is a path. Five call
//! sites: `gfx::load_picture_to_texture` (twice) and `scene::upload_car_icon` (twice), the
//! second of each pair for the picture kept for the alpha. Refuses no picture buffer, and what
//! `read_picture_file` refuses.

use crate::errors::{BadInput, Refused};
use crate::gfx::decoder::DECODER;
use crate::gfx::picture::picture;
use crate::gfx::picture_store::PICTURE;
use crate::gfx::read_picture_file::read_picture_file;
use crate::pointers::GamePtr;
use rc2k_core::gfx::picture_size as sizes;

// ---- Rust version ----

/// The registers it leaves: the pixels, their width and height, and whether the
/// reading failed.
pub unsafe fn read_texture_picture(path: *mut u8) -> Result<(u32, u32, u32, bool), Refused> {
    unsafe {
        let buffer = picture();
        if buffer.is_null() {
            return Err(BadInput("no picture buffer").into());
        }
        let place = GamePtr::new(crate::pointers::game_address(buffer));
        // Converted: `rc2k_core::gfx::picture_size` (`TEXTURE`, `TEXTURE_FILLED`).
        crate::gfx::picture::set_size(sizes::TEXTURE);
        place.fill(0xFF, sizes::TEXTURE_FILLED * 4, "the picture buffer")?;
        let failed = if path.is_null() {
            false
        } else if read_picture_file(path)? {
            return Ok((
                DECODER.pixels.get(),
                DECODER.width.get(),
                DECODER.height.get(),
                false,
            ));
        } else {
            true
        };
        Ok((
            crate::pointers::game_address(buffer),
            PICTURE.width.get(),
            PICTURE.height.get(),
            failed,
        ))
    }
}
