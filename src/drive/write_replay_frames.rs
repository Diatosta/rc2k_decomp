//! `drive::write_replay_frames` (0x41E930): while a replay is written to its file
//! (`dword_5BEC88` 1): `dword_5BECD4` + 100, the frame's head (`dword_9613A4`, `9613A8`)
//! 0, its byte +5 held to 2 ..= 3 (signed) unless `dword_719B34` is 1,
//! `race::write_replay_block` with `dword_5C252C` 100 round it, and `dword_9613A0` the
//! frame's address. All in `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::flush_replay`.
//! Refuses what `write_replay_block` refuses. Shadow mode runs the original alone (a file
//! write).
//! Self-test: the original and the Rust version with the replay not being written (its
//! writing is `race::write_replay_block`'s test).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::{a, g, p};
use rc2k_core::drive::replay_frames::written_view;

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::written_view`.
pub unsafe fn write_replay_frames(entry: &Regs) -> Result<(), Refused> {
    if g(0x5BEC88) != 1 {
        return Ok(());
    }
    p(0x5BECD4, g(0x5BECD4).wrapping_add(0x64));
    p(0x5C252C, 0x64);
    p(0x9613A4, 0);
    p(0x9613A8, 0);
    if g(0x719B34) != 1 {
        let at = a(0x9613A9) as *mut i8;
        let v = written_view(unsafe { at.read() });
        unsafe { at.write(v) };
    }
    unsafe { crate::race::replay::write_replay_block(entry, &mut crate::race::replay::game_part) }?;
    p(0x9613A0, a(0x9613A4));
    p(0x5C252C, 0);
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { write_replay_frames(r) }?;
    Ok(())
}
