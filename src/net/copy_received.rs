//! Under the received messages' mutex, copies each player's received messages for
//! `net::apply_received`, then resets the count and moves the newest sequence to the oldest. The
//! loop runs at least once. Refuses more than the 8 players there are blocks for.

use crate::errors::{BadInput, Refused};
use crate::net::store::NetShared;
use crate::net::{lock_received, unlock_received};
use rc2k_core::net::inbox;

/// The blocks' room: 8 players.
pub const MOST_PLAYERS: u32 = rc2k_core::net::players::MOST;
pub fn copy_received(vars: &NetShared) -> Result<(), Refused> {
    let count = || vars.players.get();
    if count() > MOST_PLAYERS {
        return Err(BadInput("more than the blocks' 8 players").into());
    }
    lock_received::lock_received(vars);
    let mut player = 0u32;
    loop {
        // The logic is in `rc2k_core::net::inbox::hand_over`, on the inbox.
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
