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
use crate::gfx::build_glyphs::SheetVars;
use crate::gfx::build_glyphs::build_glyphs;
use crate::gfx::glyph_builder::RUN_LENGTHS;
use crate::gfx::glyph_tables::{
    begin_glyph_table, begin_glyph_tables, end_glyph_tables, palette_to_pixels,
};
use crate::memory::take_from_list::{rewind_list, take_from_list};
use crate::pointers::GamePtr;
use crate::session::Dword;

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
        let fonts = &gfx().fonts;
        match self {
            Source::Palettes => &fonts.menu_fonts.palettes,
            Source::Font13 => &fonts.menu_fonts.font13_glyphs,
            Source::Font21 => &fonts.menu_fonts.font21_glyphs,
            Source::Font23 => &fonts.menu_fonts.font23_glyphs,
            Source::Font27 => &fonts.menu_fonts.font27_glyphs,
        }
    }
}
const PALETTE_BYTES: usize = 0x400;
/// The four font sheets and where each one's table is kept.
fn font_sheets() -> [(&'static [u8], &'static Dword); 4] {
    let fonts = &gfx().fonts;
    use crate::gfx::texts::{FNT13, FNT21, FNT23, FNT27};
    [
        (&FNT13, &fonts.menu_fonts.font13_glyphs), // fnt13_00.pcx
        (&FNT21, &fonts.menu_fonts.font21_glyphs), // fnt21_00.pcx
        (&FNT23, &fonts.menu_fonts.font23_glyphs), // fnt23_00.pcx
        (&FNT27, &fonts.menu_fonts.font27_glyphs), // fnt27_00.pcx
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
use crate::gfx::owner::gfx;
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
fn tables(vars: &SheetVars, palettes: &Dword) -> Result<(), Refused> {
    let gfx = gfx();
    let list = crate::pointers::GamePtr::to(&gfx.fonts.glyph_tables.memory);
    let bytes = gfx
        .fonts
        .glyph_tables
        .units
        .get()
        .wrapping_mul(0x400)
        .wrapping_mul(crate::system::owner::system().session.screen.bytes.get());
    crate::memory::owned::count_on(list, bytes)?;
    rewind_list(list)?;
    begin_glyph_tables(vars)?;
    gfx.pictures.decoder.builder_flags.set(RUN_LENGTHS);
    palettes.set(palette_place(list)?);
    begin_glyph_table(vars, list)?;
    build_glyphs(vars, &crate::gfx::texts::PAL_00, list)?;
    gfx.fonts.menu_fonts.font27_glyphs.set(vars.table.get());
    palette_to_pixels(palettes.get())?;
    GamePtr::to(&gfx.pictures.decoder.palette).copy_to(
        GamePtr::to(&gfx.fonts.menu_fonts.palette_sheet),
        PALETTE_BYTES,
        "the fonts' palette sheet",
    )?;
    for (sheet, kept) in font_sheets() {
        begin_glyph_table(vars, list)?;
        build_glyphs(vars, sheet, list)?;
        kept.set(vars.table.get());
    }
    end_glyph_tables()?;
    Ok(())
}

/// 0x400 bytes carved off `list` for a palette: its place in `gfx::glyph_store`, or 0
/// when the list has no room (the original's cursor would have been none).
pub fn palette_place(list: crate::pointers::GamePtr) -> Result<u32, Refused> {
    Ok(match take_from_list(list, 0x400)? {
        Some(_) => crate::gfx::glyph_store::new_palette(list)?,
        None => 0,
    })
}

/// What a store of `HUNG` stores.
fn hung_value(variable: Source, plus: u32) -> u32 {
    variable.cell().get().wrapping_add(plus)
}

pub fn build_fonts() -> Result<(), Refused> {
    let gfx = gfx();
    if gfx.fonts.menu_fonts.palettes.get() != 0 {
        return Err(FatalError {
            code: 0x80A4_8020,
            message: "",
        }
        .into());
    }
    tables(&gfx.sheet, &gfx.fonts.menu_fonts.palettes)?;
    for (font, field, variable, plus) in HUNG {
        let object = &gfx.fonts.typefaces.objects[font];
        let value = hung_value(variable, plus);
        match field {
            Palette => object.palette.set(value),
            GlyphTable => object.glyph_table.set(value),
        }
    }
    Ok(())
}
