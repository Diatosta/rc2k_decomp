//! `menu::redefine_key` (0x47D472): one frame of redefining a key of the current player.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `menu/redefine_keys.rs`; the player is `dword_75598C`'s. In the draw lists of the
//! redefine keys pages of players 2, 3 and 4 (0x758DEE, 0x758DFA, 0x758E06). The function
//! `sub_47D4DE` (where the key goes) is inlined.
//! Refuses, before the key is taken: no current player, a place of 12 or more, a reserved
//! list without its end.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::menu::redefine_keys::rust;
use crate::service::harness::state_entry;

/// The current player.
fn player() -> u32 {
    crate::menu::player_settings::current().map_or(0, |p| p as u32)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust(player))
}
