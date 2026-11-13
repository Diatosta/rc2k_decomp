//! The fonts, out of RAL.EXE's image:
//! IDA 0x51E3A4 .. 0x51EB84, 63 font objects of 0x20 bytes (`rc2k_core::text::Font`). Their
//! palettes (+0x14) and glyph tables (+0x18) are hung on by `gfx::build_fonts`; the
//! drawing functions take a font by its place (`FontId::addr`). The game's own fonts (the
//! credits' and the race's, 0x51E2C4 ..) are a block of `rest`'s.
//!
//! Which font is which is a reading of the callers: a font's mode (+2) says how it draws
//! (0 at x as given, 2 centred, 3 right-aligned or fixed pitch, 4 measuring), its line
//! height (+8) the size of the sheet it was built from (10, 21, 23, 27, 13).

use crate::session::{PlainByte, PlainDword};
use rc2k_core::text::Font;

/// IDA's address of the block's first byte.
#[cfg(test)]
pub const START: usize = 0x51E3A4;
/// The font objects' number.
pub const COUNT: usize = 63;

/// A font object (0x20 bytes, `rc2k_core::text::Font`), its fields as cells.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct FontObject {
    /// +0x00: the character drawn with glyph 0.
    pub first_char: PlainByte,
    /// +0x01: the last character with a glyph.
    pub last_char: PlainByte,
    /// +0x02: how it draws (`rc2k_core::text::MODE_*`).
    pub mode: PlainByte,
    /// +0x03: `FLAG_*` bits.
    pub flags: PlainByte,
    /// +0x04: pixels added after every normal glyph.
    pub spacing: PlainDword,
    /// +0x08: pixels from one line to the next.
    pub line_height: PlainDword,
    /// +0x0C: added to y once.
    pub y_offset: PlainDword,
    /// +0x10: pixels added after every extended glyph.
    pub extended_spacing: PlainDword,
    /// +0x14: the palette (a counter in the measuring mode).
    pub palette: PlainDword,
    /// +0x18: the glyph table.
    pub glyph_table: PlainDword,
    /// +0x1C: the box width lines are aligned in (the widest line's width, measuring).
    pub box_width: PlainDword,
}

const _: () = assert!(size_of::<FontObject>() == size_of::<Font>());

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

    /// A copy of the font (its fields may be read half changed, as the original's).
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
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct FontObjects {
    pub objects: [FontObject; COUNT],
}

impl FontObjects {
    /// The font objects; the first values are the image's (the glyph tables and palettes come
    /// with `gfx::build_fonts`; the loading font's palette is the image's `FontBlock.palette_sheet[0]`, made a
    /// home at start).
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
                FontObject::new(2, 27, 0x51dec4),
                FontObject::new(3, 13, 0),
                FontObject::new(0, 13, 0),
                FontObject::new(2, 13, 0),
                FontObject::new(3, 21, 0),
                FontObject::new(0, 21, 0),
                FontObject::new(2, 21, 0),
            ],
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

    /// The font's number in the block, as the screens' core keeps a font in a work
    /// variable (`GameScreen::font_value`).
    pub const fn number(self) -> u32 {
        self.0 as u32
    }

    /// IDA's address of the object (what the image's tables held).
    #[cfg(test)]
    pub const fn ida(self) -> usize {
        START + 0x20 * self.0
    }
}

/// What the screens' core keeps for the menus' blended font: one past the block's numbers.
pub const BLENDED_NUMBER: u32 = COUNT as u32;

/// A font the text functions draw and measure with: one of the block's, or the menus' blended
/// font (`text::blend_font::BLENDED_FONT`). Resolved to the object's cells in one place
/// (`object`).
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
            FontRef::Blended => crate::text::blend_font::BLENDED_FONT.get(),
            FontRef::Stored(object) => object,
        }
    }

    pub fn is_blended(self) -> bool {
        matches!(self, FontRef::Blended)
    }
}

/// A font a table of the game's keeps (a menu item's): the object by number, where the
/// original kept its address.
#[repr(transparent)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct FontCell(crate::session::Dword);

impl FontCell {
    pub const fn new(font: FontId) -> Self {
        Self(crate::session::Dword::new(font.0 as u32))
    }

    pub fn get(&self) -> FontId {
        FontId(self.0.get() as usize)
    }

    pub fn set(&self, font: FontId) {
        self.0.set(font.0 as u32);
    }
}

/// The fonts the callers name (`FontObjects.objects[1].first_char` and so on, by number, with IDA's address of each; the readings of
/// what each is for are the callers').
pub mod fonts {
    use super::FontId;

    /// `icon_palette`: the small number font (the volume settings' captions, the service area).
    pub const SMALL: FontId = FontId::numbered(1); // IDA 0x51E3C4
    /// `FontObjects.objects[2].first_char`: labels and descriptions (the service area, the lap time boxes).
    pub const LABEL: FontId = FontId::numbered(2); // IDA 0x51E3E4
    /// `FontObjects.objects[3].first_char`: the wide notice font (the installer's and the front end's notices).
    pub const NOTICE: FontId = FontId::numbered(3); // IDA 0x51E404
    /// `FontObjects.objects[4].first_char`: the chat lines.
    pub const CHAT_LINE: FontId = FontId::numbered(4); // IDA 0x51E424
    /// `FontObjects.objects[11].first_char`: a marked (the player's) time.
    pub const MARKED_TIME: FontId = FontId::numbered(11); // IDA 0x51E504
    /// `FontObjects.objects[12].first_char`: a marked name.
    pub const MARKED_NAME: FontId = FontId::numbered(12); // IDA 0x51E524
    /// `FontObjects.objects[13].first_char`: a marked place.
    pub const MARKED_RANK: FontId = FontId::numbered(13); // IDA 0x51E544
    /// `FontObjects.objects[14].first_char`: a time; the credits' centred lines.
    pub const TIME: FontId = FontId::numbered(14); // IDA 0x51E564
    /// `FontObjects.objects[15].first_char`: a name, the plain body font; the chat's input line.
    pub const NAME: FontId = FontId::numbered(15); // IDA 0x51E584
    /// `FontObjects.objects[16].first_char`: a place, a label.
    pub const RANK: FontId = FontId::numbered(16); // IDA 0x51E5A4
    /// `FontObjects.objects[17].first_char`: the stage panel's heading.
    pub const HEADING: FontId = FontId::numbered(17); // IDA 0x51E5C4
    /// `FontObjects.objects[18].first_char`: the class ranking's title.
    pub const RANKING_TITLE: FontId = FontId::numbered(18); // IDA 0x51E5E4
    /// `palette`: the measuring font of the 23-high sheet (the lobby, the stage results).
    pub const MEASURE_23: FontId = FontId::numbered(20); // IDA 0x51E624
    /// `FontObjects.objects[21].first_char`: the lit button font (the connections).
    pub const BUTTON_LIT: FontId = FontId::numbered(21); // IDA 0x51E644
    /// `FontObjects.objects[23].first_char`: the lit centred item.
    pub const CENTRED_LIT: FontId = FontId::numbered(23); // IDA 0x51E684
    /// `font`: the heading font (the credits' second, the service area's, the lobby's).
    pub const HEADING_23: FontId = FontId::numbered(24); // IDA 0x51E6A4
    /// `FontObjects.objects[25].first_char`: values (the credits' fourth, the lobby's names).
    pub const VALUES: FontId = FontId::numbered(25); // IDA 0x51E6C4
    /// `FontObjects.objects[26].first_char`: labels, totals (the credits' third, the fastest time).
    pub const LABELS: FontId = FontId::numbered(26); // IDA 0x51E6E4
    /// `FontObjects.objects[27].first_char`: the plain button font.
    pub const BUTTON: FontId = FontId::numbered(27); // IDA 0x51E704
    /// `FontObjects.objects[29].first_char`: the plain centred item.
    pub const CENTRED_PLAIN: FontId = FontId::numbered(29); // IDA 0x51E744
    /// `FontObjects.objects[31].first_char`: the ready marks.
    pub const READY: FontId = FontId::numbered(31); // IDA 0x51E784
    /// `FontObjects.objects[33].first_char`: the dimmed button font.
    pub const BUTTON_DIM: FontId = FontId::numbered(33); // IDA 0x51E7C4
    /// `FontObjects.objects[36].first_char`: the font the name editors measure with.
    pub const MEASURE: FontId = FontId::numbered(36); // IDA 0x51E824
    /// `font_b`: the caption font blended towards (`blend_font`).
    pub const CAPTION_B: FontId = FontId::numbered(37); // IDA 0x51E844
    /// `FontObjects.objects[39].first_char`: the stage choice's second look.
    pub const STAGE_ITEM_B: FontId = FontId::numbered(39); // IDA 0x51E884
    /// `unk_51E864`: the arcade button's pulse font.
    pub const ARCADE_BUTTON_PULSE: FontId = FontId::numbered(38); // IDA 0x51E864
    /// `unk_51E984`: the arcade button font.
    pub const ARCADE_BUTTON: FontId = FontId::numbered(47); // IDA 0x51E984
    /// `FontObjects.objects[40].first_char`: titles (the credits' first, the leave question).
    pub const TITLE: FontId = FontId::numbered(40); // IDA 0x51E8A4
    /// `FontObjects.objects[41].first_char`: the lists (names, settings).
    pub const LIST: FontId = FontId::numbered(41); // IDA 0x51E8C4
    /// `FontObjects.objects[42].first_char`: the plain item font (back for `enable_item`).
    pub const ITEM_PLAIN: FontId = FontId::numbered(42); // IDA 0x51E8E4
    /// `FontObjects.objects[43].first_char`: the message box's and installer's text.
    pub const BOX_TEXT: FontId = FontId::numbered(43); // IDA 0x51E904
    /// `FontObjects.objects[45].first_char`: the big heading (the installer's, the lobby's title).
    pub const HEAD: FontId = FontId::numbered(45); // IDA 0x51E944
    /// `FontObjects.objects[46].first_char`: the caption font blended from.
    pub const CAPTION_A: FontId = FontId::numbered(46); // IDA 0x51E964
    /// `FontObjects.objects[48].first_char`: the stage title.
    pub const STAGE_TITLE: FontId = FontId::numbered(48); // IDA 0x51E9A4
    /// `FontObjects.objects[51].first_char`: the grey item font (`disable_item`).
    pub const ITEM_GREY: FontId = FontId::numbered(51); // IDA 0x51EA04
    /// `FontObjects.objects[56].first_char`: the loading font.
    pub const LOADING: FontId = FontId::numbered(56); // IDA 0x51EAA4
    /// `FontObjects.objects[57].first_char` .. `FontObjects.objects[62].first_char`: the replay viewer's panel fonts.
    pub const PANEL_E: FontId = FontId::numbered(57); // IDA 0x51EAC4
    pub const PANEL_A: FontId = FontId::numbered(58); // IDA 0x51EAE4
    pub const PANEL_C: FontId = FontId::numbered(59); // IDA 0x51EB04
    /// `FontObjects.objects[60].first_char`: the sixth panel font (only its palette is set).
    pub const PANEL_F: FontId = FontId::numbered(60); // IDA 0x51EB24
    pub const PANEL_B: FontId = FontId::numbered(61); // IDA 0x51EB44
    pub const PANEL_D: FontId = FontId::numbered(62); // IDA 0x51EB64
}
