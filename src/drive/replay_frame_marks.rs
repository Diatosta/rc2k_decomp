//! `drive::replay_frame_marks` (0x41D8A3): the replay frame `dword_5C2530`'s two marks
//! (`unk_963BB4[frame]`, 8 bytes) into `dword_9613A4` .. (in `pusha` / `popa`).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Callers: `sub_41D5FB`, `sub_41D846`, `sub_46020A`.

use crate::errors::BadInput;
use crate::pointers::GamePtr;
use crate::race::viewer_state_store::REPLAY_VIEW;

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::marks_offset`.
pub fn replay_frame_marks() -> Result<(), BadInput> {
    let at = (crate::race::viewer_buffers_store::VIEWER_BUFFERS
        .block_cameras
        .addr())
    .wrapping_add(rc2k_core::drive::replay_frames::marks_offset(
        REPLAY_VIEW.replay_block.get(),
    ));
    let marks: [u32; 2] = GamePtr::new(at).read("the replay frame's marks")?;
    for (k, mark) in marks.into_iter().enumerate() {
        crate::race::viewer_buffers_store::VIEWER_BUFFERS.block[k].set(mark);
    }
    Ok(())
}
