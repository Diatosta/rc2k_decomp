//! Logs "send p" and sends the 'PGSM' message with the car's place, matrix and 25 more dwords
//! (speeds, gear, engine, ...). Called at step 2 of 32.
use crate::errors::Refused;
use crate::net::owner::Net;
use crate::net::send_inputs::{Message, send_fields};
use crate::race::owner::Race;

pub const POSITION: Message = Message {
    text: c"send p", // "send p"
    kind: rc2k_core::net::race::Kind::Position,
};
pub fn send_position(race: &Race, net: &Net) -> Result<(), Refused> {
    send_fields(race, net, &POSITION)
}
