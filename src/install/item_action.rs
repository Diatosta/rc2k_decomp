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
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{item_action_at, rd};

/// Enter on an action item; the original's zero after it (the action's, else that of
/// `cmp [esi+8], 1`).
pub fn item_action(item: u32) -> Result<bool, Refused> {
    let action = rd(item + 8);
    // Converted: `rc2k_core::install::items::has_action`.
    if rc2k_core::install::items::has_action(action) {
        return item_action_at(action);
    }
    Ok(action == 1)
}
