//! `drive::load_replay_frame` (0x46020A): the replay's frame `dword_6EF7E0` read
//! (`drive::read_replay_frame_0`), its marks (`drive::replay_frame_marks`), and the frame
//! (0xA04 dwords at `dword_9613A4`) copied to the replay's starting state `unk_7172EC`.
//!
//! In:  nothing
//! Out: eax the frame, ecx 0, esi and edi past the copy; the rest kept; no flags read
//!
//! Caller: `drive::play_replay_frame`. Refuses what `read_replay_frame` refuses.

use crate::drive::replay_frames_store::REPLAY_FRAMES;
use crate::errors::Refused;

const DWORDS: usize = 0xA04;

/// The Rust version.
pub unsafe fn load_replay_frame() -> Result<(), Refused> {
    let frame = REPLAY_FRAMES.step_block.get();
    unsafe { crate::drive::read_replay_frame_0::read_replay_frame_0(frame) }?;
    crate::drive::replay_frame_marks::replay_frame_marks();
    unsafe {
        std::ptr::copy(
            crate::race::viewer_buffers_store::VIEWER_BUFFERS.block[0].addr() as *const u32,
            crate::race::blocks_store::block_addr(8) as *mut u32,
            DWORDS,
        )
    };
    Ok(())
}
