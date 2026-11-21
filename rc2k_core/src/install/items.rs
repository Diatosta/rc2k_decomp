//! The installer's page items under the keys: which item is lit, where the next edit is, what a
//! key does to a page's highlighted item, and what Enter does to a link.

/// Whether the highlighted (unmarked) item is lit: an edit being edited
/// (`editing`: its record's bit 0, `None` for any other kind) always, any other while the clock's
/// bit 3 (frame clock less move clock) is clear.
pub fn lit(editing: Option<bool>, clock: u32) -> bool {
    editing == Some(true) || clock & 8 == 0
}

/// `install::next_edit`: from the highlight on, the first item whose kind is an edit (4): its
/// number and true; else the number the search stopped at and false when `count` (unsigned) is
/// reached (the highlight then goes back to 0). The highlight item itself is looked at whatever
/// its number.
pub fn next_edit(from: u32, count: u32, kind_of: impl Fn(u32) -> u8) -> (u32, bool) {
    let mut choice = from;
    loop {
        if kind_of(choice) & 0xF == 4 {
            return (choice, true);
        }
        choice = choice.wrapping_add(1);
        if count <= choice {
            return (choice, false);
        }
    }
}

/// A key on a page's item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemKey {
    /// 0x66.
    Up,
    /// 0x6B.
    Down,
    /// 0x68.
    Left,
    /// 0x69.
    Right,
    /// 2 .. 0xA.
    Number,
}

/// What a key on a page's highlighted item does (`install::page_key`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageKey {
    /// The arrows and the number keys: the item's helpers.
    Item(ItemKey),
    /// Enter (key 0x1C): the item is entered (`EnterWay`).
    Enter,
    /// Escape (key 1): the page is left with answer 0, unless the list's flags have `STAYS`.
    Escape,
    /// Any other key (a screenshot for `SCREENSHOT_KEY`).
    Other,
}

/// The list's flag that keeps Escape from leaving its page.
pub const STAYS: u8 = 0x10;
/// The key that takes a screenshot (with screenshots on).
pub const SCREENSHOT_KEY: u8 = 0x52;

/// A key's way: `None` for no key or one of 0x80 and above.
pub fn page_key(key: u8) -> Option<PageKey> {
    if key == 0 || key & 0x80 != 0 {
        return None;
    }
    Some(match key {
        0x66 => PageKey::Item(ItemKey::Up),
        0x6B => PageKey::Item(ItemKey::Down),
        0x68 => PageKey::Item(ItemKey::Left),
        0x69 => PageKey::Item(ItemKey::Right),
        2..=0xA => PageKey::Item(ItemKey::Number),
        0x1C => PageKey::Enter,
        1 => PageKey::Escape,
        _ => PageKey::Other,
    })
}

/// What Enter does to an item, by its kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnterWay {
    /// Kind 1: a link (`LinkWay`).
    Link,
    /// Kind 4: an edit begun.
    StartEdit,
    /// Kind 6: an action run.
    Action,
}

impl EnterWay {
    pub fn of(kind: u8) -> Option<EnterWay> {
        match kind {
            1 => Some(EnterWay::Link),
            4 => Some(EnterWay::StartEdit),
            6 => Some(EnterWay::Action),
            _ => None,
        }
    }
}

/// Whether an item's action is there to run: 1 or more.
pub fn has_action(action: u32) -> bool {
    action >= 1
}

/// What Enter on a link does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkWay {
    /// Its action run.
    Action,
    /// The page left for list `n`.
    Leave(u8),
    /// Nothing: its next list is negative.
    Stay,
}

/// A link with `action` and next list `next` (asked only without an action).
pub fn link(action: u32, next: impl FnOnce() -> u8) -> LinkWay {
    if has_action(action) {
        return LinkWay::Action;
    }
    let next = next();
    if next & 0x80 == 0 {
        LinkWay::Leave(next)
    } else {
        LinkWay::Stay
    }
}

/// A key on the folder page (`install::folder_key`): with bit 3 of the key's second byte set (an
/// edit's key), 1 leaves with answer 0 (not while the edit holds `STAYS`), 0x1C with answer 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FolderKey {
    /// Bit 0x800 clear (the folder not being edited): nothing.
    NotEdited,
    /// Escape (key 1).
    Back,
    /// Enter (key 0x1C).
    On,
    /// Any other key: nothing.
    Other,
}

impl FolderKey {
    pub fn of(key: u32) -> FolderKey {
        if key & 0x800 == 0 {
            return FolderKey::NotEdited;
        }
        match key as u8 {
            1 => FolderKey::Back,
            0x1C => FolderKey::On,
            _ => FolderKey::Other,
        }
    }

    /// The answer the page is left with, if it is (`flags` the list's, asked only for
    /// `Back`).
    pub fn answer(self, flags: impl FnOnce() -> u8) -> Option<u8> {
        match self {
            FolderKey::Back => (flags() & STAYS == 0).then_some(0),
            FolderKey::On => Some(1),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_keys() {
        assert_eq!(page_key(0), None);
        assert_eq!(page_key(0x9C), None);
        assert_eq!(page_key(0x66), Some(PageKey::Item(ItemKey::Up)));
        assert_eq!(page_key(0xA), Some(PageKey::Item(ItemKey::Number)));
        assert_eq!(page_key(1), Some(PageKey::Escape));
        assert_eq!(page_key(0x1C), Some(PageKey::Enter));
        assert_eq!(page_key(0x52), Some(PageKey::Other));
        assert_eq!(EnterWay::of(4), Some(EnterWay::StartEdit));
        assert_eq!(EnterWay::of(2), None);
        assert_eq!(link(0x401000, || unreachable!()), LinkWay::Action);
        assert_eq!(link(0, || 3), LinkWay::Leave(3));
        assert_eq!(link(0, || 0xFF), LinkWay::Stay);
        assert_eq!(FolderKey::of(0x01), FolderKey::NotEdited);
        assert_eq!(FolderKey::of(0x801).answer(|| 0), Some(0));
        assert_eq!(FolderKey::of(0x801).answer(|| 0x10), None);
        assert_eq!(FolderKey::of(0x81C).answer(|| unreachable!()), Some(1));
        assert_eq!(FolderKey::of(0x830), FolderKey::Other);
    }

    #[test]
    fn lighting_and_edits() {
        assert!(lit(Some(true), 8));
        assert!(!lit(Some(false), 8));
        assert!(lit(None, 0x17));
        let kinds = [1u8, 2, 0x14, 3];
        assert_eq!(next_edit(0, 4, |n| kinds[n as usize]), (2, true));
        assert_eq!(next_edit(3, 4, |n| kinds[n as usize]), (4, false));
        assert_eq!(next_edit(3, 2, |n| kinds[n as usize]), (4, false));
    }
}
