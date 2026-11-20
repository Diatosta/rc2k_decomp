//! The replay viewer's list of buttons: 26 items, each a picture drawn at a place on the screen
//! and, when Enter is pressed on it, the action of the same number. An item holds its kind and
//! the marks the viewer sets on it (bit 6 hidden, bit 7 not usable), and the names of its screen
//! rectangle and of its picture, which the viewer panel's stores resolve
//! (`ReplayViewBlock::item_place`, `ReplayViewBlock::item_picture`). The list also names the key
//! table that says which item each arrow key moves the highlight to.

use crate::memory::session::{PlainByte, PlainDword};

/// How many items the list holds.
pub const ITEMS: u32 = 26;

/// One button of the list.
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView, rc2k_derive::ZeroCells)]
pub struct ViewerItem {
    /// The kind (low nibble) and the marks (high bits).
    pub first: PlainByte,
    /// Keeps the names that follow aligned to four bytes (a byte view has no padding).
    spare: [PlainByte; 3],
    /// The name of the item's screen rectangle.
    pub place_name: PlainDword,
    /// The name of the item's picture.
    pub picture_name: PlainDword,
}

impl ViewerItem {
    pub const fn new(first: u8, place_name: u32, picture_name: u32) -> Self {
        Self {
            first: PlainByte::new(first),
            spare: [const { PlainByte::new(0) }; 3],
            place_name: PlainDword::new(place_name),
            picture_name: PlainDword::new(picture_name),
        }
    }
}

/// The viewer's list.
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView, rc2k_derive::ZeroCells)]
pub struct ViewerList {
    /// The name of the key table: four bytes an item (what each of the four arrow keys moves
    /// the highlight to, 255 for nowhere).
    pub key_table_name: PlainDword,
    pub items: [ViewerItem; ITEMS as usize],
}

impl ViewerList {
    pub const fn new(key_table_name: u32, items: [ViewerItem; ITEMS as usize]) -> Self {
        Self {
            key_table_name: PlainDword::new(key_table_name),
            items,
        }
    }

    /// Item `n`. The viewer's counts stay within the list; an item past it is refused.
    pub fn item(&self, n: u32) -> &ViewerItem {
        match self.items.get(n as usize) {
            Some(item) => item,
            None => crate::fatal::raise::end(
                "viewer list",
                crate::errors::BadInput("an item past the items of the viewer's list"),
            ),
        }
    }

    /// The action Enter does on item `n`: the one of the same number.
    pub fn action(&self, n: u32) -> u32 {
        self.item(n);
        n
    }
}
