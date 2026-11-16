//! The game's menus and its screens' draw lists as Rust data: functions are the enums `ItemFn` and
//! `ScreenFn`, fonts are `MenuFont`. The data is generated (`tables_data.rs`, `tools/menu_tables`).
//!
//! These tables are the one source of the menus and the lists: the game's crate keeps, per item,
//! only the fields that change in play (string id, font, `behind`, sprite, skip mark and highlight
//! font: `menu::menus_store::MenuItem`, starting as the table has them) and names a menu or list by
//! its number here. Each menu and list keeps the address RAL.EXE stores it at (`at`), for the test
//! that checks the tables against RAL.EXE's data.

pub use super::item_params::{EventStart, OptionSwitch, StageCounter, VolumeKind};
pub use super::tables_data::{ItemFn, LISTS, MENUS, MenuFont, ScreenFn};

/// A menu: its labels (texts that are not items) and its items, without the -1 dwords that end both
/// lists in RAL.EXE's layout.
#[derive(Debug)]
pub struct Menu {
    /// The address RAL.EXE stores it at.
    pub at: u32,
    /// Whether anything names it (eight menus follow another's end mark and nothing refers
    /// to them).
    pub referenced: bool,
    /// The address RAL.EXE stores its labels at (0: none).
    pub labels_at: u32,
    pub labels: &'static [MenuLabel],
    pub items: &'static [MenuItem],
}

/// A label: 0x10 bytes in RAL.EXE's layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuLabel {
    pub x: i32,
    pub y: i32,
    pub font: MenuFont,
    pub string: u32,
}

/// An item: 0x5C bytes in RAL.EXE's layout.
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
    /// The item's backdrop (+0x1C in RAL.EXE's layout) as the game starts; changed in play.
    pub behind: u32,
    pub sprites: u32,
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

/// A screen's list of functions, run in order (`menu::run_list`): RAL.EXE's dwords without the -1
/// that ends them in RAL.EXE.
#[derive(Debug)]
pub struct DrawList {
    pub at: u32,
    /// Whether anything names it (ten follow another's end mark and nothing refers to them).
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
        assert_eq!(MENUS.iter().map(|m| m.items.len()).sum::<usize>(), 566);
        assert_eq!(MENUS.iter().filter(|m| !m.referenced).count(), 8);
        assert_eq!(LISTS.len(), 301);
        assert_eq!(LISTS.iter().filter(|l| !l.referenced).count(), 10);
    }

    /// The sound options' menu.
    const SOUND_MENU: u32 = 0x754934;

    /// The sound options' rows as RAL.EXE has them. Its sixth row switched Aureal's A3D, which
    /// the game no longer has: the tables leave the row out, Dolby's row stands in its place
    /// and takes its number, and the rows that led to either lead to Dolby. Here the row is
    /// put back (the EAX row's look, one row down) and Dolby's moved back down.
    fn with_a3d_row(rows: &mut Vec<[u32; 23]>) {
        // A row's dwords: its place, its text's, its number and string, the rows above and
        // below, and what the left and right keys do.
        let (y, text_y, id, string, up, down, key_left, key_right) = (1, 3, 4, 5, 9, 10, 15, 16);
        assert_eq!(rows.len(), 6);
        rows[0][up] = 7;
        let mut a3d = rows[4];
        (a3d[y], a3d[text_y], a3d[id], a3d[string]) = (383, 375, 6, 0x551);
        (a3d[up], a3d[down], a3d[key_left], a3d[key_right]) = (5, 7, 0x470203, 0x470203);
        let dolby = &mut rows[5];
        (dolby[y], dolby[text_y], dolby[id], dolby[up]) = (410, 402, 7, 6);
        rows.insert(5, a3d);
    }

    /// Every dword the tables stand for, as RAL.EXE lays them out: `(address, dword)` for each
    /// menu's label pointer, label, item and end mark, and each list's functions and end mark, the
    /// enums turned back into the addresses they replace.
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
            let mut rows: Vec<[u32; 23]> = m
                .items
                .iter()
                .map(|it| {
                    [
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
                    ]
                })
                .collect();
            if m.at == SOUND_MENU {
                with_a3d_row(&mut rows);
            }
            for (n, fields) in rows.iter().enumerate() {
                let at = m.at + 4 + 0x5C * n as u32;
                out.extend((0..).step_by(4).zip(*fields).map(|(o, w)| (at + o, w)));
            }
            out.push((m.at + 4 + 0x5C * rows.len() as u32, u32::MAX));
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

    /// Every dword of every menu, label, item and list, at its RAL.EXE address, equals what RAL.EXE
    /// holds there: checked by count and hash, frozen from a comparison with RAL.EXE's bytes that
    /// covered every dword (the sound options' A3D row, which the tables leave out, put back
    /// for it: `with_a3d_row`). The one difference the enums make: the 194 item functions at 0x471C81
    /// (a second empty function) read as `ItemFn::Nothing`, which stands for 0x471C80.
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
