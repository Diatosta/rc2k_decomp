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
use crate::net::{a, g};
use rc2k_core::net::inbox;

const SEQUENCE: usize = 0x52A915; // byte_52A915
const BUFFER: usize = 0x52AA68; // byte_52AA68: 0x100 bytes
const TO: usize = 0x5AB274; // dword_5AB274: whom to send to

/// The Rust version.
pub fn send_message(bytes: *const u8, count: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::net::inbox` (`next_sequence`, `outgoing`: a message too long
    // refused before anything is written).
    unsafe {
        let seq = a(SEQUENCE) as *mut u8;
        let byte = inbox::next_sequence(seq.read_volatile());
        let bytes = std::slice::from_raw_parts(bytes, count as usize);
        let message = inbox::outgoing(byte, bytes)?;
        seq.write_volatile(byte);
        std::ptr::copy(message.as_ptr(), a(BUFFER) as *mut u8, message.len());
    }
    dplay_send(g(TO), 0, a(BUFFER), count + 1);
    Ok(())
}
