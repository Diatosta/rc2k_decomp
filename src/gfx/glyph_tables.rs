//! The glyph tables' work: `begin_glyph_tables`, `begin_glyph_table`, `end_glyph_tables`,
//! `palette_to_pixels` and `alpha_palette`.

use crate::errors::{BadInput, Refused};
use crate::gfx::build_glyphs::SheetVars;
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::glyph_builder::DIRECT_COLOUR;
use crate::memory::MemoryList;
use crate::memory::free_list::free_list;
use crate::memory::take_from_list::take_from_list;
use crate::session::PlainDword;

/// The fonts' and the sprites' tables.
#[derive(rc2k_derive::Cells)]
pub struct GlyphTables {
    /// The tables' size in units of 0x400 bytes times `Screen.bytes` (`bundleheapsize`, a
    /// `var\ral.cfg` setting).
    pub units: PlainDword,
    /// `GlyphTables.memory.head`: the list the fonts' and the sprites' tables lie on (16 bytes).
    pub memory: crate::memory::ListCells,
}

impl GlyphTables {
    pub const fn new() -> Self {
        GlyphTables {
            units: PlainDword::new(0x15E),
            memory: crate::memory::ListCells::new(MemoryList {
                head: 0,
                total: 0,
                cursor: 0,
                remaining: 0,
            }),
        }
    }
}

/// The work memory taken, the picture's size set for sheets, the pixel layout the surface's, and no
/// table yet.
pub fn begin_glyph_tables(
    vars: &SheetVars,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
    strings: &crate::strings::owner::Strings,
) -> Result<(), Refused> {
    let pictures = &gfx.pictures;
    pictures.picture.width.set(800u32);
    pictures.picture.height.set(600u32);
    pictures.picture.row_bytes.set(0xFFFF_FFFDu32);
    pictures
        .decoder
        .format
        .set(crate::gfx::pixel_format::SCREEN_FORMAT);
    let bytes = strings
        .lexicon
        .memory
        .kilobytes
        .get()
        .wrapping_mul(0x400)
        .wrapping_mul(system.session.screen.get().bytes.get());
    // (A failed allocation cannot come: the allocator reports its own error.) Only counted: the
    // work memory holds nothing Rust reads back (`build_glyphs` keeps the sheet's table in its own
    // bytes).
    crate::memory::owned::count_on(&strings.lexicon.memory.memory, bytes)?;
    vars.table.set(0u32);
    vars.table_size.set(0u32);
    Ok(())
}

/// A new table starts where the list stands, after the padding that brings the table before it to
/// whole dwords.
pub fn begin_glyph_table(
    fonts: &crate::gfx::owner::FontStores,
    vars: &SheetVars,
    list: &crate::memory::ListCells,
) -> Result<(), BadInput> {
    let padding = vars.table_size.get().wrapping_neg() & 3;
    take_from_list(list, padding)?;
    take_from_list(list, 0)?;
    vars.table
        .set(crate::gfx::glyph_store::new_table(fonts, list)?);
    vars.table_size.set(0u32);
    Ok(())
}

pub fn end_glyph_tables(strings: &crate::strings::owner::Strings) -> Result<(), BadInput> {
    free_list(&strings.lexicon.memory.memory)
}

/// The last sheet's palette as 256 pixels of the builder's layout at `to`; nothing if `to` is 0 or
/// the last sheet was one of components.
pub fn palette_to_pixels(
    to: u32,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), BadInput> {
    let pictures = &gfx.pictures;
    if to == 0 || pictures.decoder.builder_flags.get() & DIRECT_COLOUR != 0 {
        return Ok(());
    }
    let format = pictures.decoder.format.get();
    if format == 0 {
        return Err(BadInput("no pixel layout for the palette"));
    }
    let format = crate::gfx::pixel_format::layout(system, gfx, format)?;
    let mut pixels = Vec::with_capacity(256 * 4);
    for item in pictures.decoder.palette.iter().take(256) {
        let components: u32 = item.get();
        pixels.extend_from_slice(&components_to_pixel(&format, components, gfx).to_le_bytes());
    }
    crate::gfx::glyph_store::write_bytes(&gfx.fonts, to, 0, &pixels, "a palette's pixels")
}

/// The last sheet's palette for pixels that are a colour in the low four bits and an alpha in the
/// top four: entry 0 becomes 0, every entry the colour of its low four bits' entry, and its fourth
/// byte the alpha, the four bits twice.
pub fn alpha_palette(gfx: &crate::gfx::owner::Gfx) {
    let palette = &gfx.pictures.decoder.palette;
    palette[0].set(0);
    for entry in 0..256usize {
        let colour = palette[entry & 0x0F].get();
        let alpha = (entry & 0xF0 | entry >> 4) as u32;
        palette[entry].set(colour & 0x00FF_FFFF | alpha << 24);
    }
}
