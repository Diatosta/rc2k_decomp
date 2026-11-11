//! `install::blit_glyph_list` (0x402628): a list of `count` records (16 bytes each: dx, dy,
//! glyph, palette) blitted, each at (dx + x, dy + y) with its palette (`gfx::blit_glyph`).
//! //!
//! Takes the count, table, x, y.
//! Caller the copy-progress frame (the install bar). Refuses a count of 0
//! (the original's loop is a do-while: it
//! would blit 2^32 records).

use crate::errors::Refused;
use crate::install::installer::{rd, rl};
use rc2k_core::install::layout;

/// `count` records of `table` blitted at (x, y) on, from the glyph table `glyphs`.
pub fn blit_glyph_list(
    count: u32,
    table: impl Into<crate::place::BytePlace>,
    x: u32,
    y: u32,
    glyphs: u32,
) -> Result<(), Refused> {
    let table: crate::place::BytePlace = table.into();
    // Converted: `rc2k_core::install::layout` (`list_records`, `list_place`).
    for (offset, _) in layout::list_records(count)? {
        let record = table.add(offset);
        let (x, y) = layout::list_place(rd(record), rd(record.add(4)), x, y);
        // `gfx::blit_glyph` with the glyph and the palette.
        {
            crate::gfx::blit_glyph::blit_glyph_on_surface(
                rd(record.add(8)),
                glyphs,
                x as i32,
                y as i32,
                Some(palette(rl(record.add(0xC)))),
            )
        }?;
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
    crate::pointers::GamePtr::to(crate::text::blend_font::BLENDED_PALETTE.get()).addr()
}
