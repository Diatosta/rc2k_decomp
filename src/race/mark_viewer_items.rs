//! `race::mark_viewer_items`: marks the replay viewer's items for what can be done now: the marks
//! cleared (`menu::list_clear_marks`), then bit 6 of item 0x14 when the replay is the last race's
//! (`last_race_viewed`), bit 7 (not usable) of: 0xD .. 0x10 and 0x17 .. 0x19 at the last block;
//! 0x11 when the marks are not in order; 0x12 with an empty clipboard or one too big to paste
//! (count + clipboard past 0x708); 0x13 with nothing to undo; 0xA at or before the first mark, 2 at
//! or past the second; with no blocks 0 .. 0x11 and 0x15 .. 0x19; 0x18, 0x19 when the block's
//! camera byte 5 is 0, 0xD, 0xE when it is 1; 0xF for a one-player replay with a single car.
//!
//! Takes the list (`viewer_list`).

use crate::race::owner::Race;

pub(super) fn mark_viewer_items(
    race: &Race,
    list: crate::memory::list_handle::ListMenu,
) -> Result<(), crate::errors::BadInput> {
    let replay = &race.replay;
    // Logic in `rc2k_core::race::viewer::item_marks`.
    crate::menu::list_clear_marks::list_clear_marks(list);
    let block = replay.replay_view().replay_block.get();
    let state = rc2k_core::race::viewer::ViewerState {
        last_race: replay.replay_view().last_race_viewed.is_set(),
        block,
        blocks: race.shared.replay_file().index.block_count.get(),
        mark_in: race.viewer_buffers().mark_in.get(),
        mark_out: race.viewer_buffers().mark_out.get(),
        clip_blocks: race.viewer_buffers().clip_blocks.get(),
        undo_blocks: race.viewer_buffers().undo_blocks.get(),
        camera: crate::race::viewer::camera_byte_at(
            race,
            5usize.wrapping_add(block.wrapping_mul(8) as usize),
        )?,
        one_player: !crate::race::viewer_action::views_choosable(race),
    };
    for (item, bit) in rc2k_core::race::viewer::item_marks(&state) {
        let item = list.item(item);
        item.set_first(item.first() | bit);
    }
    Ok(())
}
