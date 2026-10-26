//! `menu::section_finish` (0x4A9439): the championship finish routine for the first player
//! done with the section (`menu::find_section_player`; none: nothing but a key taken):
//! `dword_75B108` and `dword_75ACE4` 0, the race's set-up from it
//! (`finish_calls::set_up`), then the sounds' and screens' set-up, `drive::finish_race_0`,
//! and the front end's set-up again (`finish_calls::CALLED` 1 .. 13); a key taken at the
//! end (eax 0).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A0E60` (twice). The finding is its Rust version; the calls through
//! `part`, in the game through their addresses (hooks where ported). Ungated (the finish
//! routine runs). Refuses what the finding refuses, before anything. Shadow mode runs the
//! original alone. Self-test: a copy whose calls go to stand-ins, against the Rust
//! version, the done player by lot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::finish_calls::{Game, Part, game_part};
use rc2k_core::menu::finish;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn section_finish(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::finish::section_finish`.
    finish::section_finish(&mut Game { g: *entry, part })?;
    Ok(*entry)
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { section_finish(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
