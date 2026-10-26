//! `menu::clear_players_288` (0x4A969C): the four players' +0x288 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_473126`. Refuses a missing player, before anything.

use crate::errors::{BadInput, Refused};
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub fn clear_players_288() -> Result<(), Refused> {
    if (0..4).any(|n| player(n) == 0) {
        return Err(BadInput("a missing player").into());
    }
    for n in 0..4 {
        put(player(n) + 0x288, 0)?;
    }
    Ok(())
}
