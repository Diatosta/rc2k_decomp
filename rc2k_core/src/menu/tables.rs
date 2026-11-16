//! The game's menus and its screens' draw lists as Rust data: functions are the enums `ItemFn` and
//! `ScreenFn`, fonts are `MenuFont`. The data is in `tables_data.rs`.
//!
//! These tables are the one source of the menus and the lists: the game's crate keeps, per item,
//! only the fields that change in play (string id, font, `behind`, sprite, skip mark and highlight
//! font: `menu::menus_store::MenuItem`, starting as the table has them) and names a menu or list by
//! its number here.

pub use super::item_params::{EventStart, OptionSwitch, StageCounter, VolumeKind};
pub use super::tables_data::{ItemFn, LISTS, MENUS, MenuFont, ScreenFn};

/// A menu: its labels (texts that are not items) and its items.
#[derive(Debug)]
pub struct Menu {
    /// Whether anything names it (eight menus follow another's end mark and nothing refers
    /// to them).
    pub referenced: bool,
    /// Which of the label lists holds its labels as they change in play (None: no labels).
    pub label_list: Option<u8>,
    pub labels: &'static [MenuLabel],
    pub items: &'static [MenuItem],
}

/// A label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuLabel {
    pub x: i32,
    pub y: i32,
    pub font: MenuFont,
    pub string: u32,
}

/// What is drawn behind an item: a sprite by its number (and its palette), or for a button
/// one of the sprite lists (the button's look in each state).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ItemSprite {
    Sprite(u32),
    List(usize),
}

/// An item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuItem {
    pub x: i32,
    pub y: i32,
    pub width_or_text_x: i32,
    pub alignment_or_text_y: i32,
    /// The number that highlights and links name the item by.
    pub id: u32,
    /// The string id as the game starts; changed in play.
    pub string: u32,
    /// The font as the game starts; changed in play (`menu::disable_item` / `enable_item`).
    pub font: MenuFont,
    /// The item's backdrop as the game starts; changed in play.
    pub behind: u32,
    pub sprites: ItemSprite,
    /// The items up, down, left and right of this one, by `id`.
    pub links: [u32; 4],
    /// What a key does on it: up, down, left, right.
    pub on_key: [ItemFn; 4],
    /// What choosing it does.
    pub action: ItemFn,
    /// The skip mark as the game starts; changed in play.
    pub skip_mark: u32,
    pub measuring_font: MenuFont,
    pub highlight_font: MenuFont,
    /// What `menu::run_item_action` hands the action (and stores in `DrawWork.action_mode`).
    pub action_value: u32,
    pub spaced: u32,
}

/// A screen's list of functions, run in order (`menu::run_list`).
#[derive(Debug)]
pub struct DrawList {
    /// Whether anything names it (ten follow another's end mark and nothing refers to them).
    pub referenced: bool,
    pub functions: &'static [ScreenFn],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts() {
        assert_eq!(MENUS.len(), 96);
        assert_eq!(MENUS.iter().map(|m| m.items.len()).sum::<usize>(), 566);
        assert_eq!(MENUS.iter().filter(|m| !m.referenced).count(), 8);
        assert_eq!(LISTS.len(), 301);
        assert_eq!(LISTS.iter().filter(|l| !l.referenced).count(), 10);
    }

    /// The FNV-1a hash of a text.
    fn digest(text: &str) -> u64 {
        text.bytes().fold(0xCBF2_9CE4_8422_2325u64, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01B3)
        })
    }

    /// The menus and the lists are as they were frozen: every field of every menu, label, item
    /// and list, by the hash of their debug text.
    #[test]
    fn tables_are_as_frozen() {
        let menus = digest(&format!("{MENUS:?}"));
        let lists = digest(&format!("{LISTS:?}"));
        assert_eq!(
            (menus, lists),
            (FROZEN_MENUS, FROZEN_LISTS),
            "{menus:#018x} {lists:#018x}"
        );
    }

    const FROZEN_MENUS: u64 = 0xdb43_1fd2_ed24_2576;
    const FROZEN_LISTS: u64 = 0xc567_a63a_a930_5dc2;

    #[test]
    fn the_lists_of_all_are_whole() {
        let used_item: std::collections::HashSet<_> = MENUS
            .iter()
            .flat_map(|m| m.items)
            .flat_map(|i| i.on_key.into_iter().chain([i.action]))
            .collect();
        assert!(used_item.iter().all(|f| ItemFn::ALL.contains(f)));
        let used_screen: std::collections::HashSet<_> =
            LISTS.iter().flat_map(|l| l.functions).collect();
        assert!(used_screen.iter().all(|f| ScreenFn::ALL.contains(f)));
        assert_eq!(MenuFont::ALL.len(), 12);
    }
}
