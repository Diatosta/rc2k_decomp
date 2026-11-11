//! `net::send_turn` (0x46415B): the own car's messages by its step count (the current
//! car's block +8): every fourth step (& 3 = 0) `net::send_inputs`, at & 0x1F
//! = 2 `net::send_position`, at & 0xFF = 1 `net::send_extra`.
//!
//!
//! See `net/mod.rs`. One caller, `net::race_step`.

use crate::errors::Refused;
use crate::net::owner::Net;
use crate::net::{send_extra, send_inputs, send_position};
use crate::race::owner::Race;
use rc2k_core::net::race::SENT;
/// One of the three messages sent.
type Send = fn(&Race, &Net) -> Result<(), Refused>;

pub fn send_turn(race: &Race, net: &Net) -> Result<(), Refused> {
    let sends: [Send; 3] = [
        send_inputs::send_inputs,
        send_position::send_position,
        send_extra::send_extra,
    ];
    // Converted: `rc2k_core::net::race::Kind::due`.
    for (kind, send) in SENT.into_iter().zip(sends) {
        if kind.due(race.shared.car.step_clock.get()).0 {
            send(race, net)?;
        }
    }
    Ok(())
}
