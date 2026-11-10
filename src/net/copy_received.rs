//! `net::copy_received` (0x4173A0): under the mutex of the received messages
//! (`net::lock_received` / `net::unlock_received`, `dword_52AB6C`), each player's
//! messages as the receiving thread left them (its record in `byte_52AD9C`, 0x804C bytes:
//! from +0x34, the count and the records, 0x1003 dwords) copied to +0x4040 for
//! `net::apply_received`, the count made 0 and the byte at +0x4039 copied to +0x4038.
//! The loop runs at least once (player 0 with no players).
//!
//!
//! See `net/mod.rs`. Two callers: `net::apply_received` and `net::lets_rally`. Refuses
//! (before anything) more than 16 players (the records' room, up to `dword_5AB270`).

use crate::errors::{BadInput, Refused};
use crate::net::store::NetVars;
use crate::net::{lock_received, unlock_received};
use rc2k_core::net::inbox;

/// The records' room: 16 players.
pub const MOST_PLAYERS: u32 = 16;
pub fn copy_received(vars: &NetVars) -> Result<(), Refused> {
    let count = || vars.players.get();
    if count() > MOST_PLAYERS {
        return Err(BadInput("more than the records' 16 players").into());
    }
    lock_received::lock_received(vars);
    let mut player = 0u32;
    loop {
        // Converted: `rc2k_core::net::inbox::hand_over`, on the inbox and its copy.
        let at = vars.player_block(player);
        let mut both = at.read_inbox(2 * inbox::COPIED)?;
        inbox::hand_over(&mut both);
        at.write_inbox(&both)?;
        player += 1;
        if player >= count() {
            break;
        }
    }
    unlock_received::unlock_received(vars);
    Ok(())
}
