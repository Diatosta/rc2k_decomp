//! Puts a race message from another player into its block's records, kept in sequence order:
//! the sequence byte is doubled (the top bit, which marks a race message, goes) and the time is
//! stored. A message older than the oldest wanted is dropped; one not older than the newest
//! becomes the newest. With no records it is record 0; with 0x40 records the first is dropped and
//! the new one is written at record 0x40, past the end, over the fields that follow, and the
//! count is not changed (kept as is); otherwise the newer records move up one and it goes in
//! after the last one not newer, the count up one. Once a record was moved up, a message stored
//! between others keeps no length or bytes (kept as is). Refuses a count above 0x40 or a length
//! above 0xFE.

use crate::errors::Refused;
use crate::net::store::PlayerBlock;
use rc2k_core::net::inbox;
pub fn store_received(
    sequence: u8,
    block: &PlayerBlock,
    time: u32,
    message: &[u8],
) -> Result<(), Refused> {
    // The logic is in `rc2k_core::net::inbox::store`, on the block's inbox and the message.
    let mut inbox = block.inbox();
    let spill = inbox::store(&mut inbox, sequence, time, message)?;
    block.set_inbox(&inbox)?;
    if let Some(spill) = spill {
        block.write_spill(&spill);
    }
    Ok(())
}
