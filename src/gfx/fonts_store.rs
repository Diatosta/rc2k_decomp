//! The game's font objects, their palettes and glyph tables (`gfx::build_fonts`,
//! `gfx::build_game_fonts`, the screens that draw text), out of RAL.EXE's image (phase 5),
//! typed (phase 7): the menus' four fonts and their palette and glyph-table pointers, the
//! palette sheet, and the race's six fonts. A font is the 0x20-byte object `rc2k_core::text::Font`
//! describes, held in a `Shared` cell. Placed in a file of its own beside the graphics
//! module's stores (converted in parallel); its users keep naming the fonts by IDA address
//! through `moved::MOVED` until they are rewritten.
//!
//! `#[repr(C)]` structs with the image's layout, first values the image's (`tests`);
//! `moved::MOVED` points the image's addresses at them (`home`).

use crate::session::{Dword, Shared};
use rc2k_core::text::Font;

/// The fonts, their palettes and glyph tables (IDA 0x51DE30 .. 0x51E384, the image's layout).
#[repr(C, align(16))]
pub struct Fonts {
    /// 0x51DE30: `stru_51DE50` ..: the four fonts the menus and the race's text use (the third is the race's, `race::frame_text`).
    pub menu_fonts: [Shared<Font>; 4],
    /// 0x51DEB0: `g_fontPalettes`: the font palettes (a pointer; `gfx::build_fonts`).
    pub palettes: Dword,
    /// 0x51DEB4: `g_font27Glyphs`: the 27-pixel font's glyph table.
    pub font27_glyphs: Dword,
    /// 0x51DEB8: `g_font23Glyphs`: the 23-pixel font's.
    pub font23_glyphs: Dword,
    /// 0x51DEBC: `g_font21Glyphs`: the 21-pixel font's.
    pub font21_glyphs: Dword,
    /// 0x51DEC0: `g_font13Glyphs`: the 13-pixel font's.
    pub font13_glyphs: Dword,
    /// 0x51DEC4: `g_fontPaletteComponents`: the palette sheet's 256 dwords.
    pub palette_sheet: [Dword; 256],
    /// 0x51E2C4: `unk_51E2C4` ..: the game's six fonts (`gfx::build_game_fonts`): the tyre change's and the name plates' texts.
    pub game_fonts: [Shared<Font>; 6],
}

pub static FONTS: Fonts = Fonts {
    menu_fonts: [
        Shared::new(Font {
            first_char: 0x20,
            last_char: 0x80,
            mode: 0x4,
            flags: 0x0,
            spacing: 1,
            line_height: 9,
            y_offset: 0,
            extended_spacing: 6,
            palette: 0x4f66c4,
            glyph_table: 0x4f52c4,
            box_width: 0x0,
        }),
        Shared::new(Font {
            first_char: 0x20,
            last_char: 0x80,
            mode: 0x0,
            flags: 0x0,
            spacing: 1,
            line_height: 9,
            y_offset: 0,
            extended_spacing: 6,
            palette: 0x4f66c4,
            glyph_table: 0x4f52c4,
            box_width: 0x0,
        }),
        Shared::new(Font {
            first_char: 0x20,
            last_char: 0x80,
            mode: 0x2,
            flags: 0x0,
            spacing: 1,
            line_height: 9,
            y_offset: 0,
            extended_spacing: 6,
            palette: 0x4f66c4,
            glyph_table: 0x4f52c4,
            box_width: 0x0,
        }),
        Shared::new(Font {
            first_char: 0x20,
            last_char: 0x80,
            mode: 0x3,
            flags: 0x0,
            spacing: 1,
            line_height: 9,
            y_offset: 0,
            extended_spacing: 6,
            palette: 0x4f66c4,
            glyph_table: 0x4f52c4,
            box_width: 0x0,
        }),
    ],
    palettes: Dword::new(0x0),
    font27_glyphs: Dword::new(0x0),
    font23_glyphs: Dword::new(0x0),
    font21_glyphs: Dword::new(0x0),
    font13_glyphs: Dword::new(0x0),
    palette_sheet: [const { Dword::new(0x0) }; 256],
    game_fonts: [
        Shared::new(Font {
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
        Shared::new(Font {
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
        Shared::new(Font {
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
        Shared::new(Font {
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
        Shared::new(Font {
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
        Shared::new(Font {
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
};

impl Fonts {
    /// Whether IDA's `ida` is a palette or glyph-table dword of the four menu fonts
    /// (`moved::point_into_homes` passes them over). They name the jade font's colour (0x4F66C4)
    /// and glyphs (0x4F52C4), kept as the image's numbers; `gfx::glyph_store` resolves them
    /// (`jade_store::JadeFont`) where the fonts' palettes and glyph tables are read.
    pub fn holds_handle(ida: usize) -> bool {
        (0..4).any(|n| {
            let font = Self::START + 0x20 * n;
            (font + 0x14..font + 0x1C).contains(&ida)
        })
    }

    pub const START: usize = 0x51DE30;
    pub const LEN: usize = 0x554;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&FONTS as *const Fonts) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fonts_starts_as_the_games() {
        let size = size_of::<Fonts>();
        assert!(Fonts::LEN <= size && size <= (Fonts::LEN).next_multiple_of(16));
        let image = crate::moved::image_bytes(Fonts::START, Fonts::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(Fonts::home()) as *const u8,
                Fonts::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }

    #[test]
    fn menu_fonts_keep_the_images_numbers() {
        // The image's relocated dwords and the first values agree, and name the jade font's block.
        for n in 0..4 {
            let at = Fonts::START + 0x20 * n;
            assert!(Fonts::holds_handle(at + 0x14) && Fonts::holds_handle(at + 0x18));
            assert!(!Fonts::holds_handle(at + 0x10) && !Fonts::holds_handle(at + 0x1C));
            let image = crate::moved::image_bytes(at + 0x14, 8);
            let font = FONTS.menu_fonts[n].get();
            assert_eq!(
                (font.palette.to_le_bytes(), font.glyph_table.to_le_bytes()),
                (
                    image[..4].try_into().unwrap(),
                    image[4..].try_into().unwrap()
                )
            );
            assert_eq!((font.palette, font.glyph_table), (0x4F66C4, 0x4F52C4));
            assert!(crate::moved::home_of_rva(0x4F66C4 - 0x400000).is_some());
            assert!(crate::moved::home_of_rva(0x4F52C4 - 0x400000).is_some());
        }
    }
}
