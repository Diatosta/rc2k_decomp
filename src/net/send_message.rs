//! A race message to every player: the sequence byte (incremented, top bit set) and the payload
//! go into `send_buffer` and are sent with `net::dplay_send`. Refuses a message over the
//! buffer's 0x100 bytes.

use crate::errors::Refused;
use crate::net::dplay_send::dplay_send;
use crate::net::owner::Net;
use rc2k_core::net::inbox;
use rc2k_core::net::message::{Message, Payload};

/// Sends `payload` as the next race message.
pub(super) fn send_payload(net: &Net, payload: Payload) -> Result<(), Refused> {
    let sequence = inbox::next_sequence(net.lobby().sequence.get());
    let message = Message::Race { sequence, payload }.encode_for_send()?;
    net.lobby().sequence.set(sequence);
    net.lobby()
        .send_buffer
        .write_at(0, &message, "the send buffer")?;
    dplay_send(&net.link, net.link.send_to(), &message);
    Ok(())
}

/// Sends `bytes` as the payload of the next race message (the framing tests' entry).
#[cfg(test)]
pub(super) fn send_message(net: &Net, bytes: &[u8]) -> Result<(), Refused> {
    send_payload(net, Payload::parse(bytes))
}
