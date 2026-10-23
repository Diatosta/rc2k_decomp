//! `net::send_inputs` (0x464194): "send j", the player and the step logged, and the
//! message 'JGSM' (0x4D53474A) with the car's four input dwords (the current car's block
//! +0xFD0 .. +0xFDC) built in `unk_71C4D0` and sent (`net::send_race_message`).
//!
//! In:  nothing
//! Out: as `send_race_message` leaves them (in: eax the last dword, ebx the message, ecx
//!      its bytes, the rest kept)
//!
//! See `net/mod.rs`; `send_position` and `send_extra` are the same with other dwords (this
//! file has their shared body). One caller, `net::send_turn` (every fourth step). Its
//! calls through `part` (the log's and the send). Shadow mode runs the original alone.
//! Self-test: a copy whose car block and message are the test's and whose calls go to
//! noting stand-ins, against the Rust version (also the other two's tests).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::{CAR, MESSAGE, Part, SEND_RACE, a, log_send, net_entry};
use rc2k_core::net::race::Kind;

/// What one of the three sends: its log text (an RVA as IDA shows it) and its kind (the
/// tag and the car block's dwords: `rc2k_core::net::race::Kind`).
pub struct Message {
    pub text: usize,
    pub kind: Kind,
}

pub const INPUTS: Message = Message {
    text: 0x4DE0D9, // "send j"
    kind: Kind::Inputs,
};

/// The shared body on a car block and a message buffer at the addresses given.
pub fn send_fields_with(
    r: &mut Regs,
    m: &Message,
    car: u32,
    message: u32,
    part: Part,
) -> Result<(), Refused> {
    log_send(m.text, r, part)?;
    // Converted: `rc2k_core::net::race::Kind::pack`.
    let dwords = m
        .kind
        .pack(|field| unsafe { ((car + field) as *const u32).read_unaligned() });
    let out = message as *mut u32;
    for (n, &v) in dwords.iter().enumerate() {
        unsafe { out.add(n).write_unaligned(v) };
    }
    r.eax = *dwords.last().unwrap_or(&0);
    r.ebx = message;
    r.ecx = 4 * dwords.len() as u32;
    part(SEND_RACE, r)
}

/// In the game.
pub fn send_fields(r: &mut Regs, m: &Message, part: Part) -> Result<(), Refused> {
    send_fields_with(r, m, a(CAR), a(MESSAGE), part)
}

/// The Rust version.
pub fn send_inputs(r: &mut Regs, part: Part) -> Result<(), Refused> {
    send_fields(r, &INPUTS, part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, send_inputs)
}
