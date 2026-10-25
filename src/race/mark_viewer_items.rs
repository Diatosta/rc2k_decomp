//! `race::mark_viewer_items` (0x41D456): the replay viewer's items marked for what can be
//! done now: the marks cleared (`menu::list_clear_marks`), then bit 6 of item 0x14 when
//! the replay is the last race's (`dword_5C2544`), bit 7 (not usable) of: 0xD .. 0x10 and
//! 0x17 .. 0x19 at the last block; 0x11 when the marks are not in order; 0x12 with an
//! empty clipboard or one too big to paste (count + clipboard past 0x708); 0x13 with
//! nothing to undo; 0xA at or before the first mark, 2 at or past the second; with no
//! blocks 0 .. 0x11 and 0x15 .. 0x19; 0x18, 0x19 when the block's camera byte 5 is 0, 0xD,
//! 0xE when it is 1; 0xF for one player's replay (`dword_719B34`).
//!
//! In:  edi the list (`unk_5C255C`)
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Callers: `race::replay_viewer` (two).

use crate::race::replay_file_store::REPLAY_FILE;
use crate::race::replay_work_store::REPLAY_WORK;
use crate::race::viewer::{CAMERAS, CLIP_BLOCKS, MARK_IN, MARK_OUT, UNDO_BLOCKS, gb, get};
use crate::race::viewer_state_store::REPLAY_VIEW;

/// The Rust version.
pub unsafe fn mark_viewer_items(list: u32) {
    // Converted: `rc2k_core::race::viewer::item_marks`.
    unsafe { crate::menu::list_clear_marks::list_clear_marks(list) };
    let block = REPLAY_VIEW.replay_block.get();
    let state = rc2k_core::race::viewer::ViewerState {
        last_race: REPLAY_VIEW.last_race_viewed.get() == 1,
        block,
        blocks: REPLAY_FILE.index_block_count.get(),
        mark_in: get(MARK_IN),
        mark_out: get(MARK_OUT),
        clip_blocks: get(CLIP_BLOCKS),
        undo_blocks: get(UNDO_BLOCKS),
        camera: gb((CAMERAS + 5).wrapping_add(block.wrapping_mul(8) as usize)),
        one_player: REPLAY_WORK.header_one_player.get() == 1,
    };
    for (item, bit) in rc2k_core::race::viewer::item_marks(&state) {
        let at = (list + 0x18 + 0x10 * item) as *mut u8;
        unsafe { at.write(at.read() | bit) };
    }
}
