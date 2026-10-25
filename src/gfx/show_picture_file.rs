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
//! Callers: `menu::show_loading_picture`, `race::stage_loading`.

use crate::errors::Refused;
use crate::gfx::copy_picture_to_surface::{GameSurface, copy_picture_to_surface};
use crate::gfx::decoder::DECODER;
use crate::gfx::fonts_store::FONTS;
use crate::gfx::picture_store::PICTURE;
use crate::gfx::screen_state::SCREEN_MODE;
use crate::platform::{lock_surface, pump_messages_now, show_frame_now, unlock_surface};
use crate::session::Dword;
use crate::session::SCREEN;
use rc2k_core::gfx::picture_size as sizes;

// ---- game data ----

/// IDA's address of the showing switch.
pub const SHOWING_START: usize = 0x51B9C4;

/// `g_showingPicture` (`dword_51B9C4`): 1 while a picture is being shown.
pub static SHOWING: Dword = Dword::new(0);

/// Where the switch is (`moved::MOVED`'s home for IDA 0x51B9C4).
pub fn showing_home() -> usize {
    SHOWING.addr()
}
const LOADING_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::LOADING;

// ---- Rust version ----

/// The Rust version on the picture at `path`; returns the eax it leaves (see the header).
pub unsafe fn show_picture_file(path: *mut u8) -> Result<u32, Refused> {
    unsafe {
        pump_messages_now();
        SHOWING.set(1);
        // Converted: `rc2k_core::gfx::picture_size` (`READ`, `screen`, `pages`,
        // `rescaled`).
        crate::gfx::picture::set_size(sizes::READ);
        // A refused picture is not read, as `read_picture_file`'s callers by address see it.
        let read = match crate::gfx::read_picture_file::read_picture_file(path) {
            Ok(read) => read,
            Err(Refused::BadInput(_)) => false,
            Err(e) => return Err(e),
        };
        let (width, height): (u32, u32) = (SCREEN.width.get(), SCREEN.height.get());
        crate::gfx::picture::set_size(sizes::screen(width, height));
        let pages = sizes::pages(SCREEN_MODE.mode_flags.get());
        let mut eax = path as u32;
        if !read {
            crate::gfx::clear_picture::clear_picture()?;
        } else {
            eax = SHOWING.get();
            SHOWING.set(0);
            if eax != 0 && FONTS.palettes.get() != 0 {
                PICTURE.flags.set(0x802u32);
                crate::gfx::picture_as_surface::picture_as_surface()?;
                crate::text::draw_string_by_id::draw_string_by_id_at(
                    0xC0,
                    LOADING_FONT.as_ptr(),
                    0x316,
                    0x2F,
                    &mut crate::text::draw_text::GameCanvas,
                )?;
            }
            if sizes::rescaled(DECODER.width.get(), width) {
                for _ in 0..pages {
                    // `cmp g_surfaceLocked, 1`'s carry: not locked.
                    if lock_surface() < 1 {
                        return Ok(eax);
                    }
                    crate::gfx::rescale_to_surface::rescale_to_surface(
                        crate::gfx::screen_state::DRAW_SURFACE.x.addr() as *const u32,
                    )?;
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

#[cfg(test)]
mod block_tests {
    use super::*;

    #[test]
    fn first_value_is_the_games() {
        crate::gfx::first_values_are_the_games(&SHOWING, SHOWING_START, SHOWING_START + 4);
        assert_eq!(crate::moved::place(SHOWING_START), SHOWING.addr());
    }
}
