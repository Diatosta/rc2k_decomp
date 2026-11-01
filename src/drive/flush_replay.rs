//! `drive::flush_replay` (0x46022D): the replay's starting state (0xA04 dwords at
//! `unk_7172EC`) copied into the frame `dword_9613A4` and the frames written
//! (`drive::write_replay_frames`).
//!
//! In:  nothing
//! Out: ecx 0, esi and edi past the copy; the rest kept; no flags read
//!
//! Caller: `drive::flush_replay_if`. Refuses what `write_replay_frames` refuses.

use crate::errors::Refused;
use crate::pointers::GamePtr;

const DWORDS: usize = 0xA04;

/// The Rust version.
pub fn flush_replay() -> Result<(), Refused> {
    GamePtr::new(crate::race::blocks_store::block_addr(8)).copy_to(
        GamePtr::to(&crate::race::viewer_buffers_store::VIEWER_BUFFERS.block),
        DWORDS * 4,
        "a car's block",
    )?;
    crate::drive::write_replay_frames::write_replay_frames()
}
