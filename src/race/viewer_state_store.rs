//! The replay writer's and the viewer's state (IDA 0x5C2518 .. 0x5C2714; phase 7, typed): the padding frame, the file offset and frames in the block, the replay block and frame, the play end and the viewer's switches, then the viewer's list (header and 26 items of 16 bytes). `#[repr(C)]` with the image's layout, over the race block's bytes. The list's pointers are relocated addresses into the untyped rectangles and key table at 0x5C2714 .. 0x5C291C. Names are readings unless IDA's.

use crate::place::BytePlace;
use crate::race::owner::race;
use crate::session::{Dword, Flag};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ReplayViewBlock {
    /// +0x0 (0x5C2518): the 16-byte frame that pads a block's end, zeros (`race::replay`, `race::add_replay_frame`).
    pub pad_frame: [Dword; 4],
    /// +0x10 (0x5C2528): `dword_5C2528`: the new replay file's length so far (`race::save_viewed_replay`, `race::replay`).
    pub file_offset: Dword,
    /// +0x14 (0x5C252C): `dword_5C252C`: frames in the block being written, 0 .. 100 (`race::replay`, `drive::write_replay_frames`).
    pub frames_in_block: Dword,
    /// +0x18 (0x5C2530): `g_replayBlock`: the block shown/played (`race::viewer`, `drive::replay_clock_tick`, `drive::read_replay_frame`).
    pub replay_block: Dword,
    /// +0x1c (0x5C2534): `g_replayFrame`: its frame in the block, 0 .. 99 (`race::viewer`, `drive::replay_clock_tick`).
    pub replay_frame: Dword,
    /// +0x20 (0x5C2538): `g_playEnd`: playing stops before this block, -1 none (`race::viewer`, `drive::race_loop`, `drive::finish_run`).
    pub play_end: Dword,
    /// +0x24 (0x5C253C): `g_playFromHere` (`race::viewer`).
    pub play_from_here: Dword,
    /// +0x28 (0x5C2540): `g_snapshotAsked`: a picture of the next frame asked (`race::viewer_frame`).
    pub snapshot_asked: Dword,
    /// +0x2c (0x5C2544): `g_lastRaceViewed`: the slot is 222 (`race::viewer`).
    pub last_race_viewed: Flag,
    /// +0x30 (0x5C2548): `g_viewerOnlyShown`: modes 4 and 5 (`race::viewer`).
    pub viewer_only_shown: Dword,
    /// +0x34 (0x5C254C): `g_viewerKeysIgnored`: mode 5 (`race::viewer`).
    pub viewer_keys_ignored: Dword,
    /// +0x38 (0x5C2550): `g_keyAfterPlay`; image value -1 (`race::viewer`).
    pub key_after_play: Dword,
    /// +0x3c (0x5C2554): `g_waitKeyRelease`: keys held after playing (`race::viewer`).
    pub wait_key_release: Dword,
    /// +0x40 (0x5C2558): `g_escapeCount`: Escape twice leaves (`race::viewer`).
    pub escape_count: Dword,
    /// +0x44 (0x5C255C): `g_viewerList`, taken in place: header 0x18 bytes (byte +1 = item count, set at run time; +0x10 a pointer to the key table), then 26 items of 16 bytes (flags, source rectangle pointer, 0, screen rectangle pointer; relocated pointers) (`race::viewer_frame`, `race::mark_viewer_items`, `race::draw_viewer_item`).
    pub viewer_list: [Dword; 110],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
impl ReplayViewBlock {
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

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
    /// (untyped, the race block's bytes: `OWN_RANGES[1]`), and the panel with the pictures'
    /// rectangles.
    const RECTANGLES: core::ops::Range<usize> = 0x5C2714..0x5C291C;
    const PANEL: core::ops::Range<usize> =
        crate::race::viewer_screen_store::START..crate::race::viewer_screen_store::END;

    /// Where IDA's name `ida` is now, if it lies in the rectangles: a place in the race
    /// block's range of them, running on to that range's end (phase 13: the list keeps IDA's
    /// addresses as names).
    fn named(ida: u32, what: &'static str) -> Result<BytePlace, crate::errors::BadInput> {
        if !Self::RECTANGLES.contains(&(ida as usize)) {
            return Err(crate::errors::BadInput(what));
        }
        Ok(BytePlace::Store(
            &crate::race::store::OWN_RANGES[1],
            ida as usize - Self::RECTANGLES.start,
        ))
    }

    /// Where the list's key table (the header's +0x10) is.
    pub fn key_table() -> Result<BytePlace, crate::errors::BadInput> {
        Self::named(
            race().replay.replay_view.viewer_list[4].get(),
            "the viewer list's key table off its place",
        )
    }

    /// Where item `index`'s screen rectangle (its +4) is.
    pub fn item_place(index: u32) -> Result<BytePlace, crate::errors::BadInput> {
        Self::named(
            Self::item_dword(index, 1)?,
            "a viewer item's place off the rectangles",
        )
    }

    /// What item `index`'s picture (its +0xC: a rectangle, or one of the switch tables) is.
    /// The original compared the picture's address with the three tables' addresses.
    pub fn item_picture(index: u32) -> Result<ViewerPicture, crate::errors::BadInput> {
        let ida = Self::item_dword(index, 3)?;
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

    fn item_dword(index: u32, field: usize) -> Result<u32, crate::errors::BadInput> {
        if index >= Self::ITEMS {
            return Err(crate::errors::BadInput("a viewer item past the 26"));
        }
        Ok(race().replay.replay_view.viewer_list[6 + 4 * index as usize + field].get())
    }
}

/// IDA 0x5C2518, the first byte of the piece.
pub const START: usize = 0x5C2518;
/// IDA 0x5C2714, the end of the piece.
pub const END: usize = 0x5C2714;

const _: () = assert!(END - START == core::mem::size_of::<ReplayViewBlock>());

/// Where IDA 0x5C2518 is now (its own static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(&race().replay.replay_view))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &race().replay.replay_view,
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
        assert_eq!(replay.replay_view.viewer_list[4].get(), name(key_table));
        for k in 0..26u32 {
            let item = 0x18 + 0x10 * k as usize;
            // Kept as the image's names; resolved to the same places.
            assert_eq!(
                replay.replay_view.viewer_list[6 + 4 * k as usize + 1].get(),
                name(item + 4)
            );
            assert_eq!(
                replay.replay_view.viewer_list[6 + 4 * k as usize + 3].get(),
                name(item + 0xC)
            );
            match ReplayViewBlock::item_picture(k).unwrap() {
                ViewerPicture::Rectangle(ida) => assert_eq!(ida, name(item + 0xC) as usize),
                ViewerPicture::Switch(table) => assert!(table < 3),
            }
        }
        assert!(ReplayViewBlock::item_place(26).is_err());
    }

    #[test]
    fn fields_are_at_the_games_offsets() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(offset_of!(ReplayViewBlock, pad_frame), 0x0);
        assert_eq!(offset_of!(ReplayViewBlock, play_from_here), 0x24);
        assert_eq!(offset_of!(ReplayViewBlock, wait_key_release), 0x3c);
    }
}
