//! The replay viewer's list of buttons: 28 items (the two rows' 26 and the camera buttons of the
//! third and fourth part of a split screen), each a picture drawn at a place on the screen
//! and, when Enter is pressed on it, its action (`rc2k_core::race::viewer::item_action`). An item holds its kind and
//! the marks the viewer sets on it (bit 6 hidden, bit 7 not usable); its screen rectangle is the
//! one of the same number in `viewer_state_store::RECTANGLES` and its picture is
//! `viewer_state_store::PICTURES[n]`. A key table in the same store says which item each arrow
//! key moves the highlight to.

use std::cell::Cell;

/// How many items the list holds.
pub const ITEMS: u32 = rc2k_core::race::viewer::ITEMS;

/// One button of the list.
pub struct ViewerItem {
    /// The kind (low nibble) and the marks (high bits).
    pub first: Cell<u8>,
}

impl ViewerItem {
    pub const fn new(first: u8) -> Self {
        Self {
            first: Cell::new(first),
        }
    }
}

/// The viewer's list.
pub struct ViewerList {
    pub items: [ViewerItem; ITEMS as usize],
}

impl ViewerList {
    pub const fn new(items: [ViewerItem; ITEMS as usize]) -> Self {
        Self { items }
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

    /// The action Enter does on item `n`.
    pub fn action(&self, n: u32) -> u32 {
        self.item(n);
        rc2k_core::race::viewer::item_action(n)
    }
}
