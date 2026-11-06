//! `net::send_message` (0x41736D): a message to every player: the sequence byte
//! (`byte_52A915` + 1, top bit set, kept) and the bytes into `byte_52AA68`, sent
//! (`net::dplay_send`: to `dword_5AB274`, the bytes and their count, one more than given,
//! flags 0).
//!
//! In:  `bytes` (ebx), `count` (ecx)
//! Out: nothing (the original leaves eax, ebx, ecx, edx as the send has them and keeps
//!      esi, edi; its callers, `net::send_race_message`'s, read esi only: `lq -s
//!      0x46449E`, kept by a Rust function)
//!
//! See `net/mod.rs`. Callers: `net::send_inputs`, `net::send_position`, `net::send_extra`
//! (through `net::send_race_message`, `call net::send_message; retn`, folded in). Refuses
//! (before anything) more than 0xFF bytes: the buffer is 0x100 (the original writes on
//! into `dword_52AB68`).

use crate::errors::Refused;
use crate::net::dplay_send::dplay_send;
use crate::pointers::GamePtr;
use rc2k_core::net::inbox;

/// The Rust version.
pub fn send_message(bytes: GamePtr, count: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::net::inbox` (`next_sequence`, `outgoing`: a message too long
    // refused before anything is written).
    let byte = inbox::next_sequence(crate::net::owner::net().vars.sequence.get());
    let bytes = bytes.read_bytes(count as usize, "a message to send")?;
    let message = inbox::outgoing(byte, &bytes)?;
    crate::net::owner::net().vars.sequence.set(byte);
    crate::net::owner::net()
        .vars
        .send_buffer
        .write_at(0, &message, "the send buffer")?;
    dplay_send(crate::net::owner::net().vars.send_to.get(), &message);
    Ok(())
}
