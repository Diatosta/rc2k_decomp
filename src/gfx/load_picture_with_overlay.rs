//! `gfx::load_picture_with_overlay` (0x4118D1): a picture loaded, and a second laid
//! over it but for its magenta.
//!
//! In:  eax = the picture's path, ebx = the overlay's
//! Out: every register preserved; carry = the picture could not be read
//!
//! `gfx::load_picture` of the first; if it was read, memory of 0x1D4C00 + 0xC80 bytes
//! from the work list (`g_lexiconMemory`), the picture's size, row length and buffer
//! kept while `gfx::read_picture_file` reads the overlay into that memory, then put
//! back; if the overlay was read, each of its pixels that is not 0xFF00FF, row by row
//! from the memory's start (rows `g_decodedStride` apart, `g_decodedWidth` by
//! `g_decodedHeight`), goes over the picture (rows `g_pictureRowBytes` apart); the work
//! list given back. Five callers, the slideshow's picture among them.
//! Shadow mode runs the original alone.
//! Refuses an overlay read with no width or height (the original counts through all 32
//! bits), and what its parts refuse.
//! Self-test: `gfx::load_picture`'s files, each over each.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write};
use crate::gfx::build_glyphs::WORK_MEMORY_RVA;
use crate::gfx::load_picture::load_picture;
use crate::gfx::pcx::{DECODED_HEIGHT_RVA, DECODED_WIDTH_RVA, STRIDE_RVA};
use crate::gfx::picture::{HEIGHT_RVA, PICTURE_RVA, ROW_BYTES_RVA, WIDTH_RVA};
use crate::gfx::read_picture_file::read_picture_file;
use crate::memory::MemoryList;
use crate::memory::allocate::allocate;
use crate::memory::free_list::free_list;

const MEMORY: u32 = 0x1D4C00 + 0xC80;

// ---- Rust version ----

/// Returns the carry: true if the picture could not be read.
pub unsafe fn load_picture_with_overlay(path: *mut u8, overlay: *mut u8) -> Result<bool, Refused> {
    unsafe {
        if !load_picture(path)? {
            return Ok(true);
        }
        let list = game_addr(WORK_MEMORY_RVA) as *mut MemoryList;
        let memory = allocate(list, MEMORY)?;
        let kept: [u32; 4] = [
            game_read(WIDTH_RVA),
            game_read(HEIGHT_RVA),
            game_read(ROW_BYTES_RVA),
            game_read(PICTURE_RVA),
        ];
        game_write(PICTURE_RVA, memory as u32);
        let read = read_picture_file(overlay);
        game_write(WIDTH_RVA, kept[0]);
        game_write(HEIGHT_RVA, kept[1]);
        game_write(ROW_BYTES_RVA, kept[2]);
        game_write(PICTURE_RVA, kept[3]);
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
            let (width, height): (u32, u32) =
                (game_read(DECODED_WIDTH_RVA), game_read(DECODED_HEIGHT_RVA));
            let (stride, row): (u32, u32) = (game_read(STRIDE_RVA), game_read(ROW_BYTES_RVA));
            let from = std::slice::from_raw_parts(memory as *const u8, MEMORY as usize);
            let to = std::slice::from_raw_parts_mut(kept[3] as *mut u8, MEMORY as usize);
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

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let carry = unsafe { load_picture_with_overlay(r.eax as *mut u8, r.ebx as *mut u8) }?;
        r.set_carry(carry)
    }
    Ok(())
}
