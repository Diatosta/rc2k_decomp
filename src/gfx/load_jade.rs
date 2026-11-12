//! `gfx::load_jade` (0x4075F3): loads the glyphs of the fatal error's own font,
//! `files\d\gfx\jade.bun`, and works out its colour.
//!
//! Up to 0x1400 bytes of the file to `glyphs` (the glyph table of the four font
//! objects at 0x51DE30 on); a file that cannot be read is fatal error 0x80F0B0D1, one
//! that fills all 0x1400 bytes 0x80F0B0D2. Then `colour` (those fonts' palette) =
//! white in the surface's format (0x407629: `gfx::components_to_pixel` of 0xFFFFFF).
//! One caller, `system::main_loop`.
//! Refuses what `read_file` refuses.

use crate::errors::{FatalError, Refused};
use crate::files::read_file::read_file_to_store;
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::pixel_format::surface_format;

// ---- game data ----

const ROOM: u32 = rc2k_core::gfx::screenshot::JADE_ROOM;

pub fn load_jade(path: &[u8], gfx: &crate::gfx::owner::Gfx) -> Result<(), Refused> {
    let fonts = &gfx.fonts;
    // Converted: `rc2k_core::gfx::screenshot::jade_read`.
    use rc2k_core::gfx::screenshot::{JADE_COLOUR, jade_read};
    let read = read_file_to_store(path, &fonts.jade.glyphs, ROOM, 0)?;
    jade_read(read.ok, read.bytes_read).map_err(|code| FatalError { code, message: "" })?;
    fonts
        .jade
        .colour
        .set(components_to_pixel(&surface_format()?, JADE_COLOUR, gfx));
    Ok(())
}

/// The game's `files\d\gfx\jade.bun`.
pub fn load_jade_in_game(gfx: &crate::gfx::owner::Gfx) -> Result<(), Refused> {
    let path = &crate::gfx::texts::JADE;
    load_jade(&path[..path.len() - 1], gfx)
}
