//! Builds the font with an alpha that a stage's loading asks for, and hangs it on five font objects
//! ("LOADLEVEL_INIT : loadgamefonts" in the game's log; one caller, `race::loadlevel_init`). Takes
//! `units` * 0x400 * the pixel size bytes on the list `memory` (fatal error 0x80E48061 if there are
//! none), `begin_glyph_tables`, the builder's flags = 0x21 (run-length coded, with alpha sheets),
//! the pixel layout (a fixed table's entry), twice 0x400 bytes off the list for two palettes, a new
//! table, `gfx::build_glyphs` on `files\d\fnt\fnt22_00.pcx` (0x80E48062 if that leaves no table),
//! `alpha_palette`, the palette's pixels to the first palette, `end_glyph_tables`. `game_fonts[1]`
//! and `[2]` get the first palette and the table. Then the sheet's palette is made brighter where
//! it lies (first byte + 0x40, second and third + 0x20, each stopping at 0xFF) and its pixels are
//! the second palette, which `game_fonts[3 ..= 5]` get with the same table.

use crate::errors::{BadInput, FatalError, Refused};
use crate::gfx::build_glyphs::SheetVars;
use crate::gfx::build_glyphs::build_glyphs;
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::glyph_builder::{RUN_LENGTHS, WITH_ALPHA};
use crate::gfx::glyph_tables::{
    alpha_palette, begin_glyph_table, begin_glyph_tables, end_glyph_tables, palette_to_pixels,
};
use crate::memory::MemoryList;
use crate::memory::take_from_list::rewind_list;

/// The game fonts' memory.
#[derive(rc2k_derive::Cells)]
pub struct GameFontsMemory {
    /// The memory's size in units of 0x400 bytes times `Screen.bytes` (`gamegfxheapsize`, a
    /// `var\ral.cfg` setting).
    pub units: crate::memory::session::PlainDword,
    /// `GameFontsMemory.memory.head`: the list the fonts' tables lie on (16 bytes).
    pub memory: crate::memory::ListCells,
}

impl GameFontsMemory {
    /// The game fonts' memory.
    pub const fn new() -> Self {
        GameFontsMemory {
            units: crate::memory::session::PlainDword::new(0x20),
            memory: crate::memory::ListCells::new(MemoryList {
                head: 0,
                total: 0,
                cursor: 0,
                remaining: 0,
            }),
        }
    }
}

// The first palette's place and the table are the palette and glyph table of `FONTS.game_fonts[0]`;
// the five font objects after it are `game_fonts[1 ..]`. The second palette's place is the palette
// of `game_fonts[3]`.

fn fatal(code: u32) -> Refused {
    FatalError { code, message: "" }.into()
}

/// `pixels_b` is where the place of the second palette's pixels is read from when they are made. (A
/// failed allocation cannot come: the allocator reports its own error.)
pub fn build_game_fonts(
    vars: &SheetVars,
    pixels_b: &crate::memory::session::PlainDword,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
    strings: &crate::strings::owner::Strings,
) -> Result<(), Refused> {
    let list = &gfx.fonts.game_fonts_memory.memory;
    let bytes = gfx
        .fonts
        .game_fonts_memory
        .units
        .get()
        .wrapping_mul(0x400)
        .wrapping_mul(system.session.screen.get().bytes.get());
    crate::memory::owned::count_on(list, bytes)?;
    rewind_list(list)?;
    crate::logfile::load_times::mark("begin_glyph_tables");
    begin_glyph_tables(vars, gfx, system, strings)?;
    gfx.pictures
        .decoder
        .builder_flags
        .set(RUN_LENGTHS | WITH_ALPHA);
    // The textures' 8888 kind's description (`kinds[3]`): its pixel layout (8 bytes in).
    gfx.pictures
        .decoder
        .format
        .set(crate::gfx::pixel_format::texture_kind_format(3, gfx));
    let palette_a = crate::gfx::build_fonts::palette_place(&gfx.fonts, list)?;
    gfx.fonts.font_block.game_fonts[0].update(|f| f.palette = palette_a);
    let palette_b = crate::gfx::build_fonts::palette_place(&gfx.fonts, list)?;
    gfx.fonts.font_block.game_fonts[3].update(|f| f.palette = palette_b);
    begin_glyph_table(&gfx.fonts, vars, list)?;
    crate::logfile::load_times::mark("build_glyphs fnt22");
    build_glyphs(vars, &crate::gfx::texts::FNT22, list, gfx, system, strings)?;
    let table = vars.table.get();
    gfx.fonts.font_block.game_fonts[0].update(|f| f.glyph_table = table);
    if vars.table_size.get() == 0 {
        return Err(fatal(0x80E4_8062));
    }
    crate::logfile::load_times::mark("alpha_palette, palette_to_pixels");
    alpha_palette(gfx);
    palette_to_pixels(
        gfx.fonts.font_block.game_fonts[0].get().palette,
        gfx,
        system,
    )?;
    end_glyph_tables(strings)?;

    let (glyphs, palette_a): (u32, u32) = (
        gfx.fonts.font_block.game_fonts[0].get().glyph_table,
        gfx.fonts.font_block.game_fonts[0].get().palette,
    );
    for item in gfx.fonts.font_block.game_fonts.iter().skip(1).take(3 - 1) {
        item.update(|f| {
            f.glyph_table = glyphs;
            f.palette = palette_a;
        });
    }

    crate::logfile::load_times::mark("brighter palette");
    // The sheet's palette made brighter where it lies, and its pixels.
    let to: u32 = pixels_b.get();
    let layout = gfx.pictures.decoder.format.get();
    if to == 0 || layout == 0 {
        return Err(BadInput("no place for the second palette, or no pixel layout").into());
    }
    let layout = crate::gfx::pixel_format::layout(system, gfx, layout)?;
    let mut pixels = Vec::with_capacity(256 * 4);
    for n in 0..256usize {
        let brighter = rc2k_core::gfx::pixels::brighter(gfx.pictures.decoder.palette[n].get());
        gfx.pictures.decoder.palette[n].set(brighter);
        pixels.extend_from_slice(&components_to_pixel(&layout, brighter, gfx).to_le_bytes());
    }
    crate::gfx::glyph_store::write_bytes(
        &gfx.fonts,
        to,
        0,
        &pixels,
        "the second palette's pixels",
    )?;

    let palette_b: u32 = gfx.fonts.font_block.game_fonts[3].get().palette;
    gfx.fonts.font_block.game_fonts[3].update(|f| f.glyph_table = glyphs);
    for item in gfx.fonts.font_block.game_fonts.iter().skip(4).take(6 - 4) {
        item.update(|f| {
            f.glyph_table = glyphs;
            f.palette = palette_b;
        });
    }
    Ok(())
}

/// With the third font's palette's place, as the stage's loading calls it.
pub fn build_game_fonts_in_game(
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
    strings: &crate::strings::owner::Strings,
) -> Result<(), Refused> {
    build_game_fonts(
        &gfx.sheet,
        &gfx.fonts.font_block.game_fonts[3].palette,
        gfx,
        system,
        strings,
    )
}
