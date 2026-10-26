//! `net::send_message` (0x41736D): a message to every player: the sequence byte
//! (`byte_52A915` + 1, top bit set, kept) and the ecx bytes at ebx into `byte_52AA68`,
//! sent (DirectPlay's Send, `sub_4196FF`: eax to whom, `dword_5AB274`; ebx the bytes; ecx
//! their count, one more than given; edx 0 the flags).
//!
//! In:  ebx = the bytes, ecx = how many
//! Out: eax `dword_5AB274`, ebx `byte_52AA68`, ecx the count + 1, edx 0 (as the send
//!      keeps them), esi and edi kept; the carry the send's (set if it failed; not read)
//!
//! See `net/mod.rs`. One caller, `net::send_race_message`. Its call through `part` (in
//! the game DirectPlay's, through the game). Refuses (before anything) more than 0xFF
//! bytes: the buffer is 0x100 (the original writes on into `dword_52AB68`). Shadow mode
//! runs the original alone. Self-test: a copy whose three variables are the test's and
//! whose call goes to a noting stand-in, against the Rust version: counts 0, 1, 0x40,
//! 0xFF, sequence bytes either side of the wrap.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::{Part, SEND, a, net_entry};
use rc2k_core::net::inbox;

const SEQUENCE: usize = 0x52A915; // byte_52A915
const BUFFER: usize = 0x52AA68; // byte_52AA68: 0x100 bytes
const TO: usize = 0x5AB274; // dword_5AB274: whom to send to

/// Where the variables are.
#[derive(Clone, Copy)]
pub struct Vars {
    pub sequence: u32,
    pub buffer: u32,
    pub to: u32,
}

impl Vars {
    pub fn game() -> Self {
        Self {
            sequence: a(SEQUENCE),
            buffer: a(BUFFER),
            to: a(TO),
        }
    }
}

/// The Rust version on the variables in `v`.
pub fn send_message_with(r: &mut Regs, v: &Vars, part: Part) -> Result<(), Refused> {
    let count = r.ecx;
    // Converted: `rc2k_core::net::inbox` (`next_sequence`, `outgoing`: a message too long
    // refused before anything is written).
    unsafe {
        let seq = v.sequence as *mut u8;
        let byte = inbox::next_sequence(seq.read_volatile());
        let bytes = std::slice::from_raw_parts(r.ebx as *const u8, count as usize);
        let message = inbox::outgoing(byte, bytes)?;
        seq.write_volatile(byte);
        std::ptr::copy(message.as_ptr(), v.buffer as *mut u8, message.len());
        r.eax = (v.to as *const u32).read_volatile();
    }
    let (esi, edi) = (r.esi, r.edi);
    // As `rep movsb` leaves them at the send.
    r.esi = r.ebx.wrapping_add(count);
    r.edi = v.buffer + 1 + count;
    r.ebx = v.buffer;
    r.ecx = count + 1;
    r.edx = 0;
    part(SEND, r)?;
    r.esi = esi;
    r.edi = edi;
    Ok(())
}

/// The Rust version.
pub fn send_message(r: &mut Regs, part: Part) -> Result<(), Refused> {
    send_message_with(r, &Vars::game(), part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, send_message)
}
