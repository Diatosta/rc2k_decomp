//! A screen's function list by handle (phase 13, 2026-10-28): the five list pointers
//! (`g_unreadList` .. `g_sceneList`, `menu::flow_store`) hold a `FnList`, and `menu::run_list`
//! and the frame callbacks walk its functions. The lists are the typed tables' (`LISTS`,
//! named by `menu::draw_lists_store`); a screen's list may start inside one (at its end
//! mark: the in-game options' `list_758eae.from(2)`), and the menus' last list
//! (`dword_7594AA`, empty) is `FnList::Menus`.

use crate::errors::BadInput;
use rc2k_core::menu::tables::{LISTS, ScreenFn};

/// A function list: its first function's place.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FnList {
    /// `LISTS[list]`'s functions from number `from` on.
    Draw { list: u16, from: u16 },
    /// `dword_7594AA`, the menus' last draw list (no functions).
    Menus,
}

impl FnList {
    /// The list `LISTS[list]` from its function `from` on.
    pub fn draw(list: u16, from: usize) -> FnList {
        assert!(from <= LISTS[list as usize].functions.len());
        FnList::Draw {
            list,
            from: from as u16,
        }
    }

    /// The list's functions from its first on, to the end mark that ends it.
    pub fn functions(self) -> &'static [ScreenFn] {
        match self {
            FnList::Draw { list, from } => &LISTS[list as usize].functions[from as usize..],
            FnList::Menus => &[],
        }
    }

    fn code(self) -> u32 {
        match self {
            FnList::Draw { list, from } => ((list as u32) << 16 | from as u32) + 1,
            FnList::Menus => u32::MAX,
        }
    }
}

/// A list pointer: none (0, as the image starts) or a list.
#[repr(transparent)]
#[derive(rc2k_derive::Cells)]
pub struct ListCell(crate::session::Dword);

impl ListCell {
    pub const fn new() -> Self {
        ListCell(crate::session::Dword::new(0))
    }

    pub fn get(&self) -> Result<FnList, BadInput> {
        match self.0.get() {
            0 => Err(BadInput("a screen's function list not set")),
            u32::MAX => Ok(FnList::Menus),
            n => {
                let n = n - 1;
                Ok(FnList::Draw {
                    list: (n >> 16) as u16,
                    from: n as u16,
                })
            }
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
    use crate::menu::owner::menus;

    #[test]
    fn lists_by_handle() {
        let tables = &menus().tables;
        let l = tables.draw_lists.list_758eae.from(2);
        assert!(l.functions().is_empty());
        assert_eq!(tables.draw_lists.list_758eae.id().functions().len(), 2);
        let c = ListCell::new();
        assert!(c.get().is_err());
        c.set(l);
        assert_eq!(c.get().unwrap(), l);
        c.set(FnList::Menus);
        assert_eq!(c.get().unwrap(), FnList::Menus);
        assert!(FnList::Menus.functions().is_empty());
    }
}
