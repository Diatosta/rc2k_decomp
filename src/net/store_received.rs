//! Puts a race message from another player into its inbox, kept in sequence order: the sequence
//! byte is doubled (the top bit, which marks a race message, goes) and the time is stored. A
//! message older than the oldest wanted is dropped; one not older than the newest becomes the
//! newest. With no records it is record 0; with 0x40 records the first is dropped and the new one
//! is written at record 0x40, past the end, over the fields that follow, and the count is not
//! changed (kept as is); otherwise the newer records move up one and it goes in after the last
//! one not newer, the count up one. Once a record was moved up, a message stored between others
//! keeps no length or bytes (kept as is). Refuses a count above 0x40 or a length above 0xFE.

use crate::errors::Refused;
use crate::net::store::PlayerBlock;
use rc2k_core::net::inbox::{self, Inbox};

/// Stores `message` in `inbox` (the player's, under the lock). A message into a full inbox
/// spills over the inbox's sequence and time and the block's copy.
pub fn store_received(
    sequence: u8,
    inbox: &mut Inbox,
    block: &PlayerBlock,
    time: u32,
    message: &[u8],
) -> Result<(), Refused> {
    if let Some(spill) = inbox::store(inbox, sequence, time, message)? {
        inbox.oldest = spill.sequence;
        inbox.newest = spill.length;
        let mut time = inbox.time.to_le_bytes();
        let mut copied_count = block.copied_count.get().to_le_bytes();
        for (k, &byte) in spill.bytes.iter().enumerate() {
            match k {
                0..=1 => {}
                2..=5 => time[k - 2] = byte,
                6..=9 => copied_count[k - 6] = byte,
                _ => block.copied.set(k - 10, byte),
            }
        }
        inbox.time = u32::from_le_bytes(time);
        block.copied_count.set(u32::from_le_bytes(copied_count));
    }
    Ok(())
}
