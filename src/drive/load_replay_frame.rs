//! `drive::load_replay_frame` (0x46020A): the replay's frame `dword_6EF7E0` read
//! (`drive::read_replay_frame_0`), its marks (`drive::replay_frame_marks`), and the frame
//! (0xA04 dwords at `dword_9613A4`) copied to the replay's starting state `unk_7172EC`.
//!
//! In:  nothing
//! Out: eax the frame, ecx 0, esi and edi past the copy; the rest kept; no flags read
//!
//! Caller: `drive::play_replay_frame`. Refuses what `read_replay_frame` refuses.

use crate::drive::read_replay_frame::FRAME;
use crate::errors::Refused;
use crate::physics::flow::{a, g};

const DWORDS: usize = 0xA04;

/// The Rust version.
pub unsafe fn load_replay_frame() -> Result<(), Refused> {
    let frame = g(0x6EF7E0);
    unsafe { crate::drive::read_replay_frame_0::read_replay_frame_0(frame) }?;
    crate::drive::replay_frame_marks::replay_frame_marks();
    unsafe {
        std::ptr::copy(
            a(FRAME) as *const u32,
            crate::race::blocks_store::block_addr(8) as *mut u32,
            DWORDS,
        )
    };
    Ok(())
}
