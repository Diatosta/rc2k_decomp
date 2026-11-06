//! `net::send_extra` (0x4643FA): "send x", the player and the step logged, and the message
//! 'XGSM' (0x4D534758) with nine dwords of the car's block (+0x64 .. +0x84; not read)
//! built in `unk_71C4D0` and sent (`net::send_race_message`).
//!
//! In:  nothing
//! Out: nothing (the original's registers, as the send leaves them, are read by no caller:
//!      `lq -s` of its one call site, esi only, kept)
//!
//! See `net/mod.rs` and `net::send_inputs` (the shared body). One caller, `net::send_turn`
//! (step 1 of 256).
use crate::errors::Refused;
use crate::net::owner::net;
use crate::net::send_inputs::{Message, send_fields};

pub const EXTRA: Message = Message {
    text: c"send x", // "send x"
    kind: rc2k_core::net::race::Kind::Extra,
};

/// The Rust version.
pub fn send_extra() -> Result<(), Refused> {
    let net = net();
    // Converted: `rc2k_core::net::race::Kind::pack` (in `send_fields`).
    send_fields(net, &EXTRA)
}
