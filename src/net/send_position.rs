//! `net::send_position` (0x464201): "send p", the player and the step logged, and the
//! message 'PGSM' (0x4D534750) with the car's place (+0x134 .. +0x13C), its matrix (+0xB4
//! .. +0xE0) and 25 more dwords of its block (speeds, gear, engine, ...) built in
//! `net_message` and sent (`net::send_race_message`).
//!
//!
//! See `net/mod.rs` and `net::send_inputs` (the shared body). One caller, `net::send_turn`
//! (step 2 of 32).
use crate::errors::Refused;
use crate::net::owner::net;
use crate::net::send_inputs::{Message, send_fields};

pub const POSITION: Message = Message {
    text: c"send p", // "send p"
    kind: rc2k_core::net::race::Kind::Position,
};
pub fn send_position() -> Result<(), Refused> {
    let net = net();
    // Converted: `rc2k_core::net::race::Kind::pack` (in `send_fields`).
    send_fields(net, &POSITION)
}
