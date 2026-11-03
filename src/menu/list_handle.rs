//! A list menu (`sub_4144CF`'s pages, the replay viewer's items) and its items by name, in
//! place of `list + 0x18 + 0x10 * n` sums on game addresses.
//!
//! A list is a header of 0x18 bytes (+0 flags, +1 the item count, +2 the title's string,
//! +4 the item highlighted, +8 the list's action, +0x10 the neighbours table for
//! `menu::list_step`, +0x14 a key action) and its items of 0x10 bytes from +0x18 (+0 kind in
//! the low nibble and marks in the high, +1 the next list or the number of choices, +2 the
//! text, +4 the item's box, +8 the byte or text it edits or the link's action, +0xC the
//! choices, the edit record or the least value). Every accessor reads or writes inside its
//! own 0x18 header bytes or 0x10 item bytes (`GamePtr::limited`); a place outside every
//! allocation ends the program (`install::installer::refuse`). The words that name data
//! (links, boxes, tables) are read as `installer::rd` does, resolved to where the installer's
//! pages are now.
//!
//! The item count is the list's own and is not checked against the item asked for: the
//! original goes past it too (a list of no items goes round 2^32 times).

use crate::install::installer::{rb, rd, rw, wb, wd};
use crate::pointers::GamePtr;

/// The header's bytes.
const HEADER: u32 = 0x18;
/// An item's bytes.
const ITEM: u32 = 0x10;

/// A list menu (a game address, the header's and items' places taken from it).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListMenu(GamePtr);

impl ListMenu {
    /// The list at a game address (a pointer from the pages' tables).
    pub fn at(addr: u32) -> Self {
        Self(GamePtr::new(addr))
    }

    /// The list at a place in the game's data.
    pub fn of(place: impl Into<crate::place::Place>) -> Self {
        Self(place.into().game_ptr())
    }

    pub fn addr(self) -> u32 {
        self.0.addr()
    }

    /// A header field: `len` bytes at `offset`, inside the header.
    fn head(self, offset: u32, len: u32) -> GamePtr {
        debug_assert!(offset + len <= HEADER);
        self.0.offset(offset).limited(len)
    }

    /// +0: the flags (bit 4 the page stays on Escape, bits 6 and 7 the page hidden).
    pub fn flags(self) -> u8 {
        rb(self.head(0, 1))
    }

    pub fn set_flags(self, flags: u8) {
        wb(self.head(0, 1), flags);
    }

    /// +1: the number of items.
    pub fn count(self) -> u8 {
        rb(self.head(1, 1))
    }

    pub fn set_count(self, count: u8) {
        wb(self.head(1, 1), count);
    }

    /// +4: the item highlighted.
    pub fn highlight(self) -> u32 {
        rd(self.head(4, 4))
    }

    pub fn set_highlight(self, highlight: u32) {
        wd(self.head(4, 4), highlight);
    }

    /// +8: the list's action (an address in RAL.EXE's code: `install::installer::page_action`).
    pub fn action(self) -> u32 {
        rd(self.head(8, 4))
    }

    /// +0x10: the neighbours table for `menu::list_step`.
    pub fn neighbours(self) -> u32 {
        rd(self.head(0x10, 4))
    }

    /// +0x14: the key action (`install::installer::key_action`).
    pub fn key_action(self) -> u32 {
        rd(self.head(0x14, 4))
    }

    /// Item `n`, from +0x18, 0x10 bytes each (wrapping as the original's sum).
    pub fn item(self, n: u32) -> ListItem {
        ListItem(
            self.0
                .offset(HEADER.wrapping_add(ITEM.wrapping_mul(n)))
                .limited(ITEM),
        )
    }

    /// The item whose first byte is `offset` from the list (the pages' tables of marks
    /// give offsets, 0x28 the second item). An offset that is no item's start is refused.
    pub fn item_at_offset(self, offset: u32) -> ListItem {
        let past = offset.wrapping_sub(HEADER);
        if !past.is_multiple_of(ITEM) {
            crate::install::installer::refuse(crate::errors::BadInput(
                "an offset in a list that is no item's start",
            ));
        }
        self.item(past / ITEM)
    }
}

/// One item of a list menu (its 0x10 bytes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListItem(GamePtr);

impl ListItem {
    /// The item at a game address (a pointer from a piece or a page's tables).
    pub fn at(addr: u32) -> Self {
        Self(GamePtr::new(addr).limited(ITEM))
    }

    fn field(self, offset: u32, len: u32) -> GamePtr {
        debug_assert!(offset + len <= ITEM);
        self.0.offset(offset).limited(len)
    }

    /// +0: the kind (low nibble) and marks (high bits), whole.
    pub fn first(self) -> u8 {
        rb(self.field(0, 1))
    }

    pub fn set_first(self, first: u8) {
        wb(self.field(0, 1), first);
    }

    /// The kind: the low nibble of +0.
    pub fn kind(self) -> u8 {
        self.first() & 0xF
    }

    /// +1: the next list (a link) or the number of choices.
    pub fn count(self) -> u8 {
        rb(self.field(1, 1))
    }

    /// +2: the item's text (a string id).
    pub fn text_id(self) -> u16 {
        rw(self.field(2, 2))
    }

    /// +1 of a text item: the kind of its font.
    pub fn font_kind(self) -> u8 {
        self.count()
    }

    /// +4: the box the item's pieces are placed in (0: the title's).
    pub fn box_at(self) -> u32 {
        rd(self.field(4, 4))
    }

    /// +8: the link's or action's address, or where the item's value or text is.
    pub fn target(self) -> u32 {
        rd(self.field(8, 4))
    }

    /// +8 as the byte the item sets (a choice, a value).
    pub fn value(self) -> ItemByte {
        ItemByte(GamePtr::new(self.target()))
    }

    /// +0xC: the pointer to the choices, the edit record or the text action.
    pub fn data(self) -> u32 {
        rd(self.field(0xC, 4))
    }

    /// +0xC as the byte the least value is (a value item).
    pub fn value_least(self) -> u8 {
        rb(self.field(0xC, 1))
    }

    /// +0xD: the most value (a value item).
    pub fn value_most(self) -> u8 {
        rb(self.field(0xD, 1))
    }

    /// +0xC as the item's choices (a choice item).
    pub fn choices(self) -> Choices {
        Choices(GamePtr::new(self.data()))
    }

    /// +0xC as the item's edit record (an edit item).
    pub fn edit_field(self) -> EditField {
        EditField(GamePtr::new(self.data()).limited(8))
    }
}

/// The byte an item sets (its choice or its value).
#[derive(Clone, Copy, Debug)]
pub struct ItemByte(GamePtr);

impl ItemByte {
    pub fn get(self) -> u8 {
        rb(self.0.limited(1))
    }

    pub fn set(self, value: u8) {
        wb(self.0.limited(1), value);
    }
}

/// A choice item's table: two bytes a choice, the second (at +1, +3, ...) its marks. Its
/// length is the item's own count, which the original does not check an index against.
#[derive(Clone, Copy, Debug)]
pub struct Choices(GamePtr);

impl Choices {
    /// Where choice `n`'s marks byte is.
    fn marks_at(self, n: u32) -> GamePtr {
        self.0.offset(n.wrapping_mul(2).wrapping_add(1)).limited(1)
    }

    /// Choice `n`'s byte at +1.
    pub fn marks(self, n: u32) -> u8 {
        rb(self.marks_at(n))
    }

    pub fn set_marks(self, n: u32, marks: u8) {
        wb(self.marks_at(n), marks);
    }

    /// The table's first `len` bytes (a `len` the item's count gives).
    pub fn bytes(self, len: usize, what: &str) -> Result<Vec<u8>, crate::errors::BadInput> {
        self.0.read_bytes(len, what)
    }

    pub fn write_bytes(self, bytes: &[u8], what: &str) -> Result<(), crate::errors::BadInput> {
        self.0.write_bytes(bytes, what)
    }
}

/// An edit item's record (8 bytes): +0 flags, +1 the most characters, +2 the default text,
/// +4 the font's kind, +6 the width.
#[derive(Clone, Copy, Debug)]
pub struct EditField(GamePtr);

impl EditField {
    pub fn flags(self) -> u8 {
        rb(self.0.limited(1))
    }

    pub fn set_flags(self, flags: u8) {
        wb(self.0.limited(1), flags);
    }

    pub fn most(self) -> u8 {
        rb(self.0.offset(1).limited(1))
    }

    pub fn default_text(self) -> u16 {
        rw(self.0.offset(2).limited(2))
    }

    pub fn font_kind(self) -> u8 {
        rb(self.0.offset(4).limited(1))
    }

    pub fn width(self) -> u16 {
        rw(self.0.offset(6).limited(2))
    }
}
