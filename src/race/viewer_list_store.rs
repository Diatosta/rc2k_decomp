//! The replay viewer's list of buttons: 26 items, each a picture drawn at a place on the screen
//! and, when Enter is pressed on it, the action of the same number. An item holds its kind and
//! the marks the viewer sets on it (bit 6 hidden, bit 7 not usable); its screen rectangle is the
//! one of the same number in `viewer_state_store::RECTANGLES` and its picture is
//! `viewer_state_store::PICTURES[n]`. A key table in the same store says which item each arrow
//! key moves the highlight to.

use crate::memory::session::PlainByte;

/// How many items the list holds.
pub const ITEMS: u32 = 26;

/// One button of the list.
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView, rc2k_derive::ZeroCells)]
pub struct ViewerItem {
    /// The kind (low nibble) and the marks (high bits).
    pub first: PlainByte,
}

impl ViewerItem {
    pub const fn new(first: u8) -> Self {
        Self {
            first: PlainByte::new(first),
        }
    }
}

/// The viewer's list.
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView, rc2k_derive::ZeroCells)]
pub struct ViewerList {
    pub items: [ViewerItem; ITEMS as usize],
    /// Keeps the list's size a multiple of four (a byte view has no padding).
    spare: [PlainByte; 2],
}

impl ViewerList {
    pub const fn new(items: [ViewerItem; ITEMS as usize]) -> Self {
        Self {
            items,
            spare: [const { PlainByte::new(0) }; 2],
        }
    }

    /// Item `n`. The viewer's counts stay within the list; an item past it is refused.
    pub fn item(&self, n: u32) -> &ViewerItem {
        match self.items.get(n as usize) {
            Some(item) => item,
            None => crate::memory::host_hooks::end(
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
