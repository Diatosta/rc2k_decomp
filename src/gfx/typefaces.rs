//! The fonts, out of RAL.EXE's image (phase 5; typed in phase 7, `notes/conversion.md`):
//! IDA 0x51E3A4 .. 0x51EB84, 63 font objects of 0x20 bytes (`rc2k_core::text::Font`). Their
//! palettes (+0x14) and glyph tables (+0x18) are hung on by `gfx::build_fonts`; the
//! drawing functions take a font by its place (`FontId::addr`). The game's own fonts (the
//! credits' and the race's, 0x51E2C4 ..) are a block of `rest`'s.
//!
//! Which font is which is a reading of the callers: a font's mode (+2) says how it draws
//! (0 at x as given, 2 centred, 3 right-aligned or fixed pitch, 4 measuring), its line
//! height (+8) the size of the sheet it was built from (10, 21, 23, 27, 13).

use crate::session::{Byte, Dword};
use rc2k_core::text::Font;

/// IDA's address of the block's first byte.
pub const START: usize = 0x51E3A4;
/// The font objects' number.
pub const COUNT: usize = 63;
/// The block's length.
pub const LEN: usize = COUNT * 0x20;

/// A font object (0x20 bytes, `rc2k_core::text::Font`), its fields as cells.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct FontObject {
    /// +0x00: the character drawn with glyph 0.
    pub first_char: Byte,
    /// +0x01: the last character with a glyph.
    pub last_char: Byte,
    /// +0x02: how it draws (`rc2k_core::text::MODE_*`).
    pub mode: Byte,
    /// +0x03: `FLAG_*` bits.
    pub flags: Byte,
    /// +0x04: pixels added after every normal glyph.
    pub spacing: Dword,
    /// +0x08: pixels from one line to the next.
    pub line_height: Dword,
    /// +0x0C: added to y once.
    pub y_offset: Dword,
    /// +0x10: pixels added after every extended glyph.
    pub extended_spacing: Dword,
    /// +0x14: the palette (a counter in the measuring mode).
    pub palette: Dword,
    /// +0x18: the glyph table.
    pub glyph_table: Dword,
    /// +0x1C: the box width lines are aligned in (the widest line's width, measuring).
    pub box_width: Dword,
}

const _: () = assert!(size_of::<FontObject>() == size_of::<Font>());

impl FontObject {
    const fn new(mode: u8, line_height: u32, palette: u32) -> Self {
        Self {
            first_char: Byte::new(32),
            last_char: Byte::new(255),
            mode: Byte::new(mode),
            flags: Byte::new(4),
            spacing: Dword::new(1),
            line_height: Dword::new(line_height),
            y_offset: Dword::new(0),
            extended_spacing: Dword::new(6),
            palette: Dword::new(palette),
            glyph_table: Dword::new(0),
            box_width: Dword::new(0),
        }
    }

    /// The object made from a font record.
    pub const fn from_font(font: Font) -> Self {
        Self {
            first_char: Byte::new(font.first_char),
            last_char: Byte::new(font.last_char),
            mode: Byte::new(font.mode),
            flags: Byte::new(font.flags),
            spacing: Dword::new(font.spacing as u32),
            line_height: Dword::new(font.line_height as u32),
            y_offset: Dword::new(font.y_offset as u32),
            extended_spacing: Dword::new(font.extended_spacing as u32),
            palette: Dword::new(font.palette),
            glyph_table: Dword::new(font.glyph_table),
            box_width: Dword::new(font.box_width),
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

    /// Where the object is: what `draw_text` and the others take.
    pub fn addr(&self) -> u32 {
        crate::pointers::game_address(std::ptr::from_ref::<Self>(self))
    }
}

/// The font objects.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Fonts {
    pub objects: [FontObject; COUNT],
}

/// The font objects; the first values are the image's (the glyph tables and palettes come
/// with `gfx::build_fonts`; the loading font's palette is the image's `unk_51DEC4`, made a
/// home at start).
pub static FONTS: Fonts = Fonts {
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
};

#[cfg(test)]
/// Where the block is (`moved::MOVED`'s home for IDA 0x51E3A4).
pub fn home() -> usize {
    crate::pointers::game_address(&raw const FONTS) as usize
}

/// A font of the block: its number (the object's place less the first's, in 0x20s).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FontId(usize);

impl FontId {
    /// The font at IDA's address `ida` of the block (a constant that is not one fails to
    /// compile).
    pub const fn at(ida: usize) -> Self {
        assert!(START <= ida && ida < START + LEN && (ida - START).is_multiple_of(0x20));
        Self((ida - START) / 0x20)
    }

    /// The font at IDA's address `ida`, if it is one of the block's (a table that keeps
    /// the image's addresses as names resolves them here).
    pub fn try_at(ida: usize) -> Option<Self> {
        (START..START + LEN)
            .contains(&ida)
            .then(|| ida - START)
            .filter(|off| off.is_multiple_of(0x20))
            .map(|off| Self(off / 0x20))
    }

    pub fn object(self) -> &'static FontObject {
        &FONTS.objects[self.0]
    }

    /// Where the object is.
    pub fn addr(self) -> u32 {
        self.object().addr()
    }

    /// Where the object is, as a checked game pointer.
    pub fn game_ptr(self) -> crate::pointers::GamePtr {
        crate::pointers::GamePtr::to(self.object())
    }

    /// IDA's address of the object (what the image's tables held).
    #[cfg(test)]
    pub const fn ida(self) -> usize {
        START + 0x20 * self.0
    }
}

/// A font a table of the game's keeps (a menu item's): the object by number, where the
/// original kept its address (phase 13, handles: `moved::point_into_homes` passes these
/// places over).
#[repr(transparent)]
#[derive(rc2k_derive::Cells)]
pub struct FontCell(crate::session::Dword);

impl FontCell {
    /// The font at IDA's address `ida` (a constant that is not one fails to compile).
    pub const fn at(ida: u32) -> Self {
        Self(crate::session::Dword::new(
            FontId::at(ida as usize).0 as u32,
        ))
    }

    pub fn get(&self) -> FontId {
        FontId(self.0.get() as usize)
    }

    pub fn set(&self, font: FontId) {
        self.0.set(font.0 as u32);
    }
}

/// The fonts the callers name (`stru_51E3C4` and so on, by IDA's address; the readings of
/// what each is for are the callers').
pub mod fonts {
    use super::FontId;

    /// `icon_palette`: the small number font (the volume settings' captions, the service area).
    pub const SMALL: FontId = FontId::at(0x51E3C4);
    /// `stru_51E3E4`: labels and descriptions (the service area, the lap time boxes).
    pub const LABEL: FontId = FontId::at(0x51E3E4);
    /// `stru_51E404`: the wide notice font (the installer's and the front end's notices).
    pub const NOTICE: FontId = FontId::at(0x51E404);
    /// `dword_51E424`: the chat lines.
    pub const CHAT_LINE: FontId = FontId::at(0x51E424);
    /// `stru_51E504`: a marked (the player's) time.
    pub const MARKED_TIME: FontId = FontId::at(0x51E504);
    /// `stru_51E524`: a marked name.
    pub const MARKED_NAME: FontId = FontId::at(0x51E524);
    /// `stru_51E544`: a marked place.
    pub const MARKED_RANK: FontId = FontId::at(0x51E544);
    /// `stru_51E564`: a time; the credits' centred lines.
    pub const TIME: FontId = FontId::at(0x51E564);
    /// `stru_51E584`: a name, the plain body font; the chat's input line.
    pub const NAME: FontId = FontId::at(0x51E584);
    /// `stru_51E5A4`: a place, a label.
    pub const RANK: FontId = FontId::at(0x51E5A4);
    /// `stru_51E5C4`: the stage panel's heading.
    pub const HEADING: FontId = FontId::at(0x51E5C4);
    /// `stru_51E5E4`: the class ranking's title.
    pub const RANKING_TITLE: FontId = FontId::at(0x51E5E4);
    /// `palette`: the measuring font of the 23-high sheet (the lobby, the stage results).
    pub const MEASURE_23: FontId = FontId::at(0x51E624);
    /// `unk_51E644`: the lit button font (the connections).
    pub const BUTTON_LIT: FontId = FontId::at(0x51E644);
    /// `stru_51E684`: the lit centred item.
    pub const CENTRED_LIT: FontId = FontId::at(0x51E684);
    /// `font`: the heading font (the credits' second, the service area's, the lobby's).
    pub const HEADING_23: FontId = FontId::at(0x51E6A4);
    /// `stru_51E6C4`: values (the credits' fourth, the lobby's names).
    pub const VALUES: FontId = FontId::at(0x51E6C4);
    /// `stru_51E6E4`: labels, totals (the credits' third, the fastest time).
    pub const LABELS: FontId = FontId::at(0x51E6E4);
    /// `unk_51E704`: the plain button font.
    pub const BUTTON: FontId = FontId::at(0x51E704);
    /// `unk_51E744`: the plain centred item.
    pub const CENTRED_PLAIN: FontId = FontId::at(0x51E744);
    /// `stru_51E784`: the ready marks.
    pub const READY: FontId = FontId::at(0x51E784);
    /// `stru_51E7C4`: the dimmed button font.
    pub const BUTTON_DIM: FontId = FontId::at(0x51E7C4);
    /// `dword_51E824`: the font the name editors measure with.
    pub const MEASURE: FontId = FontId::at(0x51E824);
    /// `font_b`: the caption font blended towards (`blend_font`).
    pub const CAPTION_B: FontId = FontId::at(0x51E844);
    /// `stru_51E884`: the stage choice's second look.
    pub const STAGE_ITEM_B: FontId = FontId::at(0x51E884);
    /// `stru_51E8A4`: titles (the credits' first, the leave question).
    pub const TITLE: FontId = FontId::at(0x51E8A4);
    /// `stru_51E8C4`: the lists (names, settings).
    pub const LIST: FontId = FontId::at(0x51E8C4);
    /// `stru_51E8E4`: the plain item font (back for `enable_item`).
    pub const ITEM_PLAIN: FontId = FontId::at(0x51E8E4);
    /// `stru_51E904`: the message box's and installer's text.
    pub const BOX_TEXT: FontId = FontId::at(0x51E904);
    /// `stru_51E944`: the big heading (the installer's, the lobby's title).
    pub const HEAD: FontId = FontId::at(0x51E944);
    /// `stru_51E964`: the caption font blended from.
    pub const CAPTION_A: FontId = FontId::at(0x51E964);
    /// `stru_51E9A4`: the stage title.
    pub const STAGE_TITLE: FontId = FontId::at(0x51E9A4);
    /// `unk_51EA04`: the grey item font (`disable_item`).
    pub const ITEM_GREY: FontId = FontId::at(0x51EA04);
    /// `stru_51EAA4`: the loading font.
    pub const LOADING: FontId = FontId::at(0x51EAA4);
    /// `unk_51EAC4` .. `stru_51EB64`: the replay viewer's panel fonts.
    pub const PANEL_E: FontId = FontId::at(0x51EAC4);
    pub const PANEL_A: FontId = FontId::at(0x51EAE4);
    pub const PANEL_C: FontId = FontId::at(0x51EB04);
    /// `unk_51EB24`: the sixth panel font (only its palette is set).
    pub const PANEL_F: FontId = FontId::at(0x51EB24);
    pub const PANEL_B: FontId = FontId::at(0x51EB44);
    pub const PANEL_D: FontId = FontId::at(0x51EB64);
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&FONTS, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(offset_of!(FontObject, palette), 0x14);
        assert_eq!(offset_of!(FontObject, glyph_table), 0x18);
        assert_eq!(size_of::<FontObject>(), 0x20);
        assert_eq!(crate::moved::place(START), home());
        assert_eq!(
            crate::moved::place(0x51EAB8),
            FONTS.objects[56].palette.addr() as usize
        );
        assert_eq!(fonts::MEASURE.addr() as usize, home() + 0x51E824 - START);
    }
}
