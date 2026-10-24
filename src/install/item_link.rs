//! `install::item_link` (0x414A28): Enter on a link item (kind 1): its action (`[esi+8]`,
//! edx its `[esi+0Ch]`) if it has one; else, unless its next list (`[esi+1]`) is negative,
//! that number into `byte_51D415` and the page left (`g_listLeave` 1).
//!
//! In:  esi the item (edi the list, for the action)
//! Out: with an action as it leaves them; else bl the next list and the flags of
//!      `test bl, bl` (zero for list 0, carry clear)
//! Caller `install::item_enter`. Ungated: an action can be a whole page's work. Shadow mode
//! runs the original alone. Self-test: a copy, the action a noting stand-in, against the
//! Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{LIST_ANSWER, LIST_LEAVE, item_action_at, pb, rb, rd};
use rc2k_core::install::items::{self, LinkWay};

/// Enter on a link; the original's zero after it (the action's, else that of `test bl, bl`
/// on the next list).
pub fn item_link(item: u32) -> Result<bool, Refused> {
    let action = rd(item + 8);
    let mut next = 0;
    // Converted: `rc2k_core::install::items::link`.
    match items::link(action, || {
        next = rb(item + 1);
        next
    }) {
        LinkWay::Action => item_action_at(action),
        LinkWay::Leave(next) => {
            pb(LIST_ANSWER, next);
            pb(LIST_LEAVE, 1);
            Ok(next == 0)
        }
        LinkWay::Stay => Ok(next == 0),
    }
}
