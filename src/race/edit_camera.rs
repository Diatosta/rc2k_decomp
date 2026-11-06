//! The two "edit a camera" actions of the replay viewer, and the camera changes they lead
//! to (IDA 2026-10-19):
//! - `race::edit_camera_1` (0x41E599), `race::edit_camera_2` (0x41E5BA): the replay kept
//!   for an undo (`race::keep_undo`); unless the block shown (`dword_5C2530`) is the last
//!   (`dword_5BECD0`), `dword_96E480` 1 or 2: the keys then change that camera.
//! - `race::next_camera` (0x41E5EB), `race::previous_camera` (0x41E654): unless the block
//!   shown is the last, its camera byte `dword_96E480 - 1` (`unk_963BB3[edit + block * 8]`)
//!   one on (past 0x2E back to 0) or back (below 0 to 0x2E), skipping the cameras whose
//!   name (`word_5C291C`) is 0x1C1 or 0x1C2 unless the replay is one player's
//!   (`dword_719B34` 1) with a first car outside 0x17 .. 0x20; then the change spread on
//!   (`race::spread_camera`, dh the old value).

use crate::errors::Refused;
use crate::race::owner_replay::Replay;
use crate::race::replay_work_store::REPLAY_WORK;

/// `edit_camera_1` / `_2` (eax the block shown is left; its caller, `viewer_action`,
/// keeps every register).
pub fn edit_camera(replay: &Replay, camera: u32) {
    // Converted: `rc2k_core::race::viewer_flow::edits_camera`.
    crate::race::keep_undo::keep_undo();
    if rc2k_core::race::viewer_flow::edits_camera(
        replay.replay_view.replay_block.get(),
        crate::race::owner::race()
            .shared
            .replay_file
            .index_block_count
            .get(),
    ) {
        crate::race::viewer_buffers_store::VIEWER_BUFFERS
            .camera_edit
            .set(camera);
    }
}

fn name_of(camera: u8) -> u16 {
    crate::race::viewer_screen_store::VIEWER_PANEL.camera_name(camera as usize)
}

/// `next_camera` (`step` 1) and `previous_camera` (-1). What they leave their caller,
/// `viewer_action` (which keeps every register), does not read.
pub fn step_camera(replay: &Replay, step: i8) -> Result<(), Refused> {
    let block = replay.replay_view.replay_block.get();
    if block
        == crate::race::owner::race()
            .shared
            .replay_file
            .index_block_count
            .get()
    {
        return Ok(());
    }
    let at = crate::race::viewer::camera_ptr(
        0u32.wrapping_sub(1)
            .wrapping_add(
                crate::race::viewer_buffers_store::VIEWER_BUFFERS
                    .camera_edit
                    .get(),
            )
            .wrapping_add(block.wrapping_mul(8)),
    );
    let old: u8 = at.read("a replay camera byte")?;
    // Converted: `rc2k_core::race::viewer::{every_camera, step_camera}`.
    use rc2k_core::race::viewer::{camera_skipped, every_camera, step_camera};
    let every = every_camera(
        REPLAY_WORK.header_one_player.get() == 1,
        REPLAY_WORK.header_kind_0.get(),
    );
    let dl = step_camera(old, step > 0, every, |n| camera_skipped(name_of(n)))?;
    at.write(dl, "a replay camera byte")?;
    let spread = crate::race::spread_camera::spread_camera(replay, at, old);
    if spread.is_err() {
        at.write(old, "a replay camera byte")?; // refused before it wrote: nothing left written
    }
    spread
}
