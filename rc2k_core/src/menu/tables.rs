//! The game's menus and its screens' draw lists as Rust data (typed tables):
//! what RAL.EXE keeps at 0x747000 .. 0x755200 (menus, their labels) and 0x758B00 ..
//! 0x7594B0 (draw lists), with the code addresses in them replaced by enums
//! (`ItemFn`, `ScreenFn`) and the fonts they name by `MenuFont`. The data is generated
//! (`tables_data.rs`, `tools/menu_tables`).
//!
//! These tables are the one source of the menus and the lists: the game's crate keeps, per
//! item, only the fields the game changes in play (an item's string id, font, +0x1C, sprite,
//! skip mark and highlight font: `menu::menus_store::MenuItem`, starting as the table has
//! them) and names a menu or list by its number here. Each menu and list keeps its RAL
//! address (`at`) for the tests that compare the tables with the image.

pub use super::item_params::{EventStart, OptionSwitch, StageCounter, VolumeKind};
pub use super::tables_data::{ItemFn, LISTS, MENUS, MenuFont, ScreenFn};

/// A menu: its labels (texts that are not items) and its items, without the -1 dwords
/// that end both lists in RAL.EXE.
#[derive(Debug)]
pub struct Menu {
    /// Where RAL.EXE keeps it.
    pub at: u32,
    /// Whether anything in RAL.EXE names it (eight menus follow another's end mark and are
    /// named by nothing).
    pub referenced: bool,
    /// Where RAL.EXE keeps its labels (0: none).
    pub labels_at: u32,
    pub labels: &'static [MenuLabel],
    pub items: &'static [MenuItem],
}

/// A label: 0x10 bytes in RAL.EXE.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuLabel {
    pub x: i32,
    pub y: i32,
    pub font: MenuFont,
    pub string: u32,
}

/// An item: 0x5C bytes in RAL.EXE (IDA's `MenuItem`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuItem {
    pub x: i32,
    pub y: i32,
    pub width_or_text_x: i32,
    pub alignment_or_text_y: i32,
    /// IDA's `type`: the number highlights and links name the item by.
    pub id: u32,
    /// The string id (+0x14), as the game starts; changed in play.
    pub string: u32,
    /// +0x18, as the game starts; changed in play (`menu::disable_item` / `enable_item`).
    pub font: MenuFont,
    /// +0x1C, as the game starts; changed in play.
    pub behind: u32,
    pub sprites: u32,
    /// The items up, down, left and right of this one, by `id`.
    pub links: [u32; 4],
    /// What a key does on it: up, down, left, right (+0x34 .. +0x40).
    pub on_key: [ItemFn; 4],
    /// What choosing it does (+0x44).
    pub action: ItemFn,
    /// +0x48, as the game starts; changed in play.
    pub skip_mark: u32,
    pub measuring_font: MenuFont,
    pub highlight_font: MenuFont,
    /// What `menu::run_item_action` hands the action (in eax, and in `dword_755A38`).
    pub action_value: u32,
    pub spaced: u32,
}

/// A screen's list of functions, run in order (`menu::run_list`): RAL.EXE's dwords
/// without the -1 that ends them.
#[derive(Debug)]
pub struct DrawList {
    pub at: u32,
    /// Whether anything in RAL.EXE names it (ten follow another's end mark).
    pub referenced: bool,
    pub functions: &'static [ScreenFn],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorted_and_found() {
        assert!(MENUS.windows(2).all(|w| w[0].at < w[1].at));
        assert!(LISTS.windows(2).all(|w| w[0].at < w[1].at));
    }

    #[test]
    fn counts() {
        assert_eq!(MENUS.len(), 96);
        assert_eq!(MENUS.iter().map(|m| m.items.len()).sum::<usize>(), 567);
        assert_eq!(MENUS.iter().filter(|m| !m.referenced).count(), 8);
        assert_eq!(LISTS.len(), 301);
        assert_eq!(LISTS.iter().filter(|l| !l.referenced).count(), 10);
    }

    /// RAL.EXE's image as the standalone builds it (`data/ral_image.bin`), unmoved (its
    /// addresses as IDA shows them), indexed by RVA.
    fn image() -> Vec<u8> {
        // `RC2K_IMAGE` for a test binary run away from the source tree.
        let path = std::env::var("RC2K_IMAGE").unwrap_or_else(|_| {
            concat!(env!("CARGO_MANIFEST_DIR"), "/../data/ral_image.bin").to_string()
        });
        let d = std::fs::read(path).expect("data/ral_image.bin");
        let u = |at: usize| u32::from_le_bytes(d[at..at + 4].try_into().unwrap()) as usize;
        assert_eq!(&d[..8], b"RC2KIMG1");
        let (size, sections) = (u(8), u(16));
        let mut image = vec![0u8; size];
        let mut at = 20;
        for _ in 0..sections {
            let (rva, flags) = (u(at), u(at + 8));
            at += 12;
            if flags & 4 != 0 {
                let count = u(at);
                at += 4;
                for _ in 0..count {
                    let (place, n) = (u(at), u(at + 4));
                    image[place..place + n].copy_from_slice(&d[at + 8..at + 8 + n]);
                    at += 8 + n;
                }
            }
            if flags & 2 == 0 {
                continue;
            }
            let raw = u(at);
            at += 4;
            let mut done = 0;
            while done < raw {
                let (zeros, literal) = (u(at), u(at + 4));
                let to = rva + done + zeros;
                image[to..to + literal].copy_from_slice(&d[at + 8..at + 8 + literal]);
                at += 8 + literal;
                done += zeros + literal;
            }
        }
        image
    }

    /// Every dword of every menu, label, item and list as RAL.EXE has it: the tables are
    /// the game's data.
    #[test]
    fn tables_are_the_games() {
        let image = image();
        let dword = |at: u32| {
            let at = (at - 0x40_0000) as usize;
            u32::from_le_bytes(image[at..at + 4].try_into().unwrap())
        };
        let font = |at: u32| MenuFont::from_ral(dword(at));
        let item_fn = |at: u32| ItemFn::from_ral(dword(at));
        for m in MENUS.iter() {
            assert_eq!(dword(m.at), m.labels_at, "menu {:#x}", m.at);
            for (n, l) in m.labels.iter().enumerate() {
                let at = m.labels_at + 0x10 * n as u32;
                assert_eq!(
                    (
                        dword(at) as i32,
                        dword(at + 4) as i32,
                        font(at + 8),
                        dword(at + 12)
                    ),
                    (l.x, l.y, Some(l.font), l.string),
                    "label {at:#x}"
                );
            }
            if m.labels_at != 0 {
                assert_eq!(dword(m.labels_at + 0x10 * m.labels.len() as u32), u32::MAX);
            }
            for (n, it) in m.items.iter().enumerate() {
                let at = m.at + 4 + 0x5C * n as u32;
                let w = |o: u32| dword(at + o);
                let i = |o: u32| dword(at + o) as i32;
                assert_eq!(
                    (i(0), i(4), i(8), i(0xC)),
                    (it.x, it.y, it.width_or_text_x, it.alignment_or_text_y),
                    "item {at:#x}"
                );
                assert_eq!(
                    (w(0x10), w(0x14), font(at + 0x18), w(0x1C), w(0x20)),
                    (it.id, it.string, Some(it.font), it.behind, it.sprites),
                    "item {at:#x}"
                );
                assert_eq!(
                    [w(0x24), w(0x28), w(0x2C), w(0x30)],
                    it.links,
                    "item {at:#x}"
                );
                assert_eq!(
                    [0x34, 0x38, 0x3C, 0x40, 0x44].map(|o| item_fn(at + o)),
                    [
                        it.on_key[0],
                        it.on_key[1],
                        it.on_key[2],
                        it.on_key[3],
                        it.action
                    ]
                    .map(Some),
                    "item {at:#x}"
                );
                assert_eq!(
                    (w(0x48), font(at + 0x4C), font(at + 0x50), w(0x54), w(0x58)),
                    (
                        it.skip_mark,
                        Some(it.measuring_font),
                        Some(it.highlight_font),
                        it.action_value,
                        it.spaced
                    ),
                    "item {at:#x}"
                );
            }
            assert_eq!(dword(m.at + 4 + 0x5C * m.items.len() as u32), u32::MAX);
        }
        for l in LISTS.iter() {
            for (n, f) in l.functions.iter().enumerate() {
                assert_eq!(
                    ScreenFn::from_ral(dword(l.at + 4 * n as u32)),
                    Some(*f),
                    "list {:#x}",
                    l.at
                );
            }
            assert_eq!(dword(l.at + 4 * l.functions.len() as u32), u32::MAX);
        }
    }

    #[test]
    fn enums_and_addresses_agree() {
        for f in ItemFn::ALL {
            assert_eq!(ItemFn::from_ral(f.ral()), Some(*f));
        }
        for f in ScreenFn::ALL {
            assert_eq!(ScreenFn::from_ral(f.ral()), Some(*f));
        }
        for f in MenuFont::ALL {
            assert_eq!(MenuFont::from_ral(f.ral()), Some(*f));
        }
    }
}
