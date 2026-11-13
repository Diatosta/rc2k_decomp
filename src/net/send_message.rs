//! `net::send_message` (0x41736D): a message to every player: the sequence byte
//! (`sequence` + 1, top bit set, kept) and the bytes into `send_buffer`, sent
//! (`net::dplay_send`: to `send_to`, the bytes and their count, one more than given,
//! flags 0).
//!
//! Takes `bytes`, `count`.
//!
//! See `net/mod.rs`. Callers: `net::send_inputs`, `net::send_position`, `net::send_extra`
//! (through `net::send_race_message`). Refuses
//! (before anything) more than 0xFF bytes: the buffer is 0x100 (the original writes on
//! into the receiving thread's event handle).

use crate::errors::Refused;
use crate::net::dplay_send::dplay_send;
use crate::net::owner::Net;
use rc2k_core::net::inbox;
pub fn send_message(net: &Net, bytes: &[u8]) -> Result<(), Refused> {
    // Converted: `rc2k_core::net::inbox` (`next_sequence`, `outgoing`: a message too long
    // refused before anything is written).
    let byte = inbox::next_sequence(net.lobby().sequence.get());
    let message = inbox::outgoing(byte, bytes)?;
    net.lobby().sequence.set(byte);
    net.lobby()
        .send_buffer
        .write_at(0, &message, "the send buffer")?;
    dplay_send(&net.vars, net.vars.send_to.get(), &message);
    Ok(())
}
