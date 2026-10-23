//! `gfx::show_picture_file` (0x41172B): loads a picture and shows it at once, on every
//! page of the screen: the loading picture.
//!
//! In:  eax = the picture's path
//! Out: eax = what `dword_51B9C4` held (1, unless the file could not be read: then the
//!      path); the rest as its callees leave them
//! The window's messages are passed on (`sub_410535`); `dword_51B9C4` = 1; the picture's
//! size is set to 800 x 600 with row length 0xFFFFFFFD and the file is read
//! (`gfx::read_picture_file`); the size then becomes the screen's, the row length the
//! screen's width * 4.
//! - Not read: `gfx::clear_picture`, and on as for a picture of the screen's width.
//! - Read: `dword_51B9C4` is taken (set to 0); if it was not 0 and the fonts are built
//!   (`g_fontPalettes`), the picture's flags become 0x802, the picture is made the
//!   surface (`gfx::picture_as_surface`) and string 0xC0 is drawn on it at (790, 47) with
//!   the font at 0x51EAA4.
//! - A picture of the screen's width: `gfx::convert_picture`, then twice
//!   `gfx::copy_picture_to_surface` and `gfx::show_frame`, and a third time if the
//!   screen mode has bit 0x40000 (three pages).
//! - Any other width: as many times, `gfx::lock_surface` (failing ends it),
//!   `gfx::rescale_to_surface`, `gfx::unlock_surface`, `gfx::show_frame`.
//!
//! Callers: `menu::show_loading_picture`, `race::stage_loading`. Every callee is called
//! directly (2026-10-23; before, through the game's address with the registers handed
//! on); of the registers only eax is kept for the callers (see Out).

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_ptr, game_read, game_write, rva};
use crate::gfx::copy_picture_to_surface::{GameSurface, copy_picture_to_surface};
use crate::platform::{lock_surface, pump_messages_now, show_frame_now, unlock_surface};
use rc2k_core::gfx::picture_size as sizes;

// ---- game data ----

const SHOWING_RVA: usize = rva(0x51B9C4); // dword_51B9C4
const PICTURE_FLAGS_RVA: usize = rva(0x4F49CC); // g_pictureFlags
const PICTURE_WIDTH_RVA: usize = rva(0x4F49D0); // g_pictureWidth, height, row bytes
const SCREEN_WIDTH_RVA: usize = rva(0x4F494C); // g_screenWidth
const SCREEN_HEIGHT_RVA: usize = rva(0x4F4950); // g_screenHeight
const SCREEN_MODE_FLAGS_RVA: usize = rva(0x4F4934); // g_screenModeFlags
const DECODED_WIDTH_RVA: usize = rva(0x519124); // g_decodedWidth
const FONT_PALETTES_RVA: usize = rva(0x51DEB0); // g_fontPalettes
const LOADING_FONT_RVA: usize = rva(0x51EAA4); // stru_51EAA4

// ---- Rust version ----

/// The Rust version on the picture at `path`; returns the eax it leaves (see the header).
pub unsafe fn show_picture_file(path: *mut u8) -> Result<u32, Refused> {
    unsafe {
        pump_messages_now();
        game_write(SHOWING_RVA, 1u32);
        // Converted: `rc2k_core::gfx::picture_size` (`READ`, `screen`, `pages`,
        // `rescaled`).
        game_write(PICTURE_WIDTH_RVA, sizes::READ.dwords());
        // A refused picture is not read, as `read_picture_file`'s callers by address see it.
        let read = match crate::gfx::read_picture_file::read_picture_file(path) {
            Ok(read) => read,
            Err(Refused::BadInput(_)) => false,
            Err(e) => return Err(e),
        };
        let (width, height): (u32, u32) =
            (game_read(SCREEN_WIDTH_RVA), game_read(SCREEN_HEIGHT_RVA));
        game_write(PICTURE_WIDTH_RVA, sizes::screen(width, height).dwords());
        let pages = sizes::pages(game_read::<u32>(SCREEN_MODE_FLAGS_RVA));
        let mut eax = path as u32;
        if !read {
            crate::gfx::clear_picture::clear_picture()?;
        } else {
            eax = game_read(SHOWING_RVA);
            game_write(SHOWING_RVA, 0u32);
            if eax != 0 && game_read::<u32>(FONT_PALETTES_RVA) != 0 {
                game_write(PICTURE_FLAGS_RVA, 0x802u32);
                crate::gfx::picture_as_surface::picture_as_surface()?;
                crate::text::draw_string_by_id::draw_string_by_id_at(
                    0xC0,
                    game_ptr(LOADING_FONT_RVA),
                    0x316,
                    0x2F,
                    &mut crate::text::draw_text::GameCanvas,
                )?;
            }
            if sizes::rescaled(game_read::<u32>(DECODED_WIDTH_RVA), width) {
                for _ in 0..pages {
                    // `cmp g_surfaceLocked, 1`'s carry: not locked.
                    if lock_surface() < 1 {
                        return Ok(eax);
                    }
                    crate::gfx::rescale_to_surface::rescale_to_surface(game_addr(
                        crate::gfx::rescale_to_surface::SURFACE_RVA,
                    )
                        as *const u32)?;
                    unlock_surface();
                    show_frame_now();
                }
                return Ok(eax);
            }
        }
        crate::gfx::convert_picture::convert_picture_now()?;
        for _ in 0..pages {
            copy_picture_to_surface(&mut GameSurface)?;
            show_frame_now();
        }
        Ok(eax)
    }
}

/// The Rust version where Rust callers run it (a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), Refused> {
    r.eax = unsafe { show_picture_file(r.eax as *mut u8) }?;
    Ok(())
}
