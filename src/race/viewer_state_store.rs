//! The replay writer's and the viewer's state: the padding frame, the file offset and frames in the
//! block, the replay block and frame, the play end and the viewer's switches, then the viewer's
//! list (header and 26 items of 16 bytes). `#[repr(C)]`, laid over the race block's bytes. The
//! list's names are image addresses into the untyped rectangles and key table.

use crate::cells::ZeroCells;

use crate::list_handle::{Item, ListData26, ListHeader};
use crate::place::BytePlace;
#[cfg(test)]
use crate::race::owner::race;
use crate::race::owner_replay::Replay;
use crate::session::{PlainDword, PlainFlag};

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells, rc2k_derive::ByteView)]
pub struct ReplayViewBlock {
    /// +0x0: the 16-byte frame that pads a block's end, zeros (`race::replay`,
    /// `race::add_replay_frame`).
    pub pad_frame: [PlainDword; 4],
    /// +0x10: the new replay file's length so far (`race::save_viewed_replay`, `race::replay`).
    pub file_offset: PlainDword,
    /// +0x14: frames in the block being written, 0 .. 100 (`race::replay`,
    /// `drive::write_replay_frames`).
    pub frames_in_block: PlainDword,
    /// +0x18: the block shown/played (`race::viewer`, `drive::replay_clock_tick`,
    /// `drive::read_replay_frame`).
    pub replay_block: PlainDword,
    /// +0x1c: its frame in the block, 0 .. 99 (`race::viewer`, `drive::replay_clock_tick`).
    pub replay_frame: PlainDword,
    /// +0x20: playing stops before this block, -1 none (`race::viewer`, `drive::race_loop`,
    /// `drive::finish_run`).
    pub play_end: PlainDword,
    /// +0x24: play from here (`race::viewer`).
    pub play_from_here: PlainDword,
    /// +0x28: a picture of the next frame asked (`race::viewer_frame`).
    pub snapshot_asked: PlainDword,
    /// +0x2c: the slot is 222 (`race::viewer`).
    pub last_race_viewed: PlainFlag,
    /// +0x30: modes 4 and 5 (`race::viewer`).
    pub viewer_only_shown: PlainDword,
    /// +0x34: mode 5 (`race::viewer`).
    pub viewer_keys_ignored: PlainDword,
    /// +0x38: the key after a play; starts at -1 (`race::viewer`).
    pub key_after_play: PlainDword,
    /// +0x3c: keys held after playing (`race::viewer`).
    pub wait_key_release: PlainDword,
    /// +0x40: Escape twice leaves (`race::viewer`).
    pub escape_count: PlainDword,
    /// +0x44: the viewer's list: 26 items. The header's `neighbours` names the key table; an
    /// item's `box_link` names its screen rectangle, its `data` its picture (image addresses into
    /// `RECTANGLES` and the viewer panel); its `count` is the action Enter does.
    pub viewer_list: ListData26,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl ReplayViewBlock {
    /// The viewer's items: kind 1 (a picture), action `n` for item `n`, and the names of its
    /// screen rectangle and picture.
    const fn items() -> [Item; 26] {
        const NAMES: [(u32, u32); 26] = [
            (0x5C277C, 0x5C2A2C),
            (0x5C278C, 0x5C2A4C),
            (0x5C279C, 0x5C2A6C),
            (0x5C27AC, 0x5C2A8C),
            (0x5C27BC, 0x5C2AAC),
            (0x5C27CC, 0x5C2ADC),
            (0x5C27DC, 0x5C2AEC),
            (0x5C27EC, 0x5C2AFC),
            (0x5C27FC, 0x5C2ABC),
            (0x5C280C, 0x5C2A9C),
            (0x5C281C, 0x5C2A7C),
            (0x5C282C, 0x5C2A5C),
            (0x5C283C, 0x5C2A3C),
            (0x5C284C, 0x5C2B0C),
            (0x5C285C, 0x5C2C70),
            (0x5C286C, 0x5C2C78),
            (0x5C287C, 0x5C2C88),
            (0x5C288C, 0x5C2BAC),
            (0x5C289C, 0x5C2BBC),
            (0x5C28AC, 0x5C2BCC),
            (0x5C28BC, 0x5C2BDC),
            (0x5C28CC, 0x5C2B2C),
            (0x5C28DC, 0x5C2BEC),
            (0x5C28EC, 0x5C2BFC),
            (0x5C28FC, 0x5C2C70),
            (0x5C290C, 0x5C2C1C),
        ];
        let mut items = [const { Item::new(0, 0, 0, 0, 0, 0) }; 26];
        let mut n = 0;
        while n < 26 {
            items[n] = Item::new(1, n as u8, 0, NAMES[n].0, 0, NAMES[n].1);
            n += 1;
        }
        items
    }

    pub const fn new() -> Self {
        Self {
            replay_block: PlainDword::new(0xFFFFFFFF),
            viewer_list: ListData26::new(
                ListHeader::new(0, 26, 0, 0, 0, 0, 0x5C2714, 0),
                Self::items(),
            ),
            ..ZeroCells::ZERO
        }
    }
}

/// The viewer list's key table and place rectangles: the list's names point into them, and
/// nothing writes them.
pub static RECTANGLES: crate::session::Bytes<0x208> = crate::session::Bytes::new([
    0xFF, 0x0D, 0xFF, 1, 0xFF, 0x0E, 0, 2, 0xFF, 0x0F, 1, 3, 0xFF, 0x10, 2, 4, 0xFF, 0x11, 3, 5,
    0xFF, 0x12, 4, 6, 0xFF, 0x13, 5, 7, 0xFF, 0x14, 6, 8, 0xFF, 0x15, 7, 9, 0xFF, 0x16, 8, 0x0A,
    0xFF, 0x17, 9, 0x0B, 0xFF, 0x18, 0x0A, 0x0C, 0xFF, 0x19, 0x0B, 0xFF, 0, 0xFF, 0xFF, 0x0E, 1,
    0xFF, 0x0D, 0x0F, 2, 0xFF, 0x0E, 0x10, 3, 0xFF, 0x0F, 0x11, 4, 0xFF, 0x10, 0x12, 5, 0xFF, 0x11,
    0x13, 6, 0xFF, 0x12, 0x14, 7, 0xFF, 0x13, 0x15, 8, 0xFF, 0x14, 0x16, 9, 0xFF, 0x15, 0x17, 0x0A,
    0xFF, 0x16, 0x18, 0x0B, 0xFF, 0x17, 0x19, 0x0C, 0xFF, 0x18, 0xFF, 0x0A, 0, 0, 0, 0x39, 0, 0, 0,
    0x30, 0, 0, 0, 0x30, 0, 0, 0, 0x47, 0, 0, 0, 0x39, 0, 0, 0, 0x30, 0, 0, 0, 0x30, 0, 0, 0, 0x84,
    0, 0, 0, 0x39, 0, 0, 0, 0x30, 0, 0, 0, 0x30, 0, 0, 0, 0xC1, 0, 0, 0, 0x39, 0, 0, 0, 0x30, 0, 0,
    0, 0x30, 0, 0, 0, 0xFE, 0, 0, 0, 0x39, 0, 0, 0, 0x30, 0, 0, 0, 0x30, 0, 0, 0, 0x3B, 1, 0, 0,
    0x39, 0, 0, 0, 0x30, 0, 0, 0, 0x30, 0, 0, 0, 0x78, 1, 0, 0, 0x39, 0, 0, 0, 0x30, 0, 0, 0, 0x30,
    0, 0, 0, 0xB5, 1, 0, 0, 0x39, 0, 0, 0, 0x30, 0, 0, 0, 0x30, 0, 0, 0, 0xF2, 1, 0, 0, 0x39, 0, 0,
    0, 0x30, 0, 0, 0, 0x30, 0, 0, 0, 0x2F, 2, 0, 0, 0x39, 0, 0, 0, 0x30, 0, 0, 0, 0x30, 0, 0, 0,
    0x6C, 2, 0, 0, 0x39, 0, 0, 0, 0x30, 0, 0, 0, 0x30, 0, 0, 0, 0xA9, 2, 0, 0, 0x39, 0, 0, 0, 0x30,
    0, 0, 0, 0x30, 0, 0, 0, 0xE6, 2, 0, 0, 0x39, 0, 0, 0, 0x30, 0, 0, 0, 0x30, 0, 0, 0, 0x0A, 0, 0,
    0, 0x76, 0, 0, 0, 0x30, 0, 0, 0, 0x30, 0, 0, 0, 0x47, 0, 0, 0, 0x76, 0, 0, 0, 0x30, 0, 0, 0,
    0x30, 0, 0, 0, 0x84, 0, 0, 0, 0x76, 0, 0, 0, 0x30, 0, 0, 0, 0x30, 0, 0, 0, 0xC1, 0, 0, 0, 0x76,
    0, 0, 0, 0x30, 0, 0, 0, 0x30, 0, 0, 0, 0xFE, 0, 0, 0, 0x76, 0, 0, 0, 0x30, 0, 0, 0, 0x30, 0, 0,
    0, 0x3B, 1, 0, 0, 0x76, 0, 0, 0, 0x30, 0, 0, 0, 0x30, 0, 0, 0, 0x78, 1, 0, 0, 0x76, 0, 0, 0,
    0x30, 0, 0, 0, 0x30, 0, 0, 0, 0xB5, 1, 0, 0, 0x76, 0, 0, 0, 0x30, 0, 0, 0, 0x30, 0, 0, 0, 0xF2,
    1, 0, 0, 0x76, 0, 0, 0, 0x30, 0, 0, 0, 0x30, 0, 0, 0, 0x2F, 2, 0, 0, 0x76, 0, 0, 0, 0x30, 0, 0,
    0, 0x30, 0, 0, 0, 0x6C, 2, 0, 0, 0x76, 0, 0, 0, 0x30, 0, 0, 0, 0x30, 0, 0, 0, 0xA9, 2, 0, 0,
    0x76, 0, 0, 0, 0x30, 0, 0, 0, 0x30, 0, 0, 0, 0xE6, 2, 0, 0, 0x76, 0, 0, 0, 0x30, 0, 0, 0, 0x30,
    0, 0, 0,
]);

/// What a viewer item shows: a texture rectangle, or the picture a camera byte picks from one of
/// the three switch tables (0 `switch_pictures_a`, 1 `switch_pictures_b`, 2 `switch_pictures_c`
/// of the viewer panel; the list names a table by its image address).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ViewerPicture {
    /// The rectangle's image address in the viewer panel.
    Rectangle(usize),
    Switch(usize),
}

impl ReplayViewBlock {
    /// The image address ranges the list's names point into: the key table and place
    /// rectangles (`RECTANGLES`), and the panel with the pictures' rectangles.
    const RECTANGLES: core::ops::Range<usize> = 0x5C2714..0x5C291C;
    /// The panel follows the rectangles; its names run over its size.
    const PANEL: core::ops::Range<usize> = Self::RECTANGLES.end
        ..Self::RECTANGLES.end
            + core::mem::size_of::<crate::race::viewer_screen_store::ViewerPanelBlock>();

    /// The place the name `ida` (an image address) names, if it lies in the rectangles: a place
    /// in `RECTANGLES`, running on to its end.
    fn named(ida: u32, what: &'static str) -> Result<BytePlace, crate::errors::BadInput> {
        if !Self::RECTANGLES.contains(&(ida as usize)) {
            return Err(crate::errors::BadInput(what));
        }
        Ok(BytePlace::Store(
            &RECTANGLES,
            ida as usize - Self::RECTANGLES.start,
        ))
    }

    /// Where the list's key table (the header's +0x10) is.
    pub fn key_table(replay: &Replay) -> Result<BytePlace, crate::errors::BadInput> {
        Self::named(
            replay.replay_view().viewer_list.header.neighbours.get(),
            "the viewer list's key table off its place",
        )
    }

    /// Where item `index`'s screen rectangle (its +4) is.
    pub fn item_place(replay: &Replay, index: u32) -> Result<BytePlace, crate::errors::BadInput> {
        Self::named(
            Self::item(replay, index)?.box_link.get(),
            "a viewer item's place off the rectangles",
        )
    }

    /// What item `index`'s picture (its +0xC: a rectangle, or one of the switch tables) is: the
    /// picture's name is compared with the three tables' addresses.
    pub fn item_picture(
        replay: &Replay,
        index: u32,
    ) -> Result<ViewerPicture, crate::errors::BadInput> {
        let ida = Self::item(replay, index)?.data.get();
        let table = |offset: usize| (Self::PANEL.start + offset) as u32;
        use crate::race::viewer_screen_store::ViewerPanelBlock;
        use core::mem::offset_of;
        let switch = if ida == table(offset_of!(ViewerPanelBlock, switch_pictures_a)) {
            Some(0)
        } else if ida == table(offset_of!(ViewerPanelBlock, switch_pictures_b)) {
            Some(1)
        } else if ida == table(offset_of!(ViewerPanelBlock, switch_pictures_c)) {
            Some(2)
        } else {
            None
        };
        if let Some(table) = switch {
            return Ok(ViewerPicture::Switch(table));
        }
        if !Self::PANEL.contains(&(ida as usize)) {
            return Err(crate::errors::BadInput(
                "a viewer item's picture off the panel",
            ));
        }
        Ok(ViewerPicture::Rectangle(ida as usize))
    }

    /// Item `index` of the viewer's list.
    fn item(replay: &Replay, index: u32) -> Result<&Item, crate::errors::BadInput> {
        replay
            .replay_view()
            .viewer_list
            .items
            .get(index as usize)
            .ok_or(crate::errors::BadInput("a viewer item past the 26"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_list_names_resolve() {
        let replay = &race().replay;
        assert!(ReplayViewBlock::key_table(replay).is_ok());
        for k in 0..26u32 {
            assert!(ReplayViewBlock::item_place(replay, k).is_ok());
            match ReplayViewBlock::item_picture(replay, k).unwrap() {
                ViewerPicture::Rectangle(_) => {}
                ViewerPicture::Switch(table) => assert!(table < 3),
            }
        }
        assert!(ReplayViewBlock::item_place(replay, 26).is_err());
    }

    /// M4 for the viewer's list: the list holds 26 items and its count says so; the key table
    /// only ever names items of the list (or 255 for none), so no key reaches past it.
    #[test]
    fn the_viewer_never_reaches_past_its_items() {
        let replay = &race().replay;
        let list = &replay.replay_view().viewer_list;
        assert_eq!(usize::from(list.header.count.get()), list.items.len());
        let table = ReplayViewBlock::key_table(replay).unwrap();
        let bytes = table.read(4 * list.items.len(), "the key table").unwrap();
        assert!(
            bytes
                .iter()
                .all(|&entry| entry == 0xFF || usize::from(entry) < list.items.len())
        );
    }
}
