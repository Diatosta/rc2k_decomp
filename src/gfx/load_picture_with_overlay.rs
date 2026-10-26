//! `gfx::load_picture_with_overlay` (0x4118D1): a picture loaded, and a second laid
//! over it but for its magenta.
//!
//! In:  eax = the picture's path, ebx = the overlay's
//! Out: every register preserved; carry = the picture could not be read
//!
//! `gfx::load_picture` of the first; if it was read, memory of 0x1D4C00 + 0xC80 bytes from the
//! work list (`g_lexiconMemory`), the picture's size, row length and buffer kept while
//! `gfx::read_picture_file` reads the overlay into that memory, then put back; if the overlay
//! was read, each of its pixels that is not 0xFF00FF, row by row from the memory's start (rows
//! `g_decodedStride` apart, `g_decodedWidth` by `g_decodedHeight`), goes over the picture (rows
//! `g_pictureRowBytes` apart); the work list given back. Five callers, the slideshow's picture
//! among them. Refuses an overlay read with no width or height (the original counts through all
//! 32 bits), and what its parts refuse.

use crate::errors::{BadInput, Refused};
use crate::gfx::decoder::DECODER;
use crate::gfx::load_picture::load_picture;
use crate::gfx::picture_store::PICTURE;
use crate::gfx::read_picture_file::read_picture_file;
use crate::memory::allocate::allocate;
use crate::memory::free_list::free_list;
use crate::strings::compile_lexicon::LEXICON;

const MEMORY: u32 = 0x1D4C00 + 0xC80;

// ---- Rust version ----

/// Returns the carry: true if the picture could not be read.
pub unsafe fn load_picture_with_overlay(path: *mut u8, overlay: *mut u8) -> Result<bool, Refused> {
    unsafe {
        if !load_picture(path)? {
            return Ok(true);
        }
        let list = LEXICON.memory.as_ptr();
        let memory = allocate(list, MEMORY)?;
        let kept: [u32; 4] = [
            PICTURE.width.get(),
            PICTURE.height.get(),
            PICTURE.row_bytes.get(),
            PICTURE.buffer.get(),
        ];
        PICTURE.buffer.set(crate::pointers::game_address(memory));
        let read = read_picture_file(overlay);
        PICTURE.width.set(kept[0]);
        PICTURE.height.set(kept[1]);
        PICTURE.row_bytes.set(kept[2]);
        PICTURE.buffer.set(kept[3]);
        let read = match read {
            Ok(read) => read,
            Err(e) => {
                free_list(list)?;
                return Err(e);
            }
        };
        if read {
            // Converted: `rc2k_core::gfx::loading::lay_over`; both buffers 0x1D4C00 + 0xC80
            // bytes (the picture buffer's room, `gfx::allocate_picture_buffer`).
            let (width, height): (u32, u32) = (DECODER.width.get(), DECODER.height.get());
            let (stride, row): (u32, u32) = (DECODER.stride.get(), PICTURE.row_bytes.get());
            let from = std::slice::from_raw_parts(memory as *const u8, MEMORY as usize);
            let to =
                std::slice::from_raw_parts_mut(crate::pointers::host(kept[3]), MEMORY as usize);
            let laid = rc2k_core::gfx::loading::lay_over(
                to,
                from,
                width as usize,
                height as usize,
                stride as usize,
                row as usize,
            );
            if let Err(e) = laid {
                free_list(list)?;
                return Err(BadInput(e.0).into());
            }
        }
        free_list(list)?;
        Ok(false)
    }
}
