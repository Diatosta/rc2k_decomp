//! The replay writer's and the viewer's state (IDA 0x5C2518 .. 0x5C2714; phase 7, typed): the padding frame, the file offset and frames in the block, the replay block and frame, the play end and the viewer's switches, then the viewer's list (header and 26 items of 16 bytes). `#[repr(C)]` with the image's layout, over the race block's bytes. The list's pointers are relocated addresses into the untyped rectangles and key table at 0x5C2714 .. 0x5C291C. Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
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
    pub last_race_viewed: Dword,
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

/// The piece: the race block's bytes at IDA 0x5C2518 seen as `ReplayViewBlock`. They stay there
/// while code reaches them by RVA from `race::store::asm_base()` and by IDA address
/// (`moved::place`): one copy for all three.
pub struct InRaceBlock;

impl core::ops::Deref for InRaceBlock {
    type Target = ReplayViewBlock;
    fn deref(&self) -> &ReplayViewBlock {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); every field is an atomic or a cell, as the block's bytes were shared.
        unsafe { &*(crate::pointers::host_at(piece_addr()) as *const ReplayViewBlock) }
    }
}

pub static REPLAY_VIEW: InRaceBlock = InRaceBlock;

/// IDA 0x5C2518, the first byte of the piece.
pub const START: usize = 0x5C2518;
/// IDA 0x5C2714, the end of the piece.
pub const END: usize = 0x5C2714;

const _: () = assert!(END - START == core::mem::size_of::<ReplayViewBlock>());

/// Where IDA 0x5C2518 is now (in the race block).
pub fn piece_addr() -> usize {
    crate::race::store::home(START)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_race_block() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(offset_of!(ReplayViewBlock, pad_frame), 0x5C2518 - START);
        assert_eq!(
            offset_of!(ReplayViewBlock, play_from_here),
            0x5C253C - START
        );
        assert_eq!(
            offset_of!(ReplayViewBlock, wait_key_release),
            0x5C2554 - START
        );
    }
}
