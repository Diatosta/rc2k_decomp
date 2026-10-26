//! `net::store_received` (0x419677): a race message from another player put into its
//! block of `g_netReceived` (from +0x34: a count, 0x40 records of 0x100 bytes from +0x38:
//! +0 the sequence byte doubled, +1 the length, +2 the bytes; +0x4038 the oldest sequence
//! still wanted (`net::copy_received` sets it from +0x4039), +0x4039 the newest seen,
//! +0x403C the time of the last message), kept in sequence order.
//!
//! The sequence (al) is doubled (`add al, al`: the top bit, which marks a race message,
//! goes). The time is stored. Older than +0x4038 (the difference negative as a signed
//! byte): nothing more. Not older than +0x4039: the new newest. Then: no records, it is
//! record 0; 0x40 records: the first one dropped (the other 0x3F moved down) and the new one
//! written at record 0x40, past the end, over +0x4038 .. (the count not changed; the game's
//! slip, section 16); else the records newer than it moved up one and it goes in after the
//! last one not newer (or first), the count up one. The record: the sequence, cl, and ecx
//! bytes from ebx (`rep movsb`: all of ecx); but once a record was moved up, ecx is 0
//! (the move's `rep movsd`): a message stored between others keeps no length or bytes
//! (the game's slip, section 16).
//!
//! In:  `sequence` (al), `block` (edi: the player's block, a `PlayerBlock`), `time` (edx), `bytes` (ebx) and
//!      `length` (ecx)
//! Out: nothing (every register kept, `pusha` .. `popa`; the flags not read: the one
//!      caller calls `net::unlock_received` next)
//! Caller: `net::take_messages` (under the lock). Refuses (before anything) a count above
//! 0x40 and a length above 0xFE (the original writes past the block or the record; its
//! caller sends neither).

use crate::errors::Refused;
use crate::net::store::PlayerBlock;
use crate::pointers::GamePtr;
use rc2k_core::net::inbox;

/// The Rust version.
pub fn store_received(
    sequence: u8,
    block: &PlayerBlock,
    time: u32,
    bytes: *const u8,
    length: u32,
) -> Result<(), Refused> {
    let base = GamePtr::new(block.inbox_addr() as u32);
    // Converted: `rc2k_core::net::inbox::store`, on the block's inbox and the message.
    let mut inbox = base.read_bytes(inbox::INBOX, "a player's inbox")?;
    let message = GamePtr::new(crate::pointers::game_address(bytes))
        .read_bytes(length as usize, "a received message")?;
    inbox::store(&mut inbox, sequence, time, &message)?;
    base.write_bytes(&inbox, "a player's inbox")?;
    Ok(())
}
