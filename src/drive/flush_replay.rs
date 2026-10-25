//! `drive::flush_replay` (0x46022D): the replay's starting state (0xA04 dwords at
//! `unk_7172EC`) copied into the frame `dword_9613A4` and the frames written
//! (`drive::write_replay_frames`).
//!
//! In:  nothing
//! Out: ecx 0, esi and edi past the copy; the rest kept; no flags read
//!
//! Caller: `drive::flush_replay_if`. Refuses what `write_replay_frames` refuses.

use crate::errors::Refused;

const DWORDS: usize = 0xA04;

/// The Rust version.
pub unsafe fn flush_replay() -> Result<(), Refused> {
    unsafe {
        std::ptr::copy(
            crate::race::blocks_store::block_addr(8) as *const u32,
            crate::race::viewer_buffers_store::VIEWER_BUFFERS.block[0].addr() as *mut u32,
            DWORDS,
        )
    };
    unsafe { crate::drive::write_replay_frames::write_replay_frames() }
}
