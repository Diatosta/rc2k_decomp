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
//! address (`at`) for the test that checks the tables against RAL.EXE's (frozen) data.

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
    /// What `menu::run_item_action` hands the action (and stores in `DrawWork.action_mode`).
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

    /// Every dword the tables stand for, as RAL.EXE lays them out: `(IDA address, dword)` for
    /// each menu's label pointer, label, item and end mark, and each list's functions and end
    /// mark, the enums turned back into the addresses they replace.
    fn ral_dwords() -> Vec<(u32, u32)> {
        let mut out = Vec::new();
        for m in MENUS.iter() {
            out.push((m.at, m.labels_at));
            for (n, l) in m.labels.iter().enumerate() {
                let at = m.labels_at + 0x10 * n as u32;
                out.extend([
                    (at, l.x as u32),
                    (at + 4, l.y as u32),
                    (at + 8, l.font.ral()),
                    (at + 12, l.string),
                ]);
            }
            if m.labels_at != 0 {
                out.push((m.labels_at + 0x10 * m.labels.len() as u32, u32::MAX));
            }
            for (n, it) in m.items.iter().enumerate() {
                let at = m.at + 4 + 0x5C * n as u32;
                let fields = [
                    it.x as u32,
                    it.y as u32,
                    it.width_or_text_x as u32,
                    it.alignment_or_text_y as u32,
                    it.id,
                    it.string,
                    it.font.ral(),
                    it.behind,
                    it.sprites,
                    it.links[0],
                    it.links[1],
                    it.links[2],
                    it.links[3],
                    it.on_key[0].ral(),
                    it.on_key[1].ral(),
                    it.on_key[2].ral(),
                    it.on_key[3].ral(),
                    it.action.ral(),
                    it.skip_mark,
                    it.measuring_font.ral(),
                    it.highlight_font.ral(),
                    it.action_value,
                    it.spaced,
                ];
                out.extend((0..).step_by(4).zip(fields).map(|(o, w)| (at + o, w)));
            }
            out.push((m.at + 4 + 0x5C * m.items.len() as u32, u32::MAX));
        }
        for l in LISTS.iter() {
            for (n, f) in l.functions.iter().enumerate() {
                out.push((l.at + 4 * n as u32, f.ral()));
            }
            out.push((l.at + 4 * l.functions.len() as u32, u32::MAX));
        }
        out
    }

    /// The FNV-1a hash of `(address, dword)` pairs, little-endian.
    fn digest(dwords: &[(u32, u32)]) -> u64 {
        dwords
            .iter()
            .flat_map(|(at, w)| at.to_le_bytes().into_iter().chain(w.to_le_bytes()))
            .fold(0xCBF2_9CE4_8422_2325u64, |h, b| {
                (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01B3)
            })
    }

    /// The tables are the game's data: every dword of every menu, label, item and list, at
    /// its RAL.EXE address, is what RAL.EXE holds there. Frozen on 2026-11-13 from the
    /// comparison with RAL.EXE's bytes (then `data/menu_tables.bin`, 14,196 dwords, every
    /// one covered): their count and hash. The one difference the enums make: RAL.EXE's
    /// 194 item functions at 0x471C81 (a second empty function next to `nullsub_87`) read
    /// as `ItemFn::Nothing`, which stands for 0x471C80.
    #[test]
    fn tables_are_the_games() {
        let dwords = ral_dwords();
        assert_eq!(dwords.len(), FROZEN_DWORDS);
        assert_eq!(digest(&dwords), FROZEN_DIGEST, "{:#018x}", digest(&dwords));
    }

    /// The dwords' count (16 addresses come twice, reached from two tables, each time with
    /// the same dword) and hash.
    const FROZEN_DWORDS: usize = 14_212;
    const FROZEN_DIGEST: u64 = 0x5b76_643d_e740_5f28;

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
