//! The game's font objects, their palettes and glyph tables (`gfx::build_fonts`,
//! `gfx::build_game_fonts`, the screens that draw text), out of RAL.EXE's image,
//! typed: the menus' four fonts and their palette and glyph-table pointers, the
//! palette sheet, and the race's six fonts. A font is the 0x20-byte object `rc2k_core::text::Font`
//! describes, held in a `Shared` cell. Placed in a file of its own beside the graphics
//! module's stores.
//!
//! `#[repr(C)]` structs, first values the image's (`tests`), but for
//! the four menu fonts' palette and glyph table: the image holds the addresses of the jade
//! font's block there, and here they are its places in `gfx::glyph_store` (`JADE_COLOUR`,
//! `JADE`), converted once when the block is built.

use crate::gfx::typefaces::FontObject;
use crate::session::PlainDword;
use rc2k_core::text::Font;

/// The fonts, their palettes and glyph tables (IDA 0x51DE30 .. 0x51E384).
#[derive(rc2k_derive::Cells)]
pub struct FontBlock {
    /// 0x51DE30: `stru_51DE50` ..: the four fonts the menus and the race's text use (the third is the race's, `race::frame_text`).
    pub menu_fonts: [FontObject; 4],
    /// 0x51DEB0: `g_fontPalettes`: the font palettes (a pointer; `gfx::build_fonts`).
    pub palettes: PlainDword,
    /// 0x51DEB4: `g_font27Glyphs`: the 27-pixel font's glyph table.
    pub font27_glyphs: PlainDword,
    /// 0x51DEB8: `g_font23Glyphs`: the 23-pixel font's.
    pub font23_glyphs: PlainDword,
    /// 0x51DEBC: `g_font21Glyphs`: the 21-pixel font's.
    pub font21_glyphs: PlainDword,
    /// 0x51DEC0: `g_font13Glyphs`: the 13-pixel font's.
    pub font13_glyphs: PlainDword,
    /// 0x51DEC4: `g_fontPaletteComponents`: the palette sheet's 256 dwords.
    pub palette_sheet: [PlainDword; 256],
    /// 0x51E2C4: `unk_51E2C4` ..: the game's six fonts (`gfx::build_game_fonts`): the tyre change's and the name plates' texts.
    pub game_fonts: [FontObject; 6],
}

impl FontBlock {
    /// The first values are the image's.
    pub const fn new() -> Self {
        FontBlock {
            menu_fonts: [
                FontObject::from_font(Font {
                    first_char: 0x20,
                    last_char: 0x80,
                    mode: 0x4,
                    flags: 0x0,
                    spacing: 1,
                    line_height: 9,
                    y_offset: 0,
                    extended_spacing: 6,
                    palette: crate::gfx::glyph_store::JADE_COLOUR,
                    glyph_table: crate::gfx::glyph_store::JADE,
                    box_width: 0x0,
                }),
                FontObject::from_font(Font {
                    first_char: 0x20,
                    last_char: 0x80,
                    mode: 0x0,
                    flags: 0x0,
                    spacing: 1,
                    line_height: 9,
                    y_offset: 0,
                    extended_spacing: 6,
                    palette: crate::gfx::glyph_store::JADE_COLOUR,
                    glyph_table: crate::gfx::glyph_store::JADE,
                    box_width: 0x0,
                }),
                FontObject::from_font(Font {
                    first_char: 0x20,
                    last_char: 0x80,
                    mode: 0x2,
                    flags: 0x0,
                    spacing: 1,
                    line_height: 9,
                    y_offset: 0,
                    extended_spacing: 6,
                    palette: crate::gfx::glyph_store::JADE_COLOUR,
                    glyph_table: crate::gfx::glyph_store::JADE,
                    box_width: 0x0,
                }),
                FontObject::from_font(Font {
                    first_char: 0x20,
                    last_char: 0x80,
                    mode: 0x3,
                    flags: 0x0,
                    spacing: 1,
                    line_height: 9,
                    y_offset: 0,
                    extended_spacing: 6,
                    palette: crate::gfx::glyph_store::JADE_COLOUR,
                    glyph_table: crate::gfx::glyph_store::JADE,
                    box_width: 0x0,
                }),
            ],
            palettes: PlainDword::new(0x0),
            font27_glyphs: PlainDword::new(0x0),
            font23_glyphs: PlainDword::new(0x0),
            font21_glyphs: PlainDword::new(0x0),
            font13_glyphs: PlainDword::new(0x0),
            palette_sheet: [const { PlainDword::new(0x0) }; 256],
            game_fonts: [
                FontObject::from_font(Font {
                    first_char: 0x20,
                    last_char: 0xff,
                    mode: 0x0,
                    flags: 0x4,
                    spacing: 0,
                    line_height: 25,
                    y_offset: 0,
                    extended_spacing: 6,
                    palette: 0x0,
                    glyph_table: 0x0,
                    box_width: 0x0,
                }),
                FontObject::from_font(Font {
                    first_char: 0x20,
                    last_char: 0xff,
                    mode: 0x2,
                    flags: 0x4,
                    spacing: 0,
                    line_height: 25,
                    y_offset: 0,
                    extended_spacing: 6,
                    palette: 0x0,
                    glyph_table: 0x0,
                    box_width: 0x0,
                }),
                FontObject::from_font(Font {
                    first_char: 0x20,
                    last_char: 0xff,
                    mode: 0x3,
                    flags: 0x4,
                    spacing: 0,
                    line_height: 25,
                    y_offset: 0,
                    extended_spacing: 6,
                    palette: 0x0,
                    glyph_table: 0x0,
                    box_width: 0x0,
                }),
                FontObject::from_font(Font {
                    first_char: 0x20,
                    last_char: 0xff,
                    mode: 0x0,
                    flags: 0x4,
                    spacing: 0,
                    line_height: 25,
                    y_offset: 0,
                    extended_spacing: 6,
                    palette: 0x0,
                    glyph_table: 0x0,
                    box_width: 0x0,
                }),
                FontObject::from_font(Font {
                    first_char: 0x20,
                    last_char: 0xff,
                    mode: 0x2,
                    flags: 0x4,
                    spacing: 0,
                    line_height: 25,
                    y_offset: 0,
                    extended_spacing: 6,
                    palette: 0x0,
                    glyph_table: 0x0,
                    box_width: 0x0,
                }),
                FontObject::from_font(Font {
                    first_char: 0x20,
                    last_char: 0xff,
                    mode: 0x3,
                    flags: 0x4,
                    spacing: 0,
                    line_height: 25,
                    y_offset: 0,
                    extended_spacing: 6,
                    palette: 0x0,
                    glyph_table: 0x0,
                    box_width: 0x0,
                }),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::gfx::owner::gfx;

    #[test]
    fn menu_fonts_keep_the_images_numbers() {
        // The first values are the jade font's places in the glyph store.
        for n in 0..4 {
            let font = gfx().fonts.font_block.menu_fonts[n].get();
            assert_eq!(
                (font.palette, font.glyph_table),
                (
                    crate::gfx::glyph_store::JADE_COLOUR,
                    crate::gfx::glyph_store::JADE
                )
            );
        }
    }
}
