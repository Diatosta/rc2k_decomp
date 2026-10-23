//! `race::release_stage_textures` (0x44E79F): `gfx::release_all_textures` (0x4051D3, the
//! platform's: called in the game); a `call` and a `retn`.
//!
//! In:  nothing
//! Out: the registers as `gfx::release_all_textures` leaves them
//!
//! See `race/race_shell.rs`. Callers: `sub_444940` (`pushf` follows), `sub_443396` (a
//! call follows) and code at 0x44336D (a call follows): no flags read. It begins with its
//! call: private copy, the call relocated, manual patch. Shadow mode runs the original
//! alone (it releases the textures). Self-test: a copy whose call goes to a stand-in that
//! notes it, against the Rust version on the same.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::race_shell::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let g = unsafe { release_stage_textures(r, &mut game_release_textures) };
    *r = Regs { esp: r.esp, ..g };
    Ok(())
}
