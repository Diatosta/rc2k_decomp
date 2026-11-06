//! `drive::replay_frame_marks` (0x41D8A3): the replay frame `dword_5C2530`'s two marks
//! (`unk_963BB4[frame]`, 8 bytes) into `dword_9613A4` .. (in `pusha` / `popa`).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Callers: `sub_41D5FB`, `sub_41D846`, `sub_46020A`.

use crate::errors::BadInput;
use crate::pointers::GamePtr;
use crate::race::owner::race;

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::marks_offset`.
pub fn replay_frame_marks() -> Result<(), BadInput> {
    let replay = &race().replay;
    let at = GamePtr::to(&crate::race::viewer_buffers_store::VIEWER_BUFFERS.block_cameras).offset(
        rc2k_core::drive::replay_frames::marks_offset(replay.replay_view.replay_block.get()),
    );
    let marks: [u32; 2] = at.read("the replay frame's marks")?;
    for (k, mark) in marks.into_iter().enumerate() {
        crate::race::viewer_buffers_store::VIEWER_BUFFERS.block[k].set(mark);
    }
    Ok(())
}
