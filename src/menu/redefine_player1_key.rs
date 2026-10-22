//! `menu::redefine_player1_key` (0x47D3D8): one frame of redefining a key of player 1.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `menu/redefine_keys.rs`; the player is the first of `g_players`, whatever `dword_75598C`
//! says. In the draw list of player 1's redefine keys page (0x758DE2); IDA had not made it a
//! function. `sub_47D43E` (where the key goes, the same as `sub_47D4DE`) is inlined. Refuses,
//! before the key is taken: no player 1, a place of 12 or more, a reserved list without its
//! end. Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::menu::redefine_keys::rust;
use crate::service::harness::state_entry;

/// Player 1.
fn player() -> u32 {
    crate::menu::player::player_at(0)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust(player))
}
