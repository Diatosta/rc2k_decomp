//! The fonts, 63 font objects of 0x20 bytes (`rc2k_core::strings::text::Font`). Their `palette`s
//! and `glyph_table`s are hung on by `gfx::build_fonts`; the drawing functions take a font by its
//! place (`FontId::addr`). The game's own fonts (the credits' and the race's ..) are a block of
//! `rest`'s. Which font is which is a reading of the callers: a font's `mode` says how it draws (0
//! at x as given, 2 centred, 3 right-aligned or fixed pitch, 4 measuring), its `line_height` the
//! size of the sheet it was built from (10, 21, 23, 27, 13).

use crate::memory::session::{PlainByte, PlainDword};
use rc2k_core::strings::text::Font;

#[cfg(test)]
pub const START: usize = 0x51E3A4;
/// The font objects' number.
pub const COUNT: usize = 63;

/// A font object (0x20 bytes, `rc2k_core::strings::text::Font`), its fields as cells.
#[derive(rc2k_derive::Cells)]
pub struct FontObject {
    /// The character drawn with glyph 0.
    pub first_char: PlainByte,
    /// The last character with a glyph.
    pub last_char: PlainByte,
    /// How it draws (`rc2k_core::strings::text::MODE_*`).
    pub mode: PlainByte,
    /// `FLAG_*` bits.
    pub flags: PlainByte,
    /// Pixels added after every normal glyph.
    pub spacing: PlainDword,
    /// Pixels from one line to the next.
    pub line_height: PlainDword,
    /// Added to y once.
    pub y_offset: PlainDword,
    /// Pixels added after every extended glyph.
    pub extended_spacing: PlainDword,
    /// The palette (a counter in the measuring mode).
    pub palette: PlainDword,
    /// The glyph table.
    pub glyph_table: PlainDword,
    /// The box width lines are aligned in (the widest line's width, measuring).
    pub box_width: PlainDword,
}

impl FontObject {
    const fn new(mode: u8, line_height: u32, palette: u32) -> Self {
        Self {
            first_char: PlainByte::new(32),
            last_char: PlainByte::new(255),
            mode: PlainByte::new(mode),
            flags: PlainByte::new(4),
            spacing: PlainDword::new(1),
            line_height: PlainDword::new(line_height),
            y_offset: PlainDword::new(0),
            extended_spacing: PlainDword::new(6),
            palette: PlainDword::new(palette),
            glyph_table: PlainDword::new(0),
            box_width: PlainDword::new(0),
        }
    }

    /// The object made from a font record.
    pub const fn from_font(font: Font) -> Self {
        Self {
            first_char: PlainByte::new(font.first_char),
            last_char: PlainByte::new(font.last_char),
            mode: PlainByte::new(font.mode),
            flags: PlainByte::new(font.flags),
            spacing: PlainDword::new(font.spacing as u32),
            line_height: PlainDword::new(font.line_height as u32),
            y_offset: PlainDword::new(font.y_offset as u32),
            extended_spacing: PlainDword::new(font.extended_spacing as u32),
            palette: PlainDword::new(font.palette),
            glyph_table: PlainDword::new(font.glyph_table),
            box_width: PlainDword::new(font.box_width),
        }
    }

    pub fn get(&self) -> Font {
        Font {
            first_char: self.first_char.get(),
            last_char: self.last_char.get(),
            mode: self.mode.get(),
            flags: self.flags.get(),
            spacing: self.spacing.get() as i32,
            line_height: self.line_height.get() as i32,
            y_offset: self.y_offset.get() as i32,
            extended_spacing: self.extended_spacing.get() as i32,
            palette: self.palette.get(),
            glyph_table: self.glyph_table.get(),
            box_width: self.box_width.get(),
        }
    }

    pub fn set(&self, font: Font) {
        self.first_char.set(font.first_char);
        self.last_char.set(font.last_char);
        self.mode.set(font.mode);
        self.flags.set(font.flags);
        self.spacing.set(font.spacing as u32);
        self.line_height.set(font.line_height as u32);
        self.y_offset.set(font.y_offset as u32);
        self.extended_spacing.set(font.extended_spacing as u32);
        self.palette.set(font.palette);
        self.glyph_table.set(font.glyph_table);
        self.box_width.set(font.box_width);
    }

    /// The font changed by `change` (read, changed, written back).
    pub fn update(&self, change: impl FnOnce(&mut Font)) {
        let mut font = self.get();
        change(&mut font);
        self.set(font);
    }
}

/// The font objects.
#[derive(rc2k_derive::Cells)]
pub struct FontObjects {
    pub objects: [FontObject; COUNT],
    /// The font a highlighted menu item is drawn with (`text::blend_font`).
    pub blended: FontObject,
}

impl FontObjects {
    /// The font objects (the glyph tables and palettes come with `gfx::build_fonts`; the loading
    /// font's palette is the fonts block's palette sheet from the start).
    pub const fn new() -> Self {
        FontObjects {
            objects: [
                FontObject::new(4, 10, 0),
                FontObject::new(3, 10, 0),
                FontObject::new(0, 10, 0),
                FontObject::new(2, 10, 0),
                FontObject::new(4, 21, 0),
                FontObject::new(3, 21, 0),
                FontObject::new(0, 21, 0),
                FontObject::new(2, 21, 0),
                FontObject::new(3, 21, 0),
                FontObject::new(0, 21, 0),
                FontObject::new(2, 21, 0),
                FontObject::new(3, 21, 0),
                FontObject::new(0, 21, 0),
                FontObject::new(2, 21, 0),
                FontObject::new(3, 21, 0),
                FontObject::new(0, 21, 0),
                FontObject::new(2, 21, 0),
                FontObject::new(3, 21, 0),
                FontObject::new(0, 21, 0),
                FontObject::new(2, 21, 0),
                FontObject::new(4, 23, 0),
                FontObject::new(3, 23, 0),
                FontObject::new(0, 23, 0),
                FontObject::new(2, 23, 0),
                FontObject::new(3, 23, 0),
                FontObject::new(0, 23, 0),
                FontObject::new(2, 23, 0),
                FontObject::new(3, 23, 0),
                FontObject::new(0, 23, 0),
                FontObject::new(2, 23, 0),
                FontObject::new(3, 23, 0),
                FontObject::new(0, 23, 0),
                FontObject::new(2, 23, 0),
                FontObject::new(3, 23, 0),
                FontObject::new(0, 23, 0),
                FontObject::new(2, 23, 0),
                FontObject::new(4, 27, 0),
                FontObject::new(3, 27, 0),
                FontObject::new(0, 27, 0),
                FontObject::new(2, 27, 0),
                FontObject::new(3, 27, 0),
                FontObject::new(0, 27, 0),
                FontObject::new(2, 27, 0),
                FontObject::new(3, 27, 0),
                FontObject::new(0, 27, 0),
                FontObject::new(2, 27, 0),
                FontObject::new(3, 27, 0),
                FontObject::new(0, 27, 0),
                FontObject::new(2, 27, 0),
                FontObject::new(3, 27, 0),
                FontObject::new(0, 27, 0),
                FontObject::new(2, 27, 0),
                FontObject::new(3, 27, 0),
                FontObject::new(0, 27, 0),
                FontObject::new(2, 27, 0),
                FontObject::new(0, 27, 0),
                FontObject::new(2, 27, crate::gfx::glyph_store::SHEET_PLACE),
                FontObject::new(3, 13, 0),
                FontObject::new(0, 13, 0),
                FontObject::new(2, 13, 0),
                FontObject::new(3, 21, 0),
                FontObject::new(0, 21, 0),
                FontObject::new(2, 21, 0),
            ],
            blended: FontObject::from_font(Font {
                first_char: 0x20,
                last_char: 0xFF,
                mode: 0,
                flags: 4,
                spacing: 1,
                line_height: 0x1B,
                y_offset: 0,
                extended_spacing: 6,
                palette: 0,
                glyph_table: 0,
                box_width: 0,
            }),
        }
    }
}

/// A font of the block: its number (the object's place less the first's, in 0x20s).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FontId(usize);

impl FontId {
    /// Font number `number` of the block (a constant past the block fails to compile).
    pub const fn numbered(number: usize) -> Self {
        assert!(number < COUNT);
        Self(number)
    }

    /// Font number `number` of the block, if there is one.
    pub fn try_numbered(number: usize) -> Option<Self> {
        (number < COUNT).then_some(Self(number))
    }

    pub fn object(self, typefaces: &'static FontObjects) -> &'static FontObject {
        &typefaces.objects[self.0]
    }

    /// The font's number in the block, as the screens' core keeps a font in a work variable
    /// (`GameScreen::font_value`).
    pub const fn number(self) -> u32 {
        self.0 as u32
    }

    #[cfg(test)]
    pub const fn ida(self) -> usize {
        START + 0x20 * self.0
    }
}

/// A font the text functions draw and measure with: one of the block's, or the menus' blended font
/// (`FontObjects::blended`). Resolved to the object's cells in one place (`object`).
#[derive(Clone, Copy)]
pub enum FontRef {
    Typeface(FontId),
    /// The menus' blended font.
    Blended,
    /// One of the fonts of `gfx::fonts_store` (the race's: frame rate, tyre change, credits).
    Stored(&'static FontObject),
}

impl From<&'static FontObject> for FontRef {
    fn from(object: &'static FontObject) -> Self {
        FontRef::Stored(object)
    }
}

impl From<FontId> for FontRef {
    fn from(id: FontId) -> Self {
        FontRef::Typeface(id)
    }
}

impl FontRef {
    /// The font object.
    pub fn object(self, typefaces: &'static FontObjects) -> &'static FontObject {
        match self {
            FontRef::Typeface(id) => id.object(typefaces),
            FontRef::Blended => &typefaces.blended,
            FontRef::Stored(object) => object,
        }
    }

    pub fn is_blended(self) -> bool {
        matches!(self, FontRef::Blended)
    }
}

/// A font a table of the game's keeps (a menu item's): the object by number.
#[repr(transparent)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct FontCell(crate::memory::session::Dword);

impl FontCell {
    pub const fn new(font: FontId) -> Self {
        Self(crate::memory::session::Dword::new(font.0 as u32))
    }

    pub fn get(&self) -> FontId {
        FontId(self.0.get() as usize)
    }

    pub fn set(&self, font: FontId) {
        self.0.set(font.0 as u32);
    }
}

/// The fonts the callers name, by number (what each is for is a reading of its callers).
pub mod fonts {
    use super::FontId;

    /// The small number font (the volume settings' captions, the service area).
    pub const SMALL: FontId = FontId::numbered(1);
    /// Labels and descriptions (the service area, the lap time boxes).
    pub const LABEL: FontId = FontId::numbered(2);
    /// The wide notice font (the installer's and the front end's notices).
    pub const NOTICE: FontId = FontId::numbered(3);
    /// The chat lines.
    pub const CHAT_LINE: FontId = FontId::numbered(4);
    /// A marked (the player's) time.
    pub const MARKED_TIME: FontId = FontId::numbered(11);
    /// A marked name.
    pub const MARKED_NAME: FontId = FontId::numbered(12);
    /// A marked place.
    pub const MARKED_RANK: FontId = FontId::numbered(13);
    /// A time; the credits' centred lines.
    pub const TIME: FontId = FontId::numbered(14);
    /// A name, the plain body font; the chat's input line.
    pub const NAME: FontId = FontId::numbered(15);
    /// A place, a label.
    pub const RANK: FontId = FontId::numbered(16);
    /// The stage panel's heading.
    pub const HEADING: FontId = FontId::numbered(17);
    /// The class ranking's title.
    pub const RANKING_TITLE: FontId = FontId::numbered(18);
    /// The measuring font of the 23-high sheet (the lobby, the stage results).
    pub const MEASURE_23: FontId = FontId::numbered(20);
    /// The lit button font (the connections).
    pub const BUTTON_LIT: FontId = FontId::numbered(21);
    /// The lit centred item.
    pub const CENTRED_LIT: FontId = FontId::numbered(23);
    /// The heading font (the credits' second, the service area's, the lobby's).
    pub const HEADING_23: FontId = FontId::numbered(24);
    /// Values (the credits' fourth, the lobby's names).
    pub const VALUES: FontId = FontId::numbered(25);
    /// Labels, totals (the credits' third, the fastest time).
    pub const LABELS: FontId = FontId::numbered(26);
    /// The plain button font.
    pub const BUTTON: FontId = FontId::numbered(27);
    /// The plain centred item.
    pub const CENTRED_PLAIN: FontId = FontId::numbered(29);
    /// The ready marks.
    pub const READY: FontId = FontId::numbered(31);
    /// The dimmed button font.
    pub const BUTTON_DIM: FontId = FontId::numbered(33);
    /// The font the name editors measure with.
    pub const MEASURE: FontId = FontId::numbered(36);
    /// The caption font blended towards (`blend_font`).
    pub const CAPTION_B: FontId = FontId::numbered(37);
    /// The stage choice's second look.
    pub const STAGE_ITEM_B: FontId = FontId::numbered(39);
    /// The arcade button's pulse font.
    pub const ARCADE_BUTTON_PULSE: FontId = FontId::numbered(38);
    /// The arcade button font.
    pub const ARCADE_BUTTON: FontId = FontId::numbered(47);
    /// Titles (the credits' first, the leave question).
    pub const TITLE: FontId = FontId::numbered(40);
    /// The lists (names, settings).
    pub const LIST: FontId = FontId::numbered(41);
    /// The plain item font (back for `enable_item`).
    pub const ITEM_PLAIN: FontId = FontId::numbered(42);
    /// The message box's and installer's text.
    pub const BOX_TEXT: FontId = FontId::numbered(43);
    /// The big heading (the installer's, the lobby's title).
    pub const HEAD: FontId = FontId::numbered(45);
    /// The caption font blended from.
    pub const CAPTION_A: FontId = FontId::numbered(46);
    /// The stage title.
    pub const STAGE_TITLE: FontId = FontId::numbered(48);
    /// The grey item font (`disable_item`).
    pub const ITEM_GREY: FontId = FontId::numbered(51);
    /// The loading font.
    pub const LOADING: FontId = FontId::numbered(56);
    /// The replay viewer's panel fonts.
    pub const PANEL_E: FontId = FontId::numbered(57);
    pub const PANEL_A: FontId = FontId::numbered(58);
    pub const PANEL_C: FontId = FontId::numbered(59);
    /// The sixth panel font (only its palette is set).
    pub const PANEL_F: FontId = FontId::numbered(60);
    pub const PANEL_B: FontId = FontId::numbered(61);
    pub const PANEL_D: FontId = FontId::numbered(62);
}
