//! Loads a picture and shows it at once: the loading picture.
//!
//! The window's messages are passed on; `showing` = 1; the picture's size is set to 800 x 600 with
//! row length 0xFFFFFFFD and the file is read (`gfx::read_picture_file`); the size then becomes
//! the screen's, the row length the screen's width * 4.
//! - Not read: `gfx::clear_picture`, and on as for a picture of the screen's width.
//! - Read: `showing` is taken (set to 0); if it was not 0 and the fonts are built, the picture's
//!   flags become 0x802, the picture is made the surface (`gfx::picture_as_surface`) and string
//!   0xC0 is drawn on it at (790, 47) with the loading font.
//! - A picture of the screen's width: `gfx::convert_picture`, then
//!   `gfx::copy_picture_to_surface` and `gfx::show_frame`.
//! - Any other width: `gfx::lock_surface` (failing ends it), `gfx::rescale_to_surface`,
//!   `gfx::unlock_surface`, `gfx::show_frame`.
//!
//! The picture is shown once: the screen is one picture, with no second page behind it to
//! fill.
//!
//! Callers: `menu::show_loading_picture`, `race::stage_loading`.

use crate::errors::Refused;
use crate::gfx::copy_picture_to_surface::{GameSurface, copy_picture_to_surface};
use crate::platform::{lock_surface, pump_messages_now, show_frame_now, unlock_surface};
use rc2k_core::gfx::picture_size as sizes;

const LOADING_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::LOADING;

/// Shows the picture at `path`.
pub fn show_picture_file(
    o: crate::owners::Owners<'static>,
    path: &dyn crate::memory::session::TextPlace,
) -> Result<(), Refused> {
    let (gfx, system) = (o.gfx, o.system);
    pump_messages_now(o);
    gfx.pictures.showing.set(1);
    crate::gfx::picture::set_size(&gfx.pictures, sizes::READ);
    // A refused picture is not read, as `read_picture_file`'s callers see it.
    crate::race::load_times::mark("picture: read");
    let read = match crate::gfx::read_picture_file::read_picture_file(path, gfx, system) {
        Ok(read) => read,
        Err(Refused::BadInput(_)) => false,
        Err(e) => return Err(e),
    };
    let (width, height): (u32, u32) = (
        system.session.screen.get().width.get(),
        system.session.screen.get().height.get(),
    );
    crate::gfx::picture::set_size(&gfx.pictures, sizes::screen(width, height));
    crate::race::load_times::mark("picture: set up");
    if !read {
        crate::gfx::clear_picture::clear_picture(&gfx.pictures)?;
    } else {
        let was_showing = gfx.pictures.showing.get();
        gfx.pictures.showing.set(0);
        if was_showing != 0 && gfx.fonts.font_block.palettes.is_set() {
            gfx.pictures.picture.flags.set(0x802u32);
            crate::gfx::picture_as_surface::picture_as_surface(gfx)?;
            crate::text::draw_string_by_id::draw_string_by_id_at(
                crate::strings::append_string_by_id::StringOwners::of(o),
                0xC0,
                LOADING_FONT,
                0x316,
                0x2F,
                &mut crate::text::draw_text::GameCanvas::new(gfx),
            )?;
        }
        if sizes::rescaled(gfx.pictures.decoder.width.get(), width) {
            crate::race::load_times::mark("picture: rescale");
            // Not locked.
            if lock_surface(gfx, system) < 1 {
                return Ok(());
            }
            crate::gfx::rescale_to_surface::rescale_to_surface(
                &gfx.rescale,
                &gfx.drawing.draw_surface,
                gfx,
                system,
                &mut None,
            )?;
            unlock_surface(gfx);
            crate::race::load_times::mark("picture: show frame");
            show_frame_now(o);
            return Ok(());
        }
    }
    crate::race::load_times::mark("picture: convert");
    crate::gfx::convert_picture::convert_picture_now_in(gfx)?;
    crate::race::load_times::mark("picture: copy");
    copy_picture_to_surface(&mut GameSurface(gfx, system))?;
    crate::race::load_times::mark("picture: show frame");
    show_frame_now(o);
    Ok(())
}
