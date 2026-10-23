//! `race::write_replay_block` (0x41E9A3): the replay's block (`dword_9613A4`, 0x2810
//! bytes) written by `race::write_replay_chunk`, its offset and size into the index
//! ("RCIB"), the file's length and the block count on; a failed write or the index full
//! (0x708 blocks) closes the replay (`race::close_replay`) and sets carry.
//!
//! In:  nothing
//! Out: every register preserved; carry set for the failure
//!
//! See `race/replay.rs`. Callers: `race::add_replay_frame` and `sub_41E930` (neither
//! reads its flags). Refuses an index full already (before anything; the original writes
//! past it). Shadow mode runs the original alone (it writes a file). Self-test: a copy
//! whose calls go to stand-ins (the chunk packed, plain, failing; the index filling up),
//! against the Rust version on the same.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::replay::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { write_replay_block(r) }?;
        *r = Regs { esp: r.esp, ..g }
    }
    Ok(())
}
