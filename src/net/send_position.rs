//! `net::send_position` (0x464201): "send p", the player and the step logged, and the
//! message 'PGSM' (0x4D534750) with the car's place (+0x134 .. +0x13C), its matrix (+0xB4
//! .. +0xE0) and 25 more dwords of its block (speeds, gear, engine, ...) built in
//! `unk_71C4D0` and sent (`net::send_race_message`).
//!
//! In:  nothing
//! Out: as `send_race_message` leaves them (in: eax the last dword, ebx the message, ecx
//!      its bytes, the rest kept)
//!
//! See `net/mod.rs` and `net::send_inputs` (the shared body). One caller, `net::send_turn`
//! (step 2 of 32). Its calls through `part`. Shadow mode runs the original alone. Self-test: a
//! copy whose car block and message are the test's and whose calls go to noting
//! stand-ins, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::send_inputs::{Message, send_fields};
use crate::net::{Part, net_entry};

pub const POSITION: Message = Message {
    text: 0x4DE0E0, // "send p"
    kind: rc2k_core::net::race::Kind::Position,
};

/// The Rust version.
pub fn send_position(r: &mut Regs, part: Part) -> Result<(), Refused> {
    // Converted: `rc2k_core::net::race::Kind::pack` (in `send_fields`).
    send_fields(r, &POSITION, part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, send_position)
}
