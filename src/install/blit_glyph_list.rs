//! `install::blit_glyph_list` (0x402628): a list of `eax` records (16 bytes each: dx, dy,
//! glyph, palette) blitted, each at (dx + x, dy + y) with its palette (`gfx::blit_glyph`).
//! The x and y come in ecx and edx.
//!
//! In:  eax the count, ebx the table, ecx x, edx y
//! Out: every register kept (`pusha` / `popa`)
//! Caller the copy-progress frame (the install bar). Refuses a count of 0
//! (the original's loop is a do-while: it
//! would blit 2^32 records).
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::rd;
use rc2k_core::install::layout;

/// `count` records of `table` blitted at (x, y) on, from the glyph table `glyphs` (esi).
pub fn blit_glyph_list(
    count: u32,
    table: impl Into<crate::install::place::Place>,
    x: u32,
    y: u32,
    glyphs: u32,
) -> Result<(), Refused> {
    let table: u32 = table.into().game();
    // Converted: `rc2k_core::install::layout` (`list_records`, `list_place`).
    for (offset, _) in layout::list_records(count)? {
        let record = table.wrapping_add(offset);
        let (x, y) = layout::list_place(rd(record), rd(record + 4), x, y);
        // gfx::blit_glyph (0x4069B2): eax the glyph, ebx the palette.
        {
            crate::gfx::blit_glyph::blit_glyph_on_surface(
                rd(record + 8),
                glyphs,
                x as i32,
                y as i32,
                // (The bar's records name the blended palette's home: `moved::point_into_homes`.)
                rd(record + 0xC),
            )
        }?;
    }
    Ok(())
}
