//! `gfx::load_picture` (0x411863): loads a picture file into the picture buffer at the
//! screen's size.
//!
//! In:  eax = path, which may be written to (see `files::swap_missing_extension`)
//! Out: carry = the file could not be read; every register preserved
//! Sets the picture's size to 800 x 600 with rows that follow one another (row length
//! 0xFFFFFFFD), clears the buffer and calls `read_picture_file`. Whatever that says, the
//! picture's size then becomes the screen's (`dword_4F494C` x `dword_4F4950`) and its row
//! length four bytes a pixel. If the file was read and its decoded width is not the
//! screen's, `rescale_picture` makes it so.
//! Ten callers, `menu::load_background` and `service::load_background` among them.
//! Shadow mode runs the original alone: everything it calls is compared in its own hook.
//! Refuses what the functions it calls refuse. The name is a reading.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};
use crate::gfx::clear_picture::clear_picture;
use crate::gfx::pcx::DECODED_WIDTH_RVA;
use crate::gfx::picture::picture;
use crate::gfx::picture::set_size;
use crate::gfx::read_picture_file::read_picture_file;
use crate::gfx::rescale_picture::rescale_picture;
use rc2k_core::gfx::picture_size as sizes;

// ---- game data ----

pub const SCREEN_WIDTH_RVA: usize = rva(0x4F494C); // dword_4F494C
pub const SCREEN_HEIGHT_RVA: usize = rva(0x4F4950); // dword_4F4950

// ---- Rust version ----

/// Returns whether the file was read.
pub unsafe fn load_picture(path: *mut u8) -> Result<bool, Refused> {
    unsafe {
        // Refused before anything is written, if there is no buffer.
        if picture().is_null() {
            clear_picture()?;
        }
        // Converted: `rc2k_core::gfx::picture_size` (`READ`, `screen`, `rescaled`).
        set_size(sizes::READ);
        clear_picture()?;
        let read = read_picture_file(path);
        let screen_width: u32 = game_read(SCREEN_WIDTH_RVA);
        set_size(sizes::screen(
            screen_width,
            game_read::<u32>(SCREEN_HEIGHT_RVA),
        ));
        if !read? {
            return Ok(false);
        }
        if sizes::rescaled(game_read::<u32>(DECODED_WIDTH_RVA), screen_width) {
            rescale_picture()?;
        }
        Ok(true)
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let read = unsafe { load_picture(r.eax as *mut u8) }?;
        r.set_carry(!read)
    }
    Ok(())
}

// ---- for this self-test and `menu::load_background`'s: everything a picture's loading
// writes, saved and put back, and files to load ----
