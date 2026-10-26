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
//! Shadow mode runs the original alone (the memory is taken once) and checks the stores
//! and the second palette; the table is compared in `gfx::build_glyphs`' hook. Self-test:
//! a redirected copy whose ten calls note themselves and which writes the second
//! palette's pixels where the test says.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::gfx::build_glyphs::{BUILDER_FLAGS_RVA, FORMAT_RVA, TABLE_RVA, TABLE_SIZE_RVA};
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::glyph_builder::{RUN_LENGTHS, WITH_ALPHA};
use crate::gfx::glyph_tables::{
    ALLOCATE, ALPHA_PALETTE, BEGIN_TABLE, BEGIN_TABLES, BUILD_GLYPHS, END_TABLES, Game,
    PALETTE_TO_PIXELS, Parts, REWIND_LIST, TAKE_FROM_LIST, UNIT_FACTOR_RVA,
};
use crate::gfx::pcx::PALETTE_RVA;
use crate::gfx::pixel_format::PixelFormat;

// ---- game data ----

const LIST_RVA: usize = rva(0x5BE898); // stru_5BE898
const UNITS_RVA: usize = rva(0x5BE894); // dword_5BE894
/// off_5ABACC: something whose pixel layout is 8 bytes in (the textures', a reading).
const LAYOUT_OWNER_RVA: usize = rva(0x5ABACC);
const SHEETS_RVA: usize = rva(0x5BE8D8); // "files\d\fnt\fnt22_00.pcx"
const PALETTE_A_RVA: usize = rva(0x51E2D8); // dword_51E2D8: the first palette's place
const GLYPHS_RVA: usize = rva(0x51E2DC); // dword_51E2DC: the table
/// The five font objects, 0x20 bytes each: palette at +0x14, glyph table at +0x18.
const FONTS_RVA: usize = rva(0x51E2E4);
const FONT_PALETTE: usize = 0x14;
const FONT_GLYPHS: usize = 0x18;
/// The palette of the third font: where the second palette's place is kept.
const PALETTE_B_RVA: usize = FONTS_RVA + 2 * 0x20 + FONT_PALETTE;

// ---- Rust version ----

fn fatal(code: u32) -> Refused {
    FatalError { code, message: "" }.into()
}

/// `entry` are the registers at entry; `pixels_b` is where the place of the second
/// palette's pixels is read from when they are made.
pub unsafe fn build_game_fonts(
    entry: &Regs,
    parts: &mut dyn Parts,
    pixels_b: *const u32,
) -> Result<(), Refused> {
    unsafe {
        let mut g = *entry;
        g.eax = game_read::<u32>(UNITS_RVA)
            .wrapping_mul(0x400)
            .wrapping_mul(game_read(UNIT_FACTOR_RVA));
        g.ebx = game_addr(LIST_RVA) as u32;
        parts.call(ALLOCATE, &mut g)?;
        if g.carry() {
            return Err(fatal(0x80E4_8061));
        }
        parts.call(REWIND_LIST, &mut g)?;
        parts.call(BEGIN_TABLES, &mut g)?;
        game_write(BUILDER_FLAGS_RVA, RUN_LENGTHS | WITH_ALPHA);
        let format = game_read::<u32>(LAYOUT_OWNER_RVA).wrapping_add(8);
        game_write(FORMAT_RVA, format);
        g.eax = 0x400;
        parts.call(TAKE_FROM_LIST, &mut g)?;
        game_write(PALETTE_A_RVA, g.eax);
        g.eax = 0x400;
        parts.call(TAKE_FROM_LIST, &mut g)?;
        game_write(PALETTE_B_RVA, g.eax);
        parts.call(BEGIN_TABLE, &mut g)?;
        g.eax = game_addr(SHEETS_RVA) as u32;
        parts.call(BUILD_GLYPHS, &mut g)?;
        g.eax = game_read(TABLE_RVA);
        game_write(GLYPHS_RVA, g.eax);
        if game_read::<u32>(TABLE_SIZE_RVA) == 0 {
            return Err(fatal(0x80E4_8062));
        }
        parts.call(ALPHA_PALETTE, &mut g)?;
        g.edi = game_read(PALETTE_A_RVA);
        parts.call(PALETTE_TO_PIXELS, &mut g)?;
        parts.call(END_TABLES, &mut g)?;

        let (glyphs, palette_a): (u32, u32) = (game_read(GLYPHS_RVA), game_read(PALETTE_A_RVA));
        for font in 0..2 {
            game_write(FONTS_RVA + font * 0x20 + FONT_GLYPHS, glyphs);
            game_write(FONTS_RVA + font * 0x20 + FONT_PALETTE, palette_a);
        }

        // The sheet's palette made brighter where it lies, and its pixels.
        let to = pixels_b.read_unaligned() as *mut u32;
        let layout = game_read::<u32>(FORMAT_RVA) as *const PixelFormat;
        if to.is_null() || layout.is_null() {
            return Err(BadInput("no place for the second palette, or no pixel layout").into());
        }
        let layout = layout.read_unaligned();
        for n in 0..256usize {
            let place = game_addr(PALETTE_RVA + n * 4) as *mut u32;
            // Converted: `rc2k_core::gfx::pixels::brighter`.
            let brighter = rc2k_core::gfx::pixels::brighter(place.read_unaligned());
            place.write_unaligned(brighter);
            to.add(n)
                .write_unaligned(components_to_pixel(&layout, brighter));
        }

        let palette_b: u32 = game_read(PALETTE_B_RVA);
        game_write(FONTS_RVA + 2 * 0x20 + FONT_GLYPHS, glyphs);
        for font in 3..5 {
            game_write(FONTS_RVA + font * 0x20 + FONT_GLYPHS, glyphs);
            game_write(FONTS_RVA + font * 0x20 + FONT_PALETTE, palette_b);
        }
        Ok(())
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    let pixels_b = game_addr(PALETTE_B_RVA) as *const u32;
    unsafe { build_game_fonts(&entry, &mut Game, pixels_b) }?;
    Ok(())
}
