//! `install::item_action` (0x414A4B): Enter on an action item (kind 6): its action
//! (`[esi+8]`), if it has one.
//!
//! In:  esi the item (edi the list, for the action)
//! Out: with an action as it leaves them; else the flags of `cmp [esi+8], 1` (carry set)
//! Caller `install::item_enter`. Ungated: the installer's actions are whole pages' work
//! (`install::install_files`). Shadow mode runs the original alone. Self-test: a copy, the
//! action a noting stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{Part, cmp, rd};
use crate::race::viewer::regs_entry;

pub fn item_action(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let action = rd(r.esi + 8);
    cmp(&mut r, action, 1);
    // Converted: `rc2k_core::install::items::has_action`.
    if rc2k_core::install::items::has_action(action) {
        part(action, &mut r)?;
    }
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        item_action(r, &mut crate::install::installer::game_part)
    })
}
