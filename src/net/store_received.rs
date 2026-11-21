//! Puts a race message from another player into its inbox, kept in sequence order: the sequence
//! byte is doubled (the top bit, which marks a race message, goes) and the time is stored. A
//! message older than the oldest wanted is dropped; one not older than the newest becomes the
//! newest. With no records it is record 0; with 0x40 records the oldest is dropped to make room;
//! otherwise the newer records move up one and it goes in after the last one not newer, with its
//! bytes, the count up one. Refuses a count above 0x40 or a length above 0xFE.

use crate::errors::Refused;
use rc2k_core::net::inbox::{self, Inbox};

/// Stores `message` in `inbox` (the player's, under the lock).
pub fn store_received(
    sequence: u8,
    inbox: &mut Inbox,
    time: u32,
    message: &[u8],
) -> Result<(), Refused> {
    inbox::store(inbox, sequence, time, message)?;
    Ok(())
}
