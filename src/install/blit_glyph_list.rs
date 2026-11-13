//! `install::blit_glyph_list` (0x402628): a list of `count` records (16 bytes each: dx, dy,
//! glyph, palette) blitted, each at (dx + x, dy + y) with its palette (`gfx::blit_glyph`).
//! //!
//! Takes the count, table, x, y.
//! Caller the copy-progress frame (the install bar). Refuses a count of 0
//! (the original's loop is a do-while: it
//! would blit 2^32 records).

use crate::errors::Refused;
use crate::install::vars::GlyphRecord;
use rc2k_core::install::layout;

/// The records of `table` blitted at (x, y) on, from the glyph table `glyphs`. An empty table
/// is refused.
pub fn blit_glyph_list(
    table: &[GlyphRecord],
    x: u32,
    y: u32,
    glyphs: u32,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::install::layout` (`list_records`, `list_place`).
    for (record, _) in table.iter().zip(layout::list_records(table.len() as u32)?) {
        let (x, y) = layout::list_place(record.dx.get(), record.dy.get(), x, y);
        // `gfx::blit_glyph` with the glyph and the palette.
        crate::gfx::blit_glyph::blit_glyph_on_surface(
            record.glyph.get(),
            glyphs,
            x as i32,
            y as i32,
            Some(palette(record.palette_link())),
            gfx,
        )?;
    }
    Ok(())
}

/// The palette a record's link names: the blended palette, the only one the pages name.
fn palette(link: crate::install::page_links::PageLink) -> u32 {
    if !link.is_blended_palette() {
        crate::install::installer::refuse(crate::errors::BadInput(
            "a glyph list's palette that is not the blended palette",
        ));
    }
    crate::gfx::glyph_store::BLENDED_PLACE
}
