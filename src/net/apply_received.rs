//! `net::apply_received` (0x4644A4): the messages received since the last step
//! (`net::copy_received`), each player's in turn (`copied_count`,
//! then `copied`: records of 0x100 bytes, +1 the length, +2 the message) applied
//! (`net::apply_message`, with the message, its length and the player).
//! The players' loop ends on `jnz` against `players`: with no players it would not end.
//! The receiving thread keeps a count to 0x40 (`net::store_received`), the room there is.
//!
//! See `net/mod.rs`. One caller, `net::race_step`. Refuses (before anything) no players
//! and more than 8.

use crate::errors::{BadInput, Refused};
use crate::net::apply_message::apply_message;
use crate::net::copy_received::{MOST_PLAYERS, copy_received};
use crate::net::store::NetShared;
use crate::race::owner::Race;
pub fn apply_received(race: &Race, vars: &NetShared) -> Result<(), Refused> {
    let count = || vars.players.get();
    if count() == 0 || count() > MOST_PLAYERS {
        return Err(BadInput("no players, or more than the blocks' 8").into());
    }
    copy_received(vars)?;
    let mut player = 0;
    loop {
        let block = vars.player_block(player)?;
        let mut left = block.copied_count.get();
        if left != 0 {
            let mut at = 0;
            loop {
                // The inbox records are 0x100 bytes each, message bytes from +2.
                let message = block.copied.read_at(at + 2, 0xFE, "a received message")?;
                apply_message(race, vars, player, &message)?;
                at += 0x100;
                left = left.wrapping_sub(1);
                if left == 0 {
                    break;
                }
            }
        }
        player += 1;
        if player == count() {
            return Ok(());
        }
    }
}
