//! Under the received messages' lock (the guard is passed in), copies each player's received
//! messages for `net::apply_received`, then resets the count and moves the newest sequence to
//! the oldest. The loop runs at least once. Refuses more than the 8 players there are blocks for.

use crate::errors::{BadInput, Refused};
use crate::net::owner::net_race;
use crate::net::store::{NetShared, inbox_of};
use rc2k_core::net::inbox::{self, Inbox};

/// The blocks' room: 8 players.
pub const MOST_PLAYERS: u32 = rc2k_core::net::players::MOST;
pub fn copy_received(vars: &NetShared, inboxes: &mut [Inbox; 8]) -> Result<(), Refused> {
    let count = || net_race().players();
    if count() > MOST_PLAYERS {
        return Err(BadInput("more than the blocks' 8 players").into());
    }
    let mut player = 0u32;
    loop {
        // The logic is in `rc2k_core::net::inbox::hand_over`, on the inbox.
        let at = vars.player_block(player)?;
        let (handed, records) = inbox::hand_over(inbox_of(inboxes, player)?);
        at.copied_count.set(handed);
        at.copied
            .write_at(0, &records, "a player's copied records")?;
        player += 1;
        if player >= count() {
            break;
        }
    }
    Ok(())
}
