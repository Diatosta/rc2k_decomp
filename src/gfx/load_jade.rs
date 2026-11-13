//! Loads the glyphs of the fatal error's own font, `files\d\gfx\jade.bun`, and works out its
//! colour.
//! Up to 0x1400 bytes of the file go to `glyphs` (the glyph table of the four font objects); a file
//! that cannot be read is a fatal error, as is one that fills all 0x1400 bytes. Then `colour`
//! (those fonts' palette) = white in the surface's format (`gfx::components_to_pixel`). Refuses
//! what `read_file` refuses.

use crate::errors::{FatalError, Refused};
use crate::files::read_file::read_file_to_store;
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::pixel_format::surface_format;

const ROOM: u32 = rc2k_core::gfx::screenshot::JADE_ROOM;

pub fn load_jade(
    path: &[u8],
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let fonts = &gfx.fonts;
    use rc2k_core::gfx::screenshot::{JADE_COLOUR, jade_read};
    let read = read_file_to_store(path, &fonts.jade.glyphs, ROOM, 0)?;
    jade_read(read.ok, read.bytes_read).map_err(|code| FatalError { code, message: "" })?;
    fonts.jade.colour.set(components_to_pixel(
        &surface_format(system),
        JADE_COLOUR,
        gfx,
    ));
    Ok(())
}

/// The game's `files\d\gfx\jade.bun`.
pub fn load_jade_in_game(
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let path = &crate::gfx::texts::JADE;
    load_jade(&path[..path.len() - 1], gfx, system)
}
