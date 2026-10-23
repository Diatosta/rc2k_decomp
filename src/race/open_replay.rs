//! `race::open_replay` (0x41E6EC): with replays on and 30 MB free, the replay's header
//! made ("REPL", 0x50400, the date, "tmpreplay", the settings, the sealed part), the
//! index and block cleared, `var\\tmprpl` created and the header written; a failed open
//! or write closes it and sets carry. `dword_5BEC84` 0 first.
//!
//! In:  nothing
//! Out: every register preserved; carry set for the failure
//!
//! See `race/replay.rs`. Callers: `sub_443F0B`, `sub_443F96` (a `mov` follows; no
//! flags read). Shadow mode runs the original alone (it creates a file). Self-test: a
//! copy whose calls go to stand-ins (the free space, the time, the header's seal, the
//! open and the write that work or fail), against the Rust version on the same; replays
//! off each way.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::replay::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { open_replay(r) }?;
        *r = Regs { esp: r.esp, ..g }
    }
    Ok(())
}
