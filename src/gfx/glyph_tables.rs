//! `begin_glyph_tables` is `sub_412014` (0x412014), `begin_glyph_table` `sub_411FEC`
//! (0x411FEC), `end_glyph_tables` `sub_412077` (0x412077), `palette_to_pixels`
//! `sub_411FC8` (0x411FC8) with `sub_405E02` under it, `alpha_palette` `sub_412528`
//! (0x412528).

use crate::errors::{BadInput, Refused};
use crate::gfx::build_glyphs::SHEET;
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::decoder::DECODER;
use crate::gfx::glyph_builder::DIRECT_COLOUR;
use crate::gfx::picture_store::PICTURE;
use crate::gfx::pixel_format::{PixelFormat, surface_format_addr};
use crate::memory::MemoryList;
use crate::memory::allocate::allocate;
use crate::memory::free_list::free_list;
use crate::memory::take_from_list::take_from_list;
use crate::pointers::GamePtr;
use crate::session::SCREEN;
use crate::session::{Dword, Shared};
use crate::strings::compile_lexicon::LEXICON;

/// IDA's address of the glyph tables' variables.
pub const START: usize = 0x51B980;
/// Their length.
pub const LEN: usize = 0x51B9C4 - START;

/// The fonts' and the sprites' tables.
#[repr(C)]
pub struct GlyphTables {
    /// `dword_51B980`: the tables' size in units of 0x400 bytes times `dword_4F4958`
    /// (`bundleheapsize`, a `var\ral.cfg` setting).
    pub units: Dword,
    /// `g_glyphTablesMemory` (`stru_51B984`): the list the fonts' and the sprites' tables
    /// lie on (16 bytes).
    pub memory: Shared<MemoryList>,
    /// `0x51B994` .. `0x51B9C4`: nothing reads or writes them (a gap).
    pub _gap_14: [Dword; 12],
}

/// The glyph tables' variables; the first values are the image's.
pub static TABLES: GlyphTables = GlyphTables {
    units: Dword::new(0x15E),
    memory: Shared::new(MemoryList {
        head: 0,
        total: 0,
        cursor: 0,
        remaining: 0,
    }),
    _gap_14: [const { Dword::new(0) }; 12],
};

/// Where the block is (`moved::MOVED`'s home for IDA 0x51B980).
pub fn home() -> usize {
    crate::pointers::host_number(&TABLES as *const GlyphTables)
}

/// The work memory taken, the picture's size set for sheets, the pixel layout the
/// surface's, and no table yet.
pub fn begin_glyph_tables() -> Result<(), Refused> {
    PICTURE.width.set(800u32);
    PICTURE.height.set(600u32);
    PICTURE.row_bytes.set(0xFFFF_FFFDu32);
    DECODER.format.set(surface_format_addr() as u32);
    let bytes = LEXICON
        .kilobytes
        .get()
        .wrapping_mul(0x400)
        .wrapping_mul(SCREEN.bytes.get());
    // (The original's fatal error 0x80E48021 for no memory cannot come: the allocator
    // reports its own.)
    allocate(LEXICON.memory.as_ptr(), bytes)?;
    SHEET.table.set(0u32);
    SHEET.table_size.set(0u32);
    Ok(())
}

/// A new table starts where the list stands, after the padding that brings the table
/// before it to whole dwords.
pub fn begin_glyph_table(list: *mut MemoryList) -> Result<(), BadInput> {
    let padding = SHEET.table_size.get().wrapping_neg() & 3;
    take_from_list(list, padding)?;
    SHEET.table.set(take_from_list(list, 0)?.unwrap_or(0));
    SHEET.table_size.set(0u32);
    Ok(())
}

pub fn end_glyph_tables() -> Result<(), BadInput> {
    free_list(LEXICON.memory.as_ptr())
}

/// The last sheet's palette as 256 pixels of the builder's layout at `to`; nothing if `to`
/// is 0 or the last sheet was one of components.
pub fn palette_to_pixels(to: u32) -> Result<(), BadInput> {
    if to == 0 || DECODER.builder_flags.get() & DIRECT_COLOUR != 0 {
        return Ok(());
    }
    let format = DECODER.format.get();
    if format == 0 {
        return Err(BadInput("no pixel layout for the palette"));
    }
    let format: PixelFormat = GamePtr::new(format).read("the palette's pixel layout")?;
    let mut pixels = Vec::with_capacity(256 * 4);
    for entry in 0..256 {
        let components: u32 = DECODER.palette[entry].get();
        pixels.extend_from_slice(&components_to_pixel(&format, components).to_le_bytes());
    }
    GamePtr::new(to).write_bytes(&pixels, "a palette's pixels")
}

/// The last sheet's palette for pixels that are a colour in the low four bits and an alpha
/// in the top four: entry 0 becomes 0, every entry the colour of its low four bits'
/// entry, and its fourth byte the alpha, the four bits twice.
pub fn alpha_palette() {
    let palette = &DECODER.palette;
    palette[0].set(0);
    for entry in 0..256usize {
        let colour = palette[entry & 0x0F].get();
        let alpha = (entry & 0xF0 | entry >> 4) as u32;
        palette[entry].set(colour & 0x00FF_FFFF | alpha << 24);
    }
}

#[cfg(test)]
mod block_tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&TABLES, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), TABLES.units.addr());
        assert_eq!(crate::moved::place(0x51B984), TABLES.memory.addr());
    }
}
