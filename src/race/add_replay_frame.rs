//! `race::add_replay_frame` (0x41E8DE): with the replay open, ecx bytes at ebx (a frame
//! when 0x10: counted) put at the block's cursor; at 100 frames the block written
//! (`race::write_replay_block`), the cursor and count back to the start.
//!
//! In:  ebx = the bytes, ecx = their count
//! Out: every register preserved
//!
//! See `race/replay.rs`. One caller in code (`race::close_replay`, its padding). Refuses
//! bytes past the block's room (before anything; the original writes on). Shadow mode
//! runs the original alone (it may write a file). Self-test: a copy whose call goes to a
//! stand-in, against the Rust version on the same: closed, a frame, other sizes, the
//! hundredth frame.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::replay::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { add_replay_frame(r) }?;
        *r = Regs { esp: r.esp, ..g }
    }
    Ok(())
}
