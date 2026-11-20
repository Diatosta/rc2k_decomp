//! The installer's page items under the keys. An item of choices (kind 2) has a chosen byte and a
//! table of two bytes a choice, the second's bits 6, 7 its marks; an item of a value (kind 3) has
//! the value's byte and its least and most.

use crate::Invalid;

/// Steps a choice may take before the walk is taken to go round for ever.
pub(crate) const ROUND: usize = 0x202;

/// A choice's marks (bits 6 and 7 of its second byte): when marked, it is passed over.
pub const MARKS: u8 = 0xC0;

/// The choice before `chosen` that is free (`free` is asked of each choice reached), going round
/// from first to last (below 0 wraps to 0xFF; from 0x80 .. 0xFF it goes to the last, `count` - 1,
/// asked only then); `chosen`
/// again when none is. Refused: a chosen byte past the choices with every choice marked.
pub fn choice_back(
    chosen: u8,
    count: impl Fn() -> u8,
    free: impl Fn(u8) -> bool,
) -> Result<u8, Invalid> {
    let mut choice = chosen;
    for _ in 0..ROUND {
        choice = choice.wrapping_sub(1);
        if choice & 0x80 != 0 {
            choice = count().wrapping_sub(1);
        }
        if choice == chosen || free(choice) {
            return Ok(choice);
        }
    }
    Err(Invalid("every choice marked and the chosen one past them"))
}

/// The next free choice after `chosen`, round from the last (`count`, asked at each step)
/// to the first; as `choice_back`.
pub fn choice_on(
    chosen: u8,
    count: impl Fn() -> u8,
    free: impl Fn(u8) -> bool,
) -> Result<u8, Invalid> {
    let mut choice = chosen;
    for _ in 0..ROUND {
        choice = choice.wrapping_add(1);
        if choice >= count() {
            choice = 0;
        }
        if choice == chosen || free(choice) {
            return Ok(choice);
        }
    }
    Err(Invalid("every choice marked and the chosen one past them"))
}

/// A number key's choice (`number`, all 32 bits; its low byte compared with `count`): taken if
/// there is one and it is free.
pub fn choice_by_number(number: u32, count: u8, free: impl Fn() -> bool) -> Option<u8> {
    ((number as u8) < count && free()).then_some(number as u8)
}

/// A value lowered by the key's repeats, not below its least (signed): the new value, all
/// 32 bits (its low byte is stored).
pub fn value_down(value: u8, repeats: u32, least: u8) -> u32 {
    let v = (value as u32).wrapping_sub(repeats);
    if (v as i32) < least as i32 {
        least as u32
    } else {
        v
    }
}

/// A value raised by the key's repeats, not above its most (signed).
pub fn value_up(value: u8, repeats: u32, most: u8) -> u32 {
    let v = (value as u32).wrapping_add(repeats);
    if (v as i32) > most as i32 {
        most as u32
    } else {
        v
    }
}

/// A number key's value: least + (most - least) x `number` / 8, truncated towards 0, stored as a
/// byte. Refused: a quotient past 32 bits.
pub fn value_by_number(least: u8, most: u8, number: u32) -> Result<u8, Invalid> {
    let span = most as i32 - least as i32;
    let quotient = (span as i64 * number as i32 as i64) / 8;
    if quotient != quotient as i32 as i64 {
        return Err(Invalid("a quotient past 32 bits: idiv traps"));
    }
    Ok((quotient as i32 as u32).wrapping_add(least as u32) as u8)
}

/// How many choices are walked for an item's count byte: 0 is 256.
pub fn walked(count: u8) -> usize {
    if count == 0 { 0x100 } else { count as usize }
}

/// `install::mark_page`, an item of choices marked `mark` (0x80, not to be chosen): every unmarked
/// choice of its table (two bytes a choice, `walked` of them) gets the mark too.
pub fn mark_choices(table: &mut [u8], mark: u8) {
    for choice in table.chunks_mut(2) {
        if choice[1] & MARKS == 0 {
            choice[1] |= mark;
        }
    }
}

/// `install::mark_page`, an unmarked item of choices: with at most one free choice, its chosen
/// byte becomes that one (with none, the count less 2) and the item is not to be chosen;
/// `None`: more are free.
pub fn lone_choice(table: &[u8], count: u8) -> Option<u8> {
    let mut free = 0u8;
    let mut last = 2u8;
    let mut position = count;
    for choice in table.chunks(2) {
        if choice[1] & MARKS == 0 {
            free = free.wrapping_add(1);
            last = position;
        }
        position = position.wrapping_sub(1);
    }
    (free <= 1).then(|| last.wrapping_neg().wrapping_add(count))
}

/// `install::mark_page`: whether the highlighted (unmarked) item is lit: an edit being edited
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

/// What a key does to an item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Helper {
    ChoiceBack,
    ChoiceOn,
    ChoiceByNumber,
    ValueDown,
    ValueUp,
    ValueByNumber,
}

impl ItemKey {
    /// The bits of the item's first byte its kind is: 0x2F (choices shown upright are
    /// 0x22), the number keys 0xF.
    pub fn kind_mask(self) -> u8 {
        match self {
            ItemKey::Number => 0xF,
            _ => 0x2F,
        }
    }

    /// The kind its last compare is with when no helper takes the key (0x22 for up and
    /// down, 3 for the others).
    pub fn last_compared(self) -> u8 {
        match self {
            ItemKey::Up | ItemKey::Down => 0x22,
            _ => 3,
        }
    }

    /// The helper for an item of `kind` (masked), if any.
    pub fn helper(self, kind: u8) -> Option<Helper> {
        use Helper::*;
        match (self, kind) {
            (ItemKey::Up, 0x22) | (ItemKey::Left, 2) => Some(ChoiceBack),
            (ItemKey::Down, 0x22) | (ItemKey::Right, 2) => Some(ChoiceOn),
            (ItemKey::Left, 3) => Some(ValueDown),
            (ItemKey::Right, 3) => Some(ValueUp),
            (ItemKey::Number, 2) => Some(ChoiceByNumber),
            (ItemKey::Number, 3) => Some(ValueByNumber),
            _ => None,
        }
    }
}

/// A number key's choice or eighth (`key` less 2, in all 32 bits).
pub fn number_of_key(key: u8) -> u32 {
    (key as u32).wrapping_sub(2)
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

/// Whether an item takes every key itself: an edit (kind 4) being edited (`editing`, its
/// record's bit 0, asked only of an edit).
pub fn takes_every_key(kind: u8, editing: impl FnOnce() -> bool) -> bool {
    kind == 4 && editing()
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
        assert!(takes_every_key(4, || true));
        assert!(!takes_every_key(3, || unreachable!()));
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
    fn choices() {
        let marks = [0u8, 0x40, 0, 0x80];
        let free = |c: u8| marks.get(c as usize).is_some_and(|m| m & MARKS == 0);
        assert_eq!(choice_back(2, || 4, free), Ok(0));
        assert_eq!(choice_back(0, || 4, free), Ok(2));
        assert_eq!(choice_back(0x90, || 4, free), Ok(2));
        assert_eq!(choice_on(2, || 4, free), Ok(0));
        assert_eq!(choice_on(0, || 4, free), Ok(2));
        let none = |_: u8| false;
        assert_eq!(choice_on(1, || 4, none), Ok(1));
        assert!(choice_back(9, || 4, none).is_err());
        assert_eq!(choice_by_number(2, 4, || true), Some(2));
        assert_eq!(choice_by_number(0x102, 4, || true), Some(2));
        assert_eq!(choice_by_number(4, 4, || true), None);
        assert_eq!(choice_by_number(1, 4, || false), None);
    }

    #[test]
    fn values() {
        assert_eq!(value_down(10, 3, 5), 7);
        assert_eq!(value_down(6, 3, 5), 5);
        assert_eq!(value_down(1, 3, 0), 0);
        assert_eq!(value_up(10, 3, 12), 12);
        assert_eq!(value_up(0xFE, 3, 0xFF), 0xFF);
        assert_eq!(value_by_number(10, 90, 4), Ok(50));
        assert_eq!(value_by_number(90, 10, 3), Ok(60));
        assert_eq!(value_by_number(0, 0xFF, 8), Ok(0xFF));
        assert!(value_by_number(0, 0xFF, 0x7FFF_FFFF).is_err());
    }

    #[test]
    fn marking() {
        let mut table = [0u8, 0, 1, 0x40, 2, 0];
        mark_choices(&mut table, 0x80);
        assert_eq!(table, [0, 0x80, 1, 0x40, 2, 0x80]);
        assert_eq!(lone_choice(&[0, 0x40, 1, 0, 2, 0x80], 3), Some(1));
        assert_eq!(lone_choice(&[0, 0x40, 1, 0x80], 2), Some(0));
        assert_eq!(lone_choice(&[0, 0, 1, 0], 2), None);
        assert_eq!(walked(0), 0x100);
        assert!(lit(Some(true), 8));
        assert!(!lit(Some(false), 8));
        assert!(lit(None, 0x17));
        let kinds = [1u8, 2, 0x14, 3];
        assert_eq!(next_edit(0, 4, |n| kinds[n as usize]), (2, true));
        assert_eq!(next_edit(3, 4, |n| kinds[n as usize]), (4, false));
        assert_eq!(next_edit(3, 2, |n| kinds[n as usize]), (4, false));
    }

    #[test]
    fn keys() {
        assert_eq!(ItemKey::Up.helper(0x22), Some(Helper::ChoiceBack));
        assert_eq!(ItemKey::Up.helper(2), None);
        assert_eq!(ItemKey::Left.helper(3), Some(Helper::ValueDown));
        assert_eq!(ItemKey::Number.helper(3), Some(Helper::ValueByNumber));
        assert_eq!(ItemKey::Number.kind_mask(), 0xF);
        assert_eq!(ItemKey::Down.last_compared(), 0x22);
        assert_eq!(number_of_key(1), u32::MAX);
    }
}
