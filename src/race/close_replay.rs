//! `race::close_replay` (0x41E85A): with the replay open, the block filled up with the
//! padding frame (`race::add_replay_frame`), the last block's frame count, the time
//! sealed, the index ("RCIB") written, the file closed; `dword_5BEC88` 0,
//! `dword_5BEC84` 1.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/replay.rs`. Callers: `race::open_replay`, `race::write_replay_block`
//! (their failures), `sub_443F0B`, `sub_443F96` (none reads its flags). Shadow mode
//! runs the original alone (it writes a file). Self-test: a copy whose calls go to
//! stand-ins, against the Rust version on the same: closed, open with frames and
//! without, the last block or not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::replay::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { close_replay(r, &mut game_part) }?;
        *r = Regs { esp: r.esp, ..g }
    }
    Ok(())
}
