//! `race::reset_marks` (0x41E403): after an edit, the block shown held to the replay's
//! blocks (`dword_5C2530` at most `dword_5BECD0`, signed), the marks round the whole
//! replay (`dword_96E478` 0, `dword_96E47C` the count) and the block shown read again
//! (`drive::read_replay_frame_0`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Callers: `race::viewer_action` (the cut and the paste), `race::undo_edit`. Refuses what
//! `read_replay_frame_0` refuses (what it wrote put back). Shadow mode compares (the
//! file put back where it was by `read_packed`). Self-test: the original against the
//! Rust version with `read_replay_frame`'s test replay: blocks below, at and past the
//! count, not playing.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::viewer::{BLOCK, BLOCKS, MARK_IN, MARK_OUT, get, put, regs_entry};

/// The Rust version.
pub fn reset_marks() -> Result<(), Refused> {
    {
        // Converted: `rc2k_core::race::viewer::reset_marks`.
        let (block, mark_in, mark_out) =
            rc2k_core::race::viewer::reset_marks(get(BLOCK), get(BLOCKS));
        put(BLOCK, block);
        put(MARK_IN, mark_in);
        put(MARK_OUT, mark_out);
        unsafe { crate::drive::read_replay_frame_0::read_replay_frame_0(get(BLOCK)) }?;
        Ok(())
    }
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    reset_marks()?;
    Ok(*r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &rust)
}
