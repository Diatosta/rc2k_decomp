//! A message to every player: the sequence byte (incremented, top bit set) and the bytes go
//! into `send_buffer` and are sent with `net::dplay_send`. Refuses more than 0xFF bytes (the
//! buffer holds 0x100).

use crate::errors::Refused;
use crate::net::dplay_send::dplay_send;
use crate::net::owner::Net;
use rc2k_core::net::inbox;
pub fn send_message(net: &Net, bytes: &[u8]) -> Result<(), Refused> {
    // The logic is in `rc2k_core::net::inbox` (`next_sequence`, `outgoing`: a message too long
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
