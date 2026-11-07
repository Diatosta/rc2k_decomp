//! `net::apply_received` (0x4644A4): the messages received since the last step
//! (`net::copy_received`), each player's in turn (`byte_52AD9C` + player * 0x804C +
//! 0x4040: a count, then records of 0x100 bytes, +1 the length, +2 the message) applied
//! (`net::apply_message`, inside `pusha`: ebx the message, ecx its length, ebp the player).
//! The players' loop ends on `jnz` against `dword_52AD98`: with no players it would not end.
//! The receiving thread keeps a count to 0x40 (`sub_419677`), the room there is.
//!
//! In:  nothing
//! Out: nothing (the original leaves ebx, ecx, edx, esi, ebp as its loops do; its one
//!      caller reads none: `lq -s 0x4644A4`, DF only)
//!
//! See `net/mod.rs`. One caller, `net::race_step`. Refuses (before anything) no players
//! and more than 16.

use crate::errors::{BadInput, Refused};
use crate::net::apply_message::apply_message;
use crate::net::copy_received::{MOST_PLAYERS, copy_received};
use crate::net::store::NetVars;

/// The Rust version.
pub fn apply_received(vars: &NetVars) -> Result<(), Refused> {
    let count = || vars.players.get();
    if count() == 0 || count() > MOST_PLAYERS {
        return Err(BadInput("no players, or more than the records' 16").into());
    }
    copy_received(vars)?;
    let mut player = 0;
    loop {
        let block = vars.player_block(player);
        let mut left = block.copied_count.get();
        if left != 0 {
            let mut at = 0;
            loop {
                // The inbox records are 0x100 bytes each, message bytes from +2.
                let message = block.copied.read_at(at + 2, 0xFE, "a received message")?;
                apply_message(vars, player, &message)?;
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
