//! Logs "send x" and sends the 'XGSM' message with nine dwords of the car's block (shared body
//! in `net::send_inputs`). Called every 256th step.
use crate::errors::Refused;
use crate::net::owner::Net;
use crate::net::send_inputs::{Message, send_fields};
use crate::race::owner::Race;

pub const EXTRA: Message = Message {
    text: c"send x", // "send x"
    kind: rc2k_core::net::race::Kind::Extra,
};
pub fn send_extra(race: &Race, net: &Net) -> Result<(), Refused> {
    send_fields(race, net, &EXTRA)
}
