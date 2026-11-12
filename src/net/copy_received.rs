//! `net::copy_received` (0x4173A0): under the mutex of the received messages
//! (`net::lock_received` / `net::unlock_received`, `mutex`), each player's
//! messages as the receiving thread left them (its block in `received`: the count and the
//! records) copied to `copied_count` and `copied` for `net::apply_received`, the count made 0
//! and the newest sequence copied to the oldest.
//! The loop runs at least once (player 0 with no players).
//!
//!
//! See `net/mod.rs`. Two callers: `net::apply_received` and `net::lets_rally`. Refuses
//! (before anything) more than the 8 players there are blocks for.

use crate::errors::{BadInput, Refused};
use crate::net::store::NetVars;
use crate::net::{lock_received, unlock_received};
use rc2k_core::net::inbox;

/// The blocks' room: 8 players.
pub const MOST_PLAYERS: u32 = rc2k_core::net::players::MOST;
pub fn copy_received(vars: &NetVars) -> Result<(), Refused> {
    let count = || vars.players.get();
    if count() > MOST_PLAYERS {
        return Err(BadInput("more than the blocks' 8 players").into());
    }
    lock_received::lock_received(vars);
    let mut player = 0u32;
    loop {
        // Converted: `rc2k_core::net::inbox::hand_over`, on the inbox.
        let at = vars.player_block(player)?;
        let mut inbox = at.inbox();
        let (handed, records) = inbox::hand_over(&mut inbox);
        at.copied_count.set(handed);
        at.copied
            .write_at(0, &records, "a player's copied records")?;
        at.count.set(inbox.count);
        at.oldest.set(inbox.oldest);
        player += 1;
        if player >= count() {
            break;
        }
    }
    unlock_received::unlock_received(vars);
    Ok(())
}
