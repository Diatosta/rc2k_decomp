//! A screen's function list by handle (phase 13, 2026-10-28): the five list pointers
//! (`g_unreadList` .. `g_sceneList`, `menu::flow_store`) held the lists' addresses; they
//! hold a `FnList` now, and `menu::run_list` and the frame callbacks walk its entries.
//! The lists are the image's (`menu::draw_lists_store`, which share tails: a list may start
//! inside another) and the menus' own (`dword_7594AA`, `menu::lists_store`).

use crate::errors::BadInput;
use crate::menu::draw_lists_store::{DRAW_LISTS, DrawLists};
use crate::menu::lists_store::MENU_LISTS;
use crate::session::Dword;

/// A function list: its first entry's place.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FnList {
    /// The draw lists' dword `n` on.
    Draw(u16),
    /// `dword_7594AA`, the menus' last draw list.
    Menus,
}

/// The draw lists' dwords in a row.
fn draw_dwords() -> &'static [Dword] {
    const N: usize = size_of::<DrawLists>() / size_of::<Dword>();
    const _: () = assert!(size_of::<DrawLists>().is_multiple_of(size_of::<Dword>()));
    // SAFETY: `DrawLists` is `#[repr(C)]` of `DrawList<N>`s, each `#[repr(C)]` of one
    // `[Dword; N]`: dwords in a row, no padding (size checked above), a static.
    unsafe { std::slice::from_raw_parts((&raw const DRAW_LISTS).cast::<Dword>(), N) }
}

impl FnList {
    /// The list's entries from its first on, to the end of the table it is in (the -1 that
    /// ends it is among them).
    pub fn entries(self) -> &'static [Dword] {
        match self {
            FnList::Draw(n) => &draw_dwords()[n as usize..],
            FnList::Menus => &MENU_LISTS.draw_list,
        }
    }

    /// The list `entry` starts: an entry of the draw lists' (`DrawList::from`).
    pub fn starting_at(entry: &'static Dword) -> FnList {
        let all = draw_dwords();
        let n =
            (std::ptr::from_ref::<Dword>(entry) as usize).wrapping_sub(all.as_ptr() as usize) / 4;
        assert!(n < all.len(), "a draw list's entry outside the draw lists");
        FnList::Draw(n as u16)
    }

    fn code(self) -> u32 {
        match self {
            FnList::Draw(n) => n as u32 + 1,
            FnList::Menus => u32::MAX,
        }
    }
}

/// A list pointer: none (0, as the image starts) or a list.
#[repr(transparent)]
pub struct ListCell(Dword);

impl ListCell {
    pub const fn new() -> Self {
        ListCell(Dword::new(0))
    }

    pub fn get(&self) -> Result<FnList, BadInput> {
        match self.0.get() {
            0 => Err(BadInput("a screen's function list not set")),
            u32::MAX => Ok(FnList::Menus),
            n => Ok(FnList::Draw((n - 1) as u16)),
        }
    }

    pub fn set(&self, list: FnList) {
        self.0.set(list.code());
    }
}

impl Default for ListCell {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_by_handle() {
        let l = DRAW_LISTS.list_758eae.from(2);
        assert_eq!(
            l.entries()[0].get(),
            DRAW_LISTS.list_758eae.entries[2].get()
        );
        let c = ListCell::new();
        assert!(c.get().is_err());
        c.set(l);
        assert_eq!(c.get().unwrap(), l);
        c.set(FnList::Menus);
        assert_eq!(c.get().unwrap(), FnList::Menus);
        assert_eq!(DRAW_LISTS.unread_758b72.id(), FnList::Draw(0));
    }
}
