//! `gfx::build_game_fonts` (0x41B178): builds the font with an alpha that a stage's loading
//! asks for, and hangs it on five font objects.
//!
//! In:  nothing
//! Out: every register preserved
//! ("LOADLEVEL_INIT : loadgamefonts" in the game's log; one caller, `sub_44426F`.)
//! Takes `dword_5BE894` * 0x400 * `dword_4F4958` bytes on the list `stru_5BE898` (fatal
//! error 0x80E48061 if there are none), `begin_glyph_tables`, the builder's flags = 0x21
//! (run-length coded, with alpha sheets), the pixel layout at `off_5ABACC` + 8, twice
//! 0x400 bytes off the list for two palettes, a new table, `gfx::build_glyphs` on
//! `files\d\fnt\fnt22_00.pcx` (0x80E48062 if that leaves no table), `alpha_palette`, the
//! palette's pixels to the first palette, `end_glyph_tables`. The fonts at 0x51E2E4 and
//! 0x51E304 get the first palette and the table. Then the sheet's palette is made
//! brighter where it lies (first byte + 0x40, second and third + 0x20, each stopping at
//! 0xFF) and its pixels are the second palette, which the fonts at 0x51E324, 0x51E344 and
//! 0x51E364 get with the same table.

use crate::errors::{BadInput, FatalError, Refused};
use crate::gfx::build_glyphs::SHEET;
use crate::gfx::build_glyphs::build_glyphs;
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::decoder::DECODER;
use crate::gfx::fonts_store::FONTS;
use crate::gfx::glyph_builder::{RUN_LENGTHS, WITH_ALPHA};
use crate::gfx::glyph_tables::{
    alpha_palette, begin_glyph_table, begin_glyph_tables, end_glyph_tables, palette_to_pixels,
};
use crate::gfx::pixel_format::PixelFormat;
use crate::memory::MemoryList;
use crate::memory::allocate::allocate;
use crate::memory::take_from_list::{rewind_list, take_from_list};
use crate::pointers::GamePtr;
use crate::session::SCREEN;

// ---- game data ----

/// IDA's address of the game fonts' memory variables.
pub const START: usize = 0x5BE894;
/// Their length.
pub const LEN: usize = 0x5BE8D8 - START;

/// The game fonts' memory.
#[repr(C)]
pub struct GameFontsMemory {
    /// `dword_5BE894`: the memory's size in units of 0x400 bytes times `dword_4F4958`
    /// (`gamegfxheapsize`, a `var\ral.cfg` setting).
    pub units: crate::session::Dword,
    /// `g_gameFontMemory` (`stru_5BE898`): the list the fonts' tables lie on (16 bytes).
    pub memory: crate::session::Shared<MemoryList>,
    /// `0x5BE8A8` .. `0x5BE8D8`: nothing reads or writes them (a gap).
    pub _gap_14: [crate::session::Dword; 12],
}

/// The game fonts' memory; the first values are the image's.
pub static GAME_FONTS: GameFontsMemory = GameFontsMemory {
    units: crate::session::Dword::new(0x20),
    memory: crate::session::Shared::new(MemoryList {
        head: 0,
        total: 0,
        cursor: 0,
        remaining: 0,
    }),
    _gap_14: [const { crate::session::Dword::new(0) }; 12],
};

/// Where the block is (`moved::MOVED`'s home for IDA 0x5BE894).
pub fn home() -> usize {
    crate::pointers::game_address(&GAME_FONTS as *const GameFontsMemory) as usize
}

// The first palette's place (dword_51E2D8) and the table (dword_51E2DC) are the palette and
// glyph table of `FONTS.game_fonts[0]`; the five font objects after it (IDA 0x51E2E4) are
// `game_fonts[1 ..]`. The second palette's place is the palette of `game_fonts[3]`.

// ---- Rust version ----

fn fatal(code: u32) -> Refused {
    FatalError { code, message: "" }.into()
}

/// `pixels_b` is where the place of the second palette's pixels is read from when they
/// are made. (The original's fatal error 0x80E48061 for a failed allocation cannot come:
/// the allocator reports its own.)
pub unsafe fn build_game_fonts(pixels_b: u32) -> Result<(), Refused> {
    unsafe {
        let list = GAME_FONTS.memory.as_ptr();
        let bytes = GAME_FONTS
            .units
            .get()
            .wrapping_mul(0x400)
            .wrapping_mul(SCREEN.bytes.get());
        allocate(list, bytes)?;
        rewind_list(list)?;
        begin_glyph_tables()?;
        DECODER.builder_flags.set(RUN_LENGTHS | WITH_ALPHA);
        // The textures' 8888 kind's description (`kinds[3]`): its pixel layout is 8 bytes in.
        let format = crate::gfx::texture_upload::TEXTURE_TABLES.kinds[3]
            .get()
            .wrapping_add(8);
        DECODER.format.set(format);
        let palette_a = take_from_list(list, 0x400)?.unwrap_or(0);
        FONTS.game_fonts[0].update(|f| f.palette = palette_a);
        let palette_b = take_from_list(list, 0x400)?.unwrap_or(0);
        FONTS.game_fonts[3].update(|f| f.palette = palette_b);
        begin_glyph_table(list)?;
        build_glyphs(
            crate::pointers::host(crate::gfx::texts::address(&crate::gfx::texts::FNT22))
                as *const u8,
            list,
        )?;
        let table = SHEET.table.get();
        FONTS.game_fonts[0].update(|f| f.glyph_table = table);
        if SHEET.table_size.get() == 0 {
            return Err(fatal(0x80E4_8062));
        }
        alpha_palette();
        palette_to_pixels(FONTS.game_fonts[0].get().palette)?;
        end_glyph_tables()?;

        let (glyphs, palette_a): (u32, u32) = (
            FONTS.game_fonts[0].get().glyph_table,
            FONTS.game_fonts[0].get().palette,
        );
        for font in 1..3 {
            FONTS.game_fonts[font].update(|f| {
                f.glyph_table = glyphs;
                f.palette = palette_a;
            });
        }

        // The sheet's palette made brighter where it lies, and its pixels.
        let to: u32 = GamePtr::new(pixels_b).read("the second palette's place")?;
        let layout = DECODER.format.get();
        if to == 0 || layout == 0 {
            return Err(BadInput("no place for the second palette, or no pixel layout").into());
        }
        let layout: PixelFormat = GamePtr::new(layout).read("the palette's pixel layout")?;
        let mut pixels = Vec::with_capacity(256 * 4);
        for n in 0..256usize {
            // Converted: `rc2k_core::gfx::pixels::brighter`.
            let brighter = rc2k_core::gfx::pixels::brighter(DECODER.palette[n].get());
            DECODER.palette[n].set(brighter);
            pixels.extend_from_slice(&components_to_pixel(&layout, brighter).to_le_bytes());
        }
        GamePtr::new(to).write_bytes(&pixels, "the second palette's pixels")?;

        let palette_b: u32 = FONTS.game_fonts[3].get().palette;
        FONTS.game_fonts[3].update(|f| f.glyph_table = glyphs);
        for font in 4..6 {
            FONTS.game_fonts[font].update(|f| {
                f.glyph_table = glyphs;
                f.palette = palette_b;
            });
        }
        Ok(())
    }
}

/// With the third font's palette's place, as the stage's loading calls it.
pub unsafe fn build_game_fonts_in_game() -> Result<(), Refused> {
    unsafe {
        build_game_fonts(crate::pointers::game_address(
            &raw const (*FONTS.game_fonts[3].as_ptr()).palette,
        ))
    }
}

#[cfg(test)]
mod block_tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&GAME_FONTS, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), GAME_FONTS.units.addr() as usize);
        assert_eq!(
            crate::moved::place(0x5BE898),
            GAME_FONTS.memory.addr() as usize
        );
    }
}
