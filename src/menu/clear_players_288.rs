//! `menu::clear_players_288` (0x4A969C): the four players' +0x288 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_473126`. Refuses a missing player, before anything.

use crate::errors::Refused;
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub fn clear_players_288() -> Result<(), Refused> {
    for n in 0..4 {
        player(n).update(|q| q.store(&[(0x288, 0)]));
    }
    Ok(())
}
