//! `race::write_replay_chunk` (0x41EEE4): ecx bytes at ebx written to file eax, packed
//! by runs (`race::pack_runs` into `g_pngInfo`, rounded up to a dword) when that is
//! smaller and `dword_5BEC80` is not 1 (eax then the bytes written negated).
//!
//! In:  eax = the file, ebx = the bytes, ecx = their count
//! Out: eax = the bytes written (negated when packed), carry set for a failed write;
//!      the rest preserved
//!
//! See `race/replay.rs`. Callers: `race::write_replay_block` and `sub_41ED1C` (each
//! `jb` after it: dispatch leaves the carry). With `dword_5BEC80` 1 the original jumps
//! to `files::write` (a tail call). Shadow mode runs the original alone (it writes a
//! file). Self-test: a copy whose calls and jump go to stand-ins (a write that works or
//! fails, packings smaller and not), against the Rust version on the same.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::replay::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { write_replay_chunk(r, &mut game_part) }?;
        *r = Regs { esp: r.esp, ..g }
    }
    Ok(())
}
