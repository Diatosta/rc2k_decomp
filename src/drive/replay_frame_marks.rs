//! `drive::replay_frame_marks` (0x41D8A3): the replay frame `dword_5C2530`'s two marks
//! (`unk_963BB4[frame]`, 8 bytes) into `dword_9613A4` .. (in `pusha` / `popa`).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Callers: `sub_41D5FB`, `sub_41D846`, `sub_46020A`.

use crate::race::viewer_state_store::REPLAY_VIEW;

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::marks_offset`.
pub fn replay_frame_marks() {
    let at = (crate::race::viewer_buffers_store::VIEWER_BUFFERS
        .block_cameras
        .addr() as u32)
        .wrapping_add(rc2k_core::drive::replay_frames::marks_offset(
            REPLAY_VIEW.replay_block.get(),
        ));
    for k in 0..2u32 {
        crate::race::viewer_buffers_store::VIEWER_BUFFERS.block[k as usize]
            .set(unsafe { ((at + 4 * k) as *const u32).read_unaligned() });
    }
}
