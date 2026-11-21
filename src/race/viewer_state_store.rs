//! The replay writer's and the viewer's state: the padding frame, the file offset and frames in the
//! block, the replay block and frame, the play end and the viewer's switches, then the viewer's
//! list (`race::viewer_list_store`). A plain record of cells only the game's thread reaches
//! (`race::owner_replay::Replay::replay_view`). The list's rectangles, pictures and key table are
//! the constants below.

use std::cell::Cell;

use crate::memory::place::BytePlace;
use crate::race::viewer_list_store::{ITEMS, ViewerItem, ViewerList};

pub struct ReplayViewBlock {
    /// The 16-byte frame that pads a block's end, zeros (`race::replay`,
    /// `race::add_replay_frame`).
    pub pad_frame: [u32; 4],
    /// The new replay file's length so far (`race::save_viewed_replay`, `race::replay`).
    pub file_offset: Cell<u32>,
    /// Frames in the block being written, 0 .. 100 (`race::replay`,
    /// `drive::write_replay_frames`).
    pub frames_in_block: Cell<u32>,
    /// The block shown/played (`race::viewer`, `drive::replay_clock_tick`,
    /// `drive::read_replay_frame`).
    pub replay_block: Cell<u32>,
    /// Its frame in the block, 0 .. 99 (`race::viewer`, `drive::replay_clock_tick`).
    pub replay_frame: Cell<u32>,
    /// Playing stops before this block, -1 none (`race::viewer`, `drive::race_loop`,
    /// `drive::finish_run`).
    pub play_end: Cell<u32>,
    /// Play from here (`race::viewer`).
    pub play_from_here: Cell<u32>,
    /// A picture of the next frame asked (`menu::replay_viewer::frame`).
    pub snapshot_asked: Cell<u32>,
    /// The slot is 222 (`race::viewer`).
    pub last_race_viewed: Cell<bool>,
    /// Modes 4 and 5 (`race::viewer`).
    pub viewer_only_shown: Cell<u32>,
    /// Mode 5 (`race::viewer`).
    pub viewer_keys_ignored: Cell<u32>,
    /// The key after a play; starts at -1 (`race::viewer`).
    pub key_after_play: Cell<u32>,
    /// Keys held after playing (`race::viewer`).
    pub wait_key_release: Cell<u32>,
    /// Escape twice leaves (`race::viewer`).
    pub escape_count: Cell<u32>,
    /// The viewer's list: 26 items (their screen rectangles in `RECTANGLES`, their pictures
    /// `PICTURES`).
    pub viewer_list: ViewerList,
}

impl ReplayViewBlock {
    /// The viewer's items: kind 1 (a picture).
    const fn items() -> [ViewerItem; 26] {
        [const { ViewerItem::new(1) }; ITEMS as usize]
    }

    pub const fn new() -> Self {
        Self {
            pad_frame: [0; 4],
            file_offset: Cell::new(0),
            frames_in_block: Cell::new(0),
            replay_block: Cell::new(0xFFFFFFFF),
            replay_frame: Cell::new(0),
            play_end: Cell::new(0),
            play_from_here: Cell::new(0),
            snapshot_asked: Cell::new(0),
            last_race_viewed: Cell::new(false),
            viewer_only_shown: Cell::new(0),
            viewer_keys_ignored: Cell::new(0),
            key_after_play: Cell::new(0),
            wait_key_release: Cell::new(0),
            escape_count: Cell::new(0),
            viewer_list: ViewerList::new(Self::items()),
        }
    }
}

/// The viewer list's key table (four bytes an item: what each arrow key moves the highlight to,
/// 255 for nowhere) and the items' place rectangles (`ITEM_PLACES`): nothing writes them.
pub static RECTANGLES: crate::memory::session::Bytes<0x208> = crate::memory::session::Bytes::new([
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

/// Which of the viewer panel's three switch tables a picture is picked from by a camera byte
/// (`switch_pictures_a`, `switch_pictures_b`, `switch_pictures_c`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwitchTable {
    A,
    B,
    C,
}

impl SwitchTable {
    /// The table's number in `ViewerPanelBlock::switch_picture`.
    pub const fn number(self) -> usize {
        self as usize
    }
}

/// What a viewer item shows: a texture rectangle, or the picture a camera byte picks from one of
/// the three switch tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewerPicture {
    /// The rectangle at this byte offset in the viewer panel.
    Rectangle(usize),
    Switch(SwitchTable),
}

/// The picture of each of the list's items.
pub const PICTURES: [ViewerPicture; ITEMS as usize] = {
    use ViewerPicture::{Rectangle, Switch};
    [
        Rectangle(0x110),
        Rectangle(0x130),
        Rectangle(0x150),
        Rectangle(0x170),
        Rectangle(0x190),
        Rectangle(0x1C0),
        Rectangle(0x1D0),
        Rectangle(0x1E0),
        Rectangle(0x1A0),
        Rectangle(0x180),
        Rectangle(0x160),
        Rectangle(0x140),
        Rectangle(0x120),
        Rectangle(0x1F0),
        Switch(SwitchTable::A),
        Switch(SwitchTable::B),
        Switch(SwitchTable::C),
        Rectangle(0x290),
        Rectangle(0x2A0),
        Rectangle(0x2B0),
        Rectangle(0x2C0),
        Rectangle(0x210),
        Rectangle(0x2D0),
        Rectangle(0x2E0),
        Switch(SwitchTable::A),
        Rectangle(0x300),
    ]
};

/// Where item 0's screen rectangle (four dwords, 16 bytes an item) starts in `RECTANGLES`, after
/// the key table.
const ITEM_PLACES: usize = 0x68;

impl ReplayViewBlock {
    /// Where the list's key table is.
    pub fn key_table() -> BytePlace {
        BytePlace::In(&RECTANGLES, 0)
    }

    /// Where item `index`'s screen rectangle is.
    pub fn item_place(
        view: &ReplayViewBlock,
        index: u32,
    ) -> Result<BytePlace, crate::errors::BadInput> {
        Self::item(view, index)?;
        Ok(BytePlace::In(
            &RECTANGLES,
            ITEM_PLACES + 16 * index as usize,
        ))
    }

    /// What item `index`'s picture is.
    pub fn item_picture(
        view: &ReplayViewBlock,
        index: u32,
    ) -> Result<ViewerPicture, crate::errors::BadInput> {
        Self::item(view, index)?;
        Ok(PICTURES[index as usize])
    }

    /// Item `index` of the viewer's list.
    fn item(view: &ReplayViewBlock, index: u32) -> Result<&ViewerItem, crate::errors::BadInput> {
        view.viewer_list
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
        let replay = &crate::test_support::own::race().replay;
        for k in 0..26u32 {
            assert!(ReplayViewBlock::item_place(replay.replay_view(), k).is_ok());
            match ReplayViewBlock::item_picture(replay.replay_view(), k).unwrap() {
                ViewerPicture::Rectangle(offset) => assert!(offset + 16 <= 0x668),
                ViewerPicture::Switch(_) => {}
            }
        }
        assert!(ReplayViewBlock::item_place(replay.replay_view(), 26).is_err());
    }

    /// M4 for the viewer's list: the list holds 26 items; the key table
    /// only ever names items of the list (or 255 for none), so no key reaches past it.
    #[test]
    fn the_viewer_never_reaches_past_its_items() {
        let replay = &crate::test_support::own::race().replay;
        let list = &replay.replay_view().viewer_list;
        assert_eq!(list.items.len(), ITEMS as usize);
        let table = ReplayViewBlock::key_table();
        let bytes = table.read(4 * list.items.len(), "the key table").unwrap();
        assert!(
            bytes
                .iter()
                .all(|&entry| entry == 0xFF || usize::from(entry) < list.items.len())
        );
    }
}
