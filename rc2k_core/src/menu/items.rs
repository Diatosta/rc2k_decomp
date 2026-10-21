//! A menu's items: where each lies, and the looks the item actions give them (converted
//! 2026-10-21 from `menu::item_at` 0x471CEF, `menu::disable_item` 0x46B4F6 /
//! `enable_item` 0x46B50F, `menu::hide_in_game_options` 0x46FB63 /
//! `show_in_game_options` 0x46FBE3 and `menu::reset_cars_750724` 0x46FC52, `menu::run_item_action` 0x474683).

use crate::Invalid;
use crate::menu::navigation::CAN_BE_CHOSEN;
use crate::screens::menus::{NOTHING, RIGHT_BUTTON};
use std::ops::Range;

/// An item's bytes; a menu has a 4-byte head before its items.
pub const ITEM_LEN: u32 = 0x5C;
pub const MENU_HEAD_LEN: u32 = 4;
/// An item's +0x48 when the moves up and down pass over it (greyed or hidden).
pub const PASSED_OVER: u32 = 0xFFFF_FC1C;

/// Where item `number` (from 1) of the current menu lies (`menu` gives the menu, asked only
/// for an item it can find). Refused: item 0, for which the original counts through all 32
/// bits.
pub fn item_at(number: u32, menu: impl FnOnce() -> u32) -> Result<u32, Invalid> {
    if number == 0 {
        return Err(Invalid(
            "item number 0: the original counts through all 32 bits",
        ));
    }
    Ok(menu()
        .wrapping_add(MENU_HEAD_LEN)
        .wrapping_add((number - 1).wrapping_mul(ITEM_LEN)))
}

/// The two fonts an item is given (the game's at 0x51E8E4 and 0x51EA04).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    Plain,
    Grey,
}

/// `menu::enable_item` (true) / `disable_item`: the item's +0x48 and its font (+0x18).
pub fn choosable(on: bool) -> (u32, Font) {
    if on {
        (CAN_BE_CHOSEN, Font::Plain)
    } else {
        (PASSED_OVER, Font::Grey)
    }
}

/// An item's string (+0x14), state (+0x48) and what is behind its text (+0x1C).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Look {
    pub string: u32,
    pub state: u32,
    pub behind: u32,
}

/// No string, passed over, nothing behind.
pub const HIDDEN: Look = Look {
    string: 0,
    state: PASSED_OVER,
    behind: NOTHING,
};

/// The in-game options menu's items that can be hidden: 3 .. 6, counted from 0.
pub const IN_GAME_OPTIONS: Range<u32> = 3..7;

/// `menu::hide_in_game_options`: with 1, 2 or 3 the items from 3 + it on are hidden;
/// anything else hides none.
pub fn hidden_in_game_options(from: u32) -> Range<u32> {
    if (1..=3).contains(&from) {
        3 + from..IN_GAME_OPTIONS.end
    } else {
        0..0
    }
}

/// `menu::show_in_game_options`: item `n` (3 .. 6) shown with its string (0x171 on),
/// choosable, a button behind it.
pub fn shown_in_game_option(n: u32) -> Look {
    Look {
        string: 0x171 + n - IN_GAME_OPTIONS.start,
        state: CAN_BE_CHOSEN,
        behind: RIGHT_BUTTON,
    }
}

/// An item's type (+0x10), what `dword_755525` (the highlighted item) holds.
pub const ITEM_TYPE: u32 = 0x10;
/// A menu's items end with this dword.
pub const ITEMS_END: u32 = u32::MAX;
/// The most items `run_item_action` looks through.
pub const MOST_ITEMS: u32 = 256;

/// `menu::run_item_action`'s search: the number (from 0) of the first item whose type
/// (`item_type(n)`) is `highlighted`, or None when the items end first (`word(n)` an
/// item's first dword, the end mark; the first item is not checked for it). Refused:
/// no end in `MOST_ITEMS` items.
pub fn highlighted_item(
    highlighted: u32,
    mut item_type: impl FnMut(u32) -> u32,
    mut word: impl FnMut(u32) -> u32,
) -> Result<Option<u32>, Invalid> {
    for n in 0..MOST_ITEMS {
        if item_type(n) == highlighted {
            return Ok(Some(n));
        }
        if word(n + 1) == ITEMS_END {
            return Ok(None);
        }
    }
    Err(Invalid("a menu without an end"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items() {
        assert!(item_at(0, || unreachable!()).is_err());
        assert_eq!(item_at(1, || 0x1000), Ok(0x1004));
        assert_eq!(
            item_at(3, || 0xFFFF_FF00),
            Ok(0xFFFF_FF00u32.wrapping_add(4 + 0xB8))
        );
        assert_eq!(choosable(false), (0xFFFF_FC1C, Font::Grey));
        assert_eq!(choosable(true), (0xFFFF_FC1B, Font::Plain));
        assert_eq!(hidden_in_game_options(1), 4..7);
        assert_eq!(hidden_in_game_options(3), 6..7);
        assert!(hidden_in_game_options(0).is_empty() && hidden_in_game_options(4).is_empty());
        assert_eq!(
            shown_in_game_option(6),
            Look {
                string: 0x174,
                state: 0xFFFF_FC1B,
                behind: 0xFFFF_FFA7
            }
        );
        assert_eq!(HIDDEN.behind, 0xFFFF_FFA9);
    }

    #[test]
    fn highlighted() {
        let types = [5u32, 7, 9];
        let words = [u32::MAX, 0, 0, u32::MAX];
        let find = |h| highlighted_item(h, |n| types[n as usize], |n| words[n as usize]);
        assert_eq!(find(5), Ok(Some(0)));
        assert_eq!(find(9), Ok(Some(2)));
        assert_eq!(find(1), Ok(None));
        assert!(highlighted_item(1, |_| 0, |_| 0).is_err());
    }
}
