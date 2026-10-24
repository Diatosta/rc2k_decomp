//! `drive::flush_replay` (0x46022D): the replay's starting state (0xA04 dwords at
//! `unk_7172EC`) copied into the frame `dword_9613A4` and the frames written
//! (`drive::write_replay_frames`).
//!
//! In:  nothing
//! Out: ecx 0, esi and edi past the copy; the rest kept; no flags read
//!
//! Caller: `drive::flush_replay_if`.
//! Refuses what `write_replay_frames` refuses. Shadow mode runs the original alone (a
//! file write).
//! Self-test: the original and the Rust version with the replay not being written.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::physics::flow::a;

const DWORDS: usize = 0xA04;

/// The Rust version.
pub unsafe fn flush_replay() -> Result<(), Refused> {
    unsafe { std::ptr::copy(a(0x7172EC) as *const u32, a(0x9613A4) as *mut u32, DWORDS) };
    unsafe { crate::drive::write_replay_frames::write_replay_frames() }
}
