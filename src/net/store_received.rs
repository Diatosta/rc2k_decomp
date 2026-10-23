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
//! In:  al = the sequence byte, edi = the player's block, edx = the time, ecx = the length,
//!      ebx = the bytes
//! Out: every register kept (`pusha` .. `popa`); the flags not modelled (the one caller
//!      calls `net::unlock_received` next, which sets its own)
//! Caller: `net::take_messages` (under the lock). Refuses (before anything) a count above
//! 0x40 and a length above 0xFE (the original writes past the block or the record; its
//! caller sends neither). Shadow mode runs the original alone (the caller's lock).
//! Self-test: a copy against the Rust version on a test block: empty, appended, inserted,
//! first, too old, full, the sequence wrapping, lengths 0 and 0xFE; the refusals.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::{Part, net_entry};
use rc2k_core::net::inbox;

/// The Rust version.
pub fn store_received(r: &mut Regs, _part: Part) -> Result<(), Refused> {
    let base = r.edi.wrapping_add(0x34);
    // Converted: `rc2k_core::net::inbox::store`, on the block's inbox and the message.
    let inbox = unsafe { std::slice::from_raw_parts_mut(base as *mut u8, inbox::INBOX) };
    let message = unsafe { std::slice::from_raw_parts(r.ebx as *const u8, r.ecx as usize) };
    inbox::store(inbox, r.eax as u8, r.edx, message)?;
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, store_received)
}
