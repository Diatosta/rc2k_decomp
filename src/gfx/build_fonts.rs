//! `gfx::build_fonts` (0x415563): builds the fonts' glyph tables and palettes and hangs
//! them on the font objects.
//!
//! In:  nothing
//! Out: every register preserved
//! Fatal error 0x80A48020 if it has been done (`dword_51DEB0`, the palettes, is not 0).
//! Takes `dword_51B980` * 0x400 * `dword_4F4958` bytes on the list `stru_51B984`
//! (0x80E48023 if there are none), `begin_glyph_tables`, the builder's flags = 1, 0x400
//! bytes off the list for the palettes; then five builds with `gfx::build_glyphs`, a new
//! table for each: `files\d\fnt\pal_00.pcx` (a sheet without glyphs: its palette becomes
//! the 256 pixels at the palettes' place, and is kept as components at `unk_51DEC4`),
//! `fnt13_00.pcx`, `fnt21_00.pcx`, `fnt23_00.pcx`, `fnt27_00.pcx`; `end_glyph_tables`. Then
//! 125 stores: each font object's palette (+0x14: the palettes' place plus the offset of
//! its sixteen colours) and glyph table (+0x18), as listed in `HUNG`. A font that measures
//! (mode 4) gets a palette too, which `text::draw_text` then uses as its line counter.
//! Callers: `sub_47486C` (the game's start, before the main menu), `sub_4154CD`,
//! `sub_4703C2`, `sub_41CDC2`, `sub_412DE0`, `sub_443104`.
//! A refusal from one of the steps leaves the work half done; none can come with the
//! game's own files.
//! The registers above are the original's, for reference; a call by address goes through
//! `routes.rs`.

use crate::errors::{FatalError, Refused};
use crate::gfx::build_glyphs::SHEET;
use crate::gfx::build_glyphs::build_glyphs;
use crate::gfx::decoder::DECODER;
use crate::gfx::fonts_store::FONTS as MENU_FONTS;
use crate::gfx::glyph_builder::RUN_LENGTHS;
use crate::gfx::glyph_tables::{
    TABLES, begin_glyph_table, begin_glyph_tables, end_glyph_tables, palette_to_pixels,
};
use crate::gfx::typefaces::FONTS;
use crate::memory::allocate::allocate;
use crate::memory::take_from_list::{rewind_list, take_from_list};
use crate::pointers::GamePtr;
use crate::session::{Dword, SCREEN};

// ---- game data ----

/// The variables a store of `HUNG` reads: `FONTS.palettes` (dword_51DEB0: 256 pixels, the
/// fonts' palettes; 0 = the fonts are not built) and the four glyph tables' places.
#[derive(Clone, Copy)]
enum Source {
    Palettes,
    Font13,
    Font21,
    Font23,
    Font27,
}

impl Source {
    fn cell(self) -> &'static Dword {
        match self {
            Source::Palettes => &MENU_FONTS.palettes,
            Source::Font13 => &MENU_FONTS.font13_glyphs,
            Source::Font21 => &MENU_FONTS.font21_glyphs,
            Source::Font23 => &MENU_FONTS.font23_glyphs,
            Source::Font27 => &MENU_FONTS.font27_glyphs,
        }
    }
}
const PALETTE_BYTES: usize = 0x400;
/// The four font sheets and where each one's table is kept.
fn font_sheets() -> [(u32, &'static Dword); 4] {
    use crate::gfx::texts::{FNT13, FNT21, FNT23, FNT27, address};
    [
        (address(&FNT13), &MENU_FONTS.font13_glyphs), // fnt13_00.pcx
        (address(&FNT21), &MENU_FONTS.font21_glyphs), // fnt21_00.pcx
        (address(&FNT23), &MENU_FONTS.font23_glyphs), // fnt23_00.pcx
        (address(&FNT27), &MENU_FONTS.font27_glyphs), // fnt27_00.pcx
    ]
}

const P: Source = Source::Palettes;
const F13: Source = Source::Font13;
const F21: Source = Source::Font21;
const F23: Source = Source::Font23;
const F27: Source = Source::Font27;
/// Which field of a font object a store sets.
#[derive(Clone, Copy)]
enum Field {
    Palette,
    GlyphTable,
}
use Field::{GlyphTable, Palette};

/// The stores, in the original's order: (the font object's number, its field, the variable
/// stored, plus this much).
#[rustfmt::skip]
const HUNG: [(usize, Field, Source, u32); 125] = [
    (0, Palette, P, 0x0),
    (0, GlyphTable, F13, 0x0),
    (2, Palette, P, 0x300),
    (2, GlyphTable, F13, 0x0),
    (1, Palette, P, 0x300),
    (1, GlyphTable, F13, 0x0),
    (3, Palette, P, 0x300),
    (3, GlyphTable, F13, 0x0),
    (4, Palette, P, 0x0),
    (4, GlyphTable, F21, 0x0),
    (6, Palette, P, 0x180),
    (6, GlyphTable, F21, 0x0),
    (7, Palette, P, 0x180),
    (7, GlyphTable, F21, 0x0),
    (5, Palette, P, 0x180),
    (5, GlyphTable, F21, 0x0),
    (9, Palette, P, 0x200),
    (9, GlyphTable, F21, 0x0),
    (10, Palette, P, 0x200),
    (10, GlyphTable, F21, 0x0),
    (8, Palette, P, 0x200),
    (8, GlyphTable, F21, 0x0),
    (12, Palette, P, 0x80),
    (12, GlyphTable, F21, 0x0),
    (13, Palette, P, 0x80),
    (13, GlyphTable, F21, 0x0),
    (11, Palette, P, 0x80),
    (11, GlyphTable, F21, 0x0),
    (15, Palette, P, 0x0),
    (15, GlyphTable, F21, 0x0),
    (14, Palette, P, 0x0),
    (14, GlyphTable, F21, 0x0),
    (16, Palette, P, 0x0),
    (16, GlyphTable, F21, 0x0),
    (18, Palette, P, 0x380),
    (18, GlyphTable, F21, 0x0),
    (17, Palette, P, 0x380),
    (17, GlyphTable, F21, 0x0),
    (19, Palette, P, 0x380),
    (19, GlyphTable, F21, 0x0),
    (20, Palette, P, 0x0),
    (20, GlyphTable, F23, 0x0),
    (22, Palette, P, 0x80),
    (22, GlyphTable, F23, 0x0),
    (21, Palette, P, 0x80),
    (21, GlyphTable, F23, 0x0),
    (23, Palette, P, 0x80),
    (23, GlyphTable, F23, 0x0),
    (25, Palette, P, 0x0),
    (25, GlyphTable, F23, 0x0),
    (24, Palette, P, 0x0),
    (24, GlyphTable, F23, 0x0),
    (26, Palette, P, 0x0),
    (26, GlyphTable, F23, 0x0),
    (28, Palette, P, 0x380),
    (28, GlyphTable, F23, 0x0),
    (27, Palette, P, 0x380),
    (27, GlyphTable, F23, 0x0),
    (29, Palette, P, 0x380),
    (29, GlyphTable, F23, 0x0),
    (31, Palette, P, 0x340),
    (31, GlyphTable, F23, 0x0),
    (30, Palette, P, 0x340),
    (30, GlyphTable, F23, 0x0),
    (32, Palette, P, 0x340),
    (32, GlyphTable, F23, 0x0),
    (34, Palette, P, 0x180),
    (34, GlyphTable, F23, 0x0),
    (33, Palette, P, 0x180),
    (33, GlyphTable, F23, 0x0),
    (35, Palette, P, 0x180),
    (35, GlyphTable, F23, 0x0),
    (36, Palette, P, 0x0),
    (36, GlyphTable, F27, 0x0),
    (38, Palette, P, 0x80),
    (38, GlyphTable, F27, 0x0),
    (37, Palette, P, 0x80),
    (37, GlyphTable, F27, 0x0),
    (39, Palette, P, 0x80),
    (39, GlyphTable, F27, 0x0),
    (41, Palette, P, 0x0),
    (41, GlyphTable, F27, 0x0),
    (40, Palette, P, 0x0),
    (40, GlyphTable, F27, 0x0),
    (42, Palette, P, 0x0),
    (42, GlyphTable, F27, 0x0),
    (44, Palette, P, 0x100),
    (44, GlyphTable, F27, 0x0),
    (43, Palette, P, 0x100),
    (43, GlyphTable, F27, 0x0),
    (45, Palette, P, 0x100),
    (45, GlyphTable, F27, 0x0),
    (47, Palette, P, 0x380),
    (47, GlyphTable, F27, 0x0),
    (46, Palette, P, 0x380),
    (46, GlyphTable, F27, 0x0),
    (48, Palette, P, 0x380),
    (48, GlyphTable, F27, 0x0),
    (53, Palette, P, 0x340),
    (53, GlyphTable, F27, 0x0),
    (52, Palette, P, 0x340),
    (52, GlyphTable, F27, 0x0),
    (54, Palette, P, 0x340),
    (54, GlyphTable, F27, 0x0),
    (50, Palette, P, 0x180),
    (50, GlyphTable, F27, 0x0),
    (49, Palette, P, 0x180),
    (49, GlyphTable, F27, 0x0),
    (51, Palette, P, 0x180),
    (51, GlyphTable, F27, 0x0),
    (55, Palette, P, 0xC0),
    (55, GlyphTable, F27, 0x0),
    (56, GlyphTable, F27, 0x0),
    (58, Palette, P, 0x300),
    (58, GlyphTable, F13, 0x0),
    (57, Palette, P, 0x300),
    (57, GlyphTable, F13, 0x0),
    (59, Palette, P, 0x300),
    (59, GlyphTable, F13, 0x0),
    (61, Palette, P, 0x0),
    (61, GlyphTable, F21, 0x0),
    (60, Palette, P, 0x0),
    (60, GlyphTable, F21, 0x0),
    (62, Palette, P, 0x0),
    (62, GlyphTable, F21, 0x0),
];

// ---- Rust version ----

/// The part with the calls; `palettes` is where the palettes' place is kept. (The
/// original's fatal error 0x80E48023 for a failed allocation cannot come: the allocator
/// reports its own.)
unsafe fn tables(palettes: u32) -> Result<(), Refused> {
    unsafe {
        let list = TABLES.memory.as_ptr();
        let bytes = TABLES
            .units
            .get()
            .wrapping_mul(0x400)
            .wrapping_mul(SCREEN.bytes.get());
        allocate(list, bytes)?;
        rewind_list(list)?;
        begin_glyph_tables()?;
        DECODER.builder_flags.set(RUN_LENGTHS);
        GamePtr::new(palettes).write(
            take_from_list(list, 0x400)?.unwrap_or(0),
            "the fonts' palettes' place",
        )?;
        begin_glyph_table(list)?;
        build_glyphs(
            crate::gfx::texts::address(&crate::gfx::texts::PAL_00) as *const u8,
            list,
        )?;
        MENU_FONTS.font27_glyphs.set(SHEET.table.get());
        palette_to_pixels(GamePtr::new(palettes).read("the fonts' palettes' place")?)?;
        GamePtr::new(DECODER.palette_ptr() as usize as u32).copy_to(
            GamePtr::new(crate::session::address_of(&MENU_FONTS.palette_sheet) as u32),
            PALETTE_BYTES,
            "the fonts' palette sheet",
        )?;
        for (sheet, kept) in font_sheets() {
            begin_glyph_table(list)?;
            build_glyphs(sheet as *const u8, list)?;
            kept.set(SHEET.table.get());
        }
        end_glyph_tables()?;
        Ok(())
    }
}

/// What a store of `HUNG` stores.
fn hung_value(variable: Source, plus: u32) -> u32 {
    variable.cell().get().wrapping_add(plus)
}

pub unsafe fn build_fonts() -> Result<(), Refused> {
    unsafe {
        if MENU_FONTS.palettes.get() != 0 {
            return Err(FatalError {
                code: 0x80A4_8020,
                message: "",
            }
            .into());
        }
        tables(MENU_FONTS.palettes.addr() as u32)?;
        for (font, field, variable, plus) in HUNG {
            let object = &FONTS.objects[font];
            let value = hung_value(variable, plus);
            match field {
                Palette => object.palette.set(value),
                GlyphTable => object.glyph_table.set(value),
            }
        }
        Ok(())
    }
}
