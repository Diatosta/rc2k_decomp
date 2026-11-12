//! The replay writer's and the viewer's state (IDA 0x5C2518 .. 0x5C2714; phase 7, typed): the padding frame, the file offset and frames in the block, the replay block and frame, the play end and the viewer's switches, then the viewer's list (header and 26 items of 16 bytes). `#[repr(C)]` with the image's layout, over the race block's bytes. The list's pointers are relocated addresses into the untyped rectangles and key table at 0x5C2714 .. 0x5C291C. Names are readings unless IDA's.

use crate::cells::ZeroCells;

use crate::place::BytePlace;
#[cfg(test)]
use crate::race::owner::race;
use crate::race::owner_replay::Replay;
use crate::session::{PlainDword, PlainFlag};

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ReplayViewBlock {
    /// +0x0 (0x5C2518): the 16-byte frame that pads a block's end, zeros (`race::replay`, `race::add_replay_frame`).
    pub pad_frame: [PlainDword; 4],
    /// +0x10 (0x5C2528): `dword_5C2528`: the new replay file's length so far (`race::save_viewed_replay`, `race::replay`).
    pub file_offset: PlainDword,
    /// +0x14 (0x5C252C): `dword_5C252C`: frames in the block being written, 0 .. 100 (`race::replay`, `drive::write_replay_frames`).
    pub frames_in_block: PlainDword,
    /// +0x18 (0x5C2530): `g_replayBlock`: the block shown/played (`race::viewer`, `drive::replay_clock_tick`, `drive::read_replay_frame`).
    pub replay_block: PlainDword,
    /// +0x1c (0x5C2534): `g_replayFrame`: its frame in the block, 0 .. 99 (`race::viewer`, `drive::replay_clock_tick`).
    pub replay_frame: PlainDword,
    /// +0x20 (0x5C2538): `g_playEnd`: playing stops before this block, -1 none (`race::viewer`, `drive::race_loop`, `drive::finish_run`).
    pub play_end: PlainDword,
    /// +0x24 (0x5C253C): `g_playFromHere` (`race::viewer`).
    pub play_from_here: PlainDword,
    /// +0x28 (0x5C2540): `g_snapshotAsked`: a picture of the next frame asked (`race::viewer_frame`).
    pub snapshot_asked: PlainDword,
    /// +0x2c (0x5C2544): `g_lastRaceViewed`: the slot is 222 (`race::viewer`).
    pub last_race_viewed: PlainFlag,
    /// +0x30 (0x5C2548): `g_viewerOnlyShown`: modes 4 and 5 (`race::viewer`).
    pub viewer_only_shown: PlainDword,
    /// +0x34 (0x5C254C): `g_viewerKeysIgnored`: mode 5 (`race::viewer`).
    pub viewer_keys_ignored: PlainDword,
    /// +0x38 (0x5C2550): `g_keyAfterPlay`; image value -1 (`race::viewer`).
    pub key_after_play: PlainDword,
    /// +0x3c (0x5C2554): `g_waitKeyRelease`: keys held after playing (`race::viewer`).
    pub wait_key_release: PlainDword,
    /// +0x40 (0x5C2558): `g_escapeCount`: Escape twice leaves (`race::viewer`).
    pub escape_count: PlainDword,
    /// +0x44 (0x5C255C): `g_viewerList`, taken in place: header 0x18 bytes (byte +1 = item count, set at run time; +0x10 a pointer to the key table), then 26 items of 16 bytes (flags, source rectangle pointer, 0, screen rectangle pointer; relocated pointers) (`race::viewer_frame`, `race::mark_viewer_items`, `race::draw_viewer_item`).
    pub viewer_list: [PlainDword; 110],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl ReplayViewBlock {
    pub const fn new() -> Self {
        Self {
            replay_block: PlainDword::new(0xFFFFFFFF),
            viewer_list: [
                PlainDword::new(0x1A00),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0x5C2714),
                PlainDword::new(0),
                PlainDword::new(1),
                PlainDword::new(0x5C277C),
                PlainDword::new(0),
                PlainDword::new(0x5C2A2C),
                PlainDword::new(0x101),
                PlainDword::new(0x5C278C),
                PlainDword::new(0),
                PlainDword::new(0x5C2A4C),
                PlainDword::new(0x201),
                PlainDword::new(0x5C279C),
                PlainDword::new(0),
                PlainDword::new(0x5C2A6C),
                PlainDword::new(0x301),
                PlainDword::new(0x5C27AC),
                PlainDword::new(0),
                PlainDword::new(0x5C2A8C),
                PlainDword::new(0x401),
                PlainDword::new(0x5C27BC),
                PlainDword::new(0),
                PlainDword::new(0x5C2AAC),
                PlainDword::new(0x501),
                PlainDword::new(0x5C27CC),
                PlainDword::new(0),
                PlainDword::new(0x5C2ADC),
                PlainDword::new(0x601),
                PlainDword::new(0x5C27DC),
                PlainDword::new(0),
                PlainDword::new(0x5C2AEC),
                PlainDword::new(0x701),
                PlainDword::new(0x5C27EC),
                PlainDword::new(0),
                PlainDword::new(0x5C2AFC),
                PlainDword::new(0x801),
                PlainDword::new(0x5C27FC),
                PlainDword::new(0),
                PlainDword::new(0x5C2ABC),
                PlainDword::new(0x901),
                PlainDword::new(0x5C280C),
                PlainDword::new(0),
                PlainDword::new(0x5C2A9C),
                PlainDword::new(0xA01),
                PlainDword::new(0x5C281C),
                PlainDword::new(0),
                PlainDword::new(0x5C2A7C),
                PlainDword::new(0xB01),
                PlainDword::new(0x5C282C),
                PlainDword::new(0),
                PlainDword::new(0x5C2A5C),
                PlainDword::new(0xC01),
                PlainDword::new(0x5C283C),
                PlainDword::new(0),
                PlainDword::new(0x5C2A3C),
                PlainDword::new(0xD01),
                PlainDword::new(0x5C284C),
                PlainDword::new(0),
                PlainDword::new(0x5C2B0C),
                PlainDword::new(0xE01),
                PlainDword::new(0x5C285C),
                PlainDword::new(0),
                PlainDword::new(0x5C2C70),
                PlainDword::new(0xF01),
                PlainDword::new(0x5C286C),
                PlainDword::new(0),
                PlainDword::new(0x5C2C78),
                PlainDword::new(0x1001),
                PlainDword::new(0x5C287C),
                PlainDword::new(0),
                PlainDword::new(0x5C2C88),
                PlainDword::new(0x1101),
                PlainDword::new(0x5C288C),
                PlainDword::new(0),
                PlainDword::new(0x5C2BAC),
                PlainDword::new(0x1201),
                PlainDword::new(0x5C289C),
                PlainDword::new(0),
                PlainDword::new(0x5C2BBC),
                PlainDword::new(0x1301),
                PlainDword::new(0x5C28AC),
                PlainDword::new(0),
                PlainDword::new(0x5C2BCC),
                PlainDword::new(0x1401),
                PlainDword::new(0x5C28BC),
                PlainDword::new(0),
                PlainDword::new(0x5C2BDC),
                PlainDword::new(0x1501),
                PlainDword::new(0x5C28CC),
                PlainDword::new(0),
                PlainDword::new(0x5C2B2C),
                PlainDword::new(0x1601),
                PlainDword::new(0x5C28DC),
                PlainDword::new(0),
                PlainDword::new(0x5C2BEC),
                PlainDword::new(0x1701),
                PlainDword::new(0x5C28EC),
                PlainDword::new(0),
                PlainDword::new(0x5C2BFC),
                PlainDword::new(0x1801),
                PlainDword::new(0x5C28FC),
                PlainDword::new(0),
                PlainDword::new(0x5C2C70),
                PlainDword::new(0x1901),
                PlainDword::new(0x5C290C),
                PlainDword::new(0),
                PlainDword::new(0x5C2C1C),
            ],
            ..ZeroCells::ZERO
        }
    }
}

/// The viewer list's key table and place rectangles (IDA 0x5C2714 .. 0x5C291C, untyped), with
/// the image's first values: the list's names point into them, and nothing writes them.
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

/// What a viewer item shows: a texture rectangle, or the picture a camera byte picks from one
/// of the three switch tables (0 `ViewerAndKinds.viewer_panel.switch_pictures_a[0]`, 1 `ViewerAndKinds.viewer_panel.switch_pictures_b[0]`, 2 `ViewerAndKinds.viewer_panel.switch_pictures_c[0]`; the original
/// kept the table's address).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ViewerPicture {
    /// The rectangle's IDA address in the viewer panel.
    Rectangle(usize),
    Switch(usize),
}

impl ReplayViewBlock {
    /// The count of the viewer's list's items (from 24 bytes in).
    const ITEMS: u32 = 26;
    /// IDA's ranges the list's names point into: the key table and place rectangles
    /// (`RECTANGLES`), and the panel with the pictures' rectangles.
    const RECTANGLES: core::ops::Range<usize> = 0x5C2714..0x5C291C;
    const PANEL: core::ops::Range<usize> =
        crate::race::viewer_screen_store::START..crate::race::viewer_screen_store::END;

    /// The place IDA's name `ida` names, if it lies in the rectangles: a place in
    /// `RECTANGLES`, running on to its end (the list keeps IDA's addresses as names).
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
            replay.replay_view().viewer_list[4].get(),
            "the viewer list's key table off its place",
        )
    }

    /// Where item `index`'s screen rectangle (its +4) is.
    pub fn item_place(replay: &Replay, index: u32) -> Result<BytePlace, crate::errors::BadInput> {
        Self::named(
            Self::item_dword(replay, index, 1)?,
            "a viewer item's place off the rectangles",
        )
    }

    /// What item `index`'s picture (its +0xC: a rectangle, or one of the switch tables) is.
    /// The original compared the picture's address with the three tables' addresses.
    pub fn item_picture(
        replay: &Replay,
        index: u32,
    ) -> Result<ViewerPicture, crate::errors::BadInput> {
        let ida = Self::item_dword(replay, index, 3)?;
        let table = |offset: usize| (crate::race::viewer_screen_store::START + offset) as u32;
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

    fn item_dword(
        replay: &Replay,
        index: u32,
        field: usize,
    ) -> Result<u32, crate::errors::BadInput> {
        if index >= Self::ITEMS {
            return Err(crate::errors::BadInput("a viewer item past the 26"));
        }
        Ok(replay.replay_view().viewer_list[6 + 4 * index as usize + field].get())
    }
}

/// IDA 0x5C2518, the first byte of the piece.
pub const START: usize = 0x5C2518;
/// IDA 0x5C2714, the end of the piece.
pub const END: usize = 0x5C2714;

const _: () = assert!(END - START == core::mem::size_of::<ReplayViewBlock>());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            race().replay.replay_view(),
        );
    }

    #[test]
    fn the_list_names_its_targets_as_the_image_does() {
        let replay = &race().replay;
        // The image's dword at byte `at` of the list.
        let name = |at: usize| {
            crate::test_support::first_values::dword(
                "race.owner.RACE.replay.replay_view.viewer_list[0]/440",
                at / 4,
            )
        };
        let key_table = 0x10;
        assert_eq!(replay.replay_view().viewer_list[4].get(), name(key_table));
        for k in 0..26u32 {
            let item = 0x18 + 0x10 * k as usize;
            // Kept as the image's names; resolved to the same places.
            assert_eq!(
                replay.replay_view().viewer_list[6 + 4 * k as usize + 1].get(),
                name(item + 4)
            );
            assert_eq!(
                replay.replay_view().viewer_list[6 + 4 * k as usize + 3].get(),
                name(item + 0xC)
            );
            match ReplayViewBlock::item_picture(replay, k).unwrap() {
                ViewerPicture::Rectangle(ida) => assert_eq!(ida, name(item + 0xC) as usize),
                ViewerPicture::Switch(table) => assert!(table < 3),
            }
        }
        assert!(ReplayViewBlock::item_place(replay, 26).is_err());
    }
}
