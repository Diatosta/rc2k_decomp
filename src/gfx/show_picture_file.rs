//! `gfx::show_picture_file` (0x41172B): loads a picture and shows it at once, on every
//! page of the screen: the loading picture.
//!
//! Takes: the picture's path
//! The window's messages are passed on (0x410535); `showing` = 1; the picture's
//! size is set to 800 x 600 with row length 0xFFFFFFFD and the file is read
//! (`gfx::read_picture_file`); the size then becomes the screen's, the row length the
//! screen's width * 4.
//! - Not read: `gfx::clear_picture`, and on as for a picture of the screen's width.
//! - Read: `showing` is taken (set to 0); if it was not 0 and the fonts are built
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
#[cfg(test)]
use crate::gfx::owner::gfx;
use crate::platform::{lock_surface, pump_messages_now, show_frame_now, unlock_surface};
use rc2k_core::gfx::picture_size as sizes;

// ---- game data ----

/// IDA's address of the showing switch.
pub const SHOWING_START: usize = 0x51B9C4;

const LOADING_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::LOADING;

/// Shows the picture at `path`.
pub fn show_picture_file(
    path: &dyn crate::session::TextPlace,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    pump_messages_now();
    gfx.pictures.showing.set(1);
    // Converted: `rc2k_core::gfx::picture_size` (`READ`, `screen`, `pages`,
    // `rescaled`).
    crate::gfx::picture::set_size(sizes::READ);
    // A refused picture is not read, as `read_picture_file`'s callers by address see it.
    let read = match crate::gfx::read_picture_file::read_picture_file(path, gfx) {
        Ok(read) => read,
        Err(Refused::BadInput(_)) => false,
        Err(e) => return Err(e),
    };
    let (width, height): (u32, u32) = (
        system.session.screen.width.get(),
        system.session.screen.height.get(),
    );
    crate::gfx::picture::set_size(sizes::screen(width, height));
    let pages = sizes::pages(gfx.display.mode.mode_flags.get());
    if !read {
        crate::gfx::clear_picture::clear_picture()?;
    } else {
        let was_showing = gfx.pictures.showing.get();
        gfx.pictures.showing.set(0);
        if was_showing != 0 && gfx.fonts.font_block.palettes.is_set() {
            gfx.pictures.picture.flags.set(0x802u32);
            crate::gfx::picture_as_surface::picture_as_surface(gfx)?;
            crate::text::draw_string_by_id::draw_string_by_id_at(
                0xC0,
                LOADING_FONT,
                0x316,
                0x2F,
                &mut crate::text::draw_text::GameCanvas,
            )?;
        }
        if sizes::rescaled(gfx.pictures.decoder.width.get(), width) {
            for _ in 0..pages {
                // `cmp g_surfaceLocked, 1`'s carry: not locked.
                if lock_surface() < 1 {
                    return Ok(());
                }
                crate::gfx::rescale_to_surface::rescale_to_surface(
                    &gfx.rescale,
                    crate::pointers::GamePtr::to(&gfx.drawing.draw_surface.x).limited(0x30),
                    gfx,
                )?;
                unlock_surface();
                show_frame_now();
            }
            return Ok(());
        }
    }
    crate::gfx::convert_picture::convert_picture_now()?;
    for _ in 0..pages {
        copy_picture_to_surface(&mut GameSurface)?;
        show_frame_now();
    }
    Ok(())
}

#[cfg(test)]
mod block_tests {
    use super::*;

    #[test]
    fn first_value_is_the_games() {
        let pictures = &gfx().pictures;
        crate::gfx::first_values_are_the_games(&pictures.showing, SHOWING_START, SHOWING_START + 4);
        assert_eq!(
            crate::moved::place(SHOWING_START),
            pictures.showing.addr() as usize
        );
    }
}
